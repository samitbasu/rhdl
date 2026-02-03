use crate::{
    trace_string::{TraceBit, bit_char},
    utils::{discriminant, not_present, trace_type_width_in_bits},
};
use color_eyre::Result;
use ecolor::Color32;
use rhdl_trace_type::{Color, TraceType};
use surfer_translation_types::{
    SubFieldTranslationResult, TranslationResult, ValueKind, ValueRepr,
};

pub fn translate_raw(
    bits: &[TraceBit],
    ty: &TraceType,
    kind: ValueKind,
) -> Result<TranslationResult> {
    let width = trace_type_width_in_bits(ty);
    if bits.len() < width {
        return Err(color_eyre::eyre::eyre!("Invalid bit width"));
    }
    match ty {
        TraceType::Empty => Ok(TranslationResult {
            val: ValueRepr::Tuple,
            subfields: vec![],
            kind,
        }),
        TraceType::Clock | TraceType::Reset | TraceType::Bits(1) | TraceType::Signed(1) => {
            Ok(TranslationResult {
                val: ValueRepr::Bit(bit_char(bits[0])),
                subfields: vec![],
                kind,
            })
        }
        // Placeholder
        TraceType::Signed(n) | TraceType::Bits(n) => {
            let s: String = bits.iter().take(*n).copied().rev().map(bit_char).collect();
            Ok(TranslationResult {
                kind,
                val: ValueRepr::Bits(*n as u64, s),
                subfields: vec![],
            })
        }
        TraceType::Array(inner) => {
            let mut subfields = vec![];
            let mut bits = bits;
            for i in 0..inner.size {
                let width = trace_type_width_in_bits(&inner.base);
                let sub = translate_raw(&bits[0..width], &inner.base, kind)?;
                bits = &bits[width..];
                subfields.push(SubFieldTranslationResult::new(i.to_string(), sub));
            }
            Ok(TranslationResult {
                val: ValueRepr::Array,
                subfields,
                kind,
            })
        }
        TraceType::Tuple(n) => {
            let mut subfields = vec![];
            let mut bits = bits;
            for (ndx, ty) in n.elements.iter().enumerate() {
                let width = trace_type_width_in_bits(ty);
                let sub = translate_raw(&bits[0..width], ty, kind)?;
                bits = &bits[width..];
                subfields.push(SubFieldTranslationResult::new(ndx, sub));
            }
            Ok(TranslationResult {
                val: ValueRepr::Tuple,
                subfields,
                kind,
            })
        }
        TraceType::Struct(inner) => {
            let mut subfields = vec![];
            let mut bits = bits;
            for field in inner.fields.iter() {
                let width = trace_type_width_in_bits(&field.ty);
                let sub = translate_raw(&bits[0..width], &field.ty, kind)?;
                bits = &bits[width..];
                subfields.push(SubFieldTranslationResult::new(&field.name, sub));
            }
            Ok(TranslationResult {
                val: ValueRepr::Struct,
                subfields,
                kind,
            })
        }
        TraceType::Signal(inner, color) => {
            let kind = match color {
                Color::Red => ValueKind::Custom(Color32::DARK_RED),
                Color::Orange => ValueKind::Custom(Color32::from_rgb(255, 127, 0)),
                Color::Yellow => ValueKind::Custom(Color32::from_rgb(255, 255, 0)),
                Color::Green => ValueKind::Custom(Color32::from_rgb(0, 255, 0)),
                Color::Blue => ValueKind::Custom(Color32::from_rgb(0, 0, 255)),
                Color::Indigo => ValueKind::Custom(Color32::from_rgb(75, 0, 130)),
                Color::Violet => ValueKind::Custom(Color32::from_rgb(148, 0, 211)),
                _ => ValueKind::Normal,
            };
            let inner = translate_raw(bits, inner, kind)?;
            Ok(TranslationResult {
                val: ValueRepr::Struct,
                subfields: vec![SubFieldTranslationResult::new("value", inner)],
                kind,
            })
        }
        TraceType::Enum(inner) => {
            let layout = inner.discriminant_layout;
            let Ok((discr, payload)) = discriminant(bits, &layout) else {
                return Ok(TranslationResult {
                    val: ValueRepr::String(
                        bits.iter()
                            .take(width)
                            .copied()
                            .rev()
                            .map(bit_char)
                            .collect(),
                    ),
                    subfields: vec![],
                    kind,
                });
            };
            let mut subfields = vec![];
            let mut idx = 0;
            for (ndx, variant) in inner.variants.iter().enumerate() {
                if discr == variant.discriminant {
                    idx = ndx;
                    let sub = translate_raw(payload, &variant.ty, kind)?;
                    subfields.push(SubFieldTranslationResult::new(&variant.name, sub));
                } else {
                    let sub = not_present(&variant.ty);
                    subfields.push(SubFieldTranslationResult::new(&variant.name, sub));
                }
            }
            Ok(TranslationResult {
                val: ValueRepr::Enum {
                    idx,
                    name: inner.variants[idx].name.clone(),
                },
                subfields,
                kind,
            })
        }
        _ => Ok(TranslationResult {
            val: ValueRepr::String(
                bits.iter()
                    .take(width)
                    .copied()
                    .rev()
                    .map(bit_char)
                    .collect(),
            ),
            subfields: vec![],
            kind,
        }),
    }
}

#[cfg(test)]
mod tests {
    use rhdl_trace_type::RTT;

    #[test]
    fn test_translation_of_option() {
        use super::*;
        let RTT::TraceInfo(rtt) =
            ron::de::from_str(include_str!("test/async_fifo_trace.vcd.rhdl")).unwrap();
        let foo = translate_raw(
            [TraceBit::Zero; 17].as_slice(),
            rtt.get("top.drainer.input").unwrap(),
            ValueKind::Normal,
        )
        .unwrap();
        eprintln!("{}", serde_json::to_string_pretty(&foo).unwrap());
    }
}
