use crate::trace_string::TraceBit;
use color_eyre::Result;
use rhdl_trace_type::TraceType;
use surfer_translation_types::{
    SubFieldTranslationResult, TranslationResult, ValueKind, ValueRepr, VariableInfo,
};

pub fn trace_type_width_in_bits(ty: &TraceType) -> usize {
    match ty {
        TraceType::Empty => 0,
        TraceType::Signed(x) | TraceType::Bits(x) => *x,
        TraceType::Clock | TraceType::Reset => 1,
        TraceType::Array(inner) => inner.size * trace_type_width_in_bits(&inner.base),
        TraceType::Struct(inner) => inner
            .fields
            .iter()
            .map(|field| trace_type_width_in_bits(&field.ty))
            .sum(),
        TraceType::Tuple(inner) => inner.elements.iter().map(trace_type_width_in_bits).sum(),
        TraceType::Enum(inner) => {
            inner.discriminant_layout.width
                + inner
                    .variants
                    .iter()
                    .map(|v| trace_type_width_in_bits(&v.ty))
                    .max()
                    .unwrap_or(0)
        }
        TraceType::Signal(inner, _) => trace_type_width_in_bits(inner),
        _ => 0,
    }
}

/// Return kind for a binary representation
pub fn color_for_binary_representation(s: &str) -> ValueKind {
    if s.contains('x') {
        ValueKind::Undef
    } else if s.contains('z') {
        ValueKind::HighImp
    } else if s.contains('-') {
        ValueKind::DontCare
    } else if s.contains('u') || s.contains('w') {
        ValueKind::Undef
    } else if s.contains('h') || s.contains('l') {
        ValueKind::Weak
    } else {
        ValueKind::Normal
    }
}

pub fn not_present(ty: &TraceType) -> TranslationResult {
    match ty {
        TraceType::Empty | TraceType::Bits(_) | TraceType::Signed(_) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: vec![],
            kind: ValueKind::Normal,
        },
        TraceType::Array(inner) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: (0..inner.size)
                .map(|i| SubFieldTranslationResult::new(i.to_string(), not_present(&inner.base)))
                .collect(),
            kind: ValueKind::Normal,
        },
        TraceType::Struct(inner) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: inner
                .fields
                .iter()
                .map(|field| SubFieldTranslationResult::new(&field.name, not_present(&field.ty)))
                .collect(),
            kind: ValueKind::Normal,
        },
        TraceType::Tuple(inner) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: inner
                .elements
                .iter()
                .enumerate()
                .map(|(i, ty)| SubFieldTranslationResult::new(i.to_string(), not_present(ty)))
                .collect(),
            kind: ValueKind::Normal,
        },
        TraceType::Enum(inner) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: inner
                .variants
                .iter()
                .map(|v| SubFieldTranslationResult::new(&v.name, not_present(&v.ty)))
                .collect(),
            kind: ValueKind::Normal,
        },
        TraceType::Signal(inner, _) => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: vec![SubFieldTranslationResult::new("value", not_present(&inner))],
            kind: ValueKind::Normal,
        },
        _ => TranslationResult {
            val: ValueRepr::NotPresent,
            subfields: vec![],
            kind: ValueKind::Normal,
        },
    }
}

// Use an i128 here - the enum discriminant in RHDL is guaranteed to be less than 64 bits wide
fn try_i128(bits: &[TraceBit]) -> Result<i128> {
    let mut v = 0_i128;
    for (ndx, bit) in bits.iter().enumerate() {
        match *bit {
            TraceBit::Zero => {}
            TraceBit::One => {
                v |= 1 << ndx;
            }
            _ => {
                return Err(color_eyre::eyre::eyre!("Invalid bit value"));
            }
        }
    }
    Ok(v)
}

pub fn discriminant<'a>(
    bits: &'a [TraceBit],
    layout: &'a rhdl_trace_type::DiscriminantLayout,
) -> Result<(i64, &'a [TraceBit])> {
    let width = bits.len();
    let disc_width = layout.width;
    let disc_start = match layout.alignment {
        rhdl_trace_type::DiscriminantAlignment::Lsb => 0,
        rhdl_trace_type::DiscriminantAlignment::Msb => width - disc_width,
    };
    let payload_start = match layout.alignment {
        rhdl_trace_type::DiscriminantAlignment::Lsb => disc_width,
        rhdl_trace_type::DiscriminantAlignment::Msb => 0,
    };
    let payload_width = width - disc_width;
    let disc_bits = &bits[disc_start..(disc_start + disc_width)];
    let payload_bits = &bits[payload_start..(payload_start + payload_width)];
    let disc_uint = try_i128(disc_bits)?;
    Ok((
        match layout.ty {
            rhdl_trace_type::DiscriminantType::Unsigned => disc_uint as i64,
            rhdl_trace_type::DiscriminantType::Signed => {
                let sign_weight = 1_i128 << (disc_width - 1);
                if disc_uint < sign_weight {
                    disc_uint as i64
                } else {
                    ((sign_weight << 1) - disc_uint) as i64
                }
            }
        },
        payload_bits,
    ))
}

pub fn trace_type_to_variable_info(ty: &TraceType) -> VariableInfo {
    match ty {
        TraceType::Signed(1) | TraceType::Bits(1) => VariableInfo::Bool,
        TraceType::Empty | TraceType::Signed(_) | TraceType::Bits(_) => VariableInfo::Bits,
        TraceType::Clock => VariableInfo::Clock,
        TraceType::Reset => VariableInfo::Bool,
        TraceType::Tuple(inner) => VariableInfo::Compound {
            subfields: inner
                .elements
                .iter()
                .enumerate()
                .map(|(i, ty)| (format!("{i}"), trace_type_to_variable_info(ty)))
                .collect(),
        },
        TraceType::Struct(inner) => VariableInfo::Compound {
            subfields: inner
                .fields
                .iter()
                .map(|field| (field.name.clone(), trace_type_to_variable_info(&field.ty)))
                .collect(),
        },
        TraceType::Array(inner) => VariableInfo::Compound {
            subfields: (0..inner.size)
                .map(|i| (format!("{i}"), trace_type_to_variable_info(&inner.base)))
                .collect(),
        },
        TraceType::Enum(inner) => VariableInfo::Compound {
            subfields: inner
                .variants
                .iter()
                .map(|v| (v.name.clone(), trace_type_to_variable_info(&v.ty)))
                .collect(),
        },
        TraceType::Signal(inner, _) => VariableInfo::Compound {
            subfields: vec![("value".to_string(), trace_type_to_variable_info(inner))],
        },
        _ => VariableInfo::Bits,
    }
}
