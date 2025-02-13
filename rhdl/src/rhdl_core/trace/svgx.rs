use crate::{
    prelude::{BitX, Digital, Kind, Path},
    rhdl_core::TypedBits,
};

// We want to take a series of time/bool values and turn it into an SVG thing.
// The underlying time series is a set of time/impl Digital values.  So the
// translation needs to go from a type T to something representable.
// Because we can only represent scalars in the SVG, we need to slice the
// data type into it's composite low level parts.
//
// A trace consists of two parts.
//   <label> : - a string that identifies the trace to the reader
//   [data]  : - a set of data points
//  A data point can either be a bool, a vector or a string.
//  A bool region is defined by a start and end time and a value (either true or false).
//  A data region is defined by a start and end time and a string describing the value.
//  A trace set is hierarchical.  So we can have a main trace contain data as well as subtraces.

enum RegionTag {
    Bool(bool),
    Unsigned(u128),
    Signed(i128),
    String(String),
}

struct Region {
    start: u64,
    end: u64,
    tag: Option<RegionTag>,
}

struct TraceSet {
    label: String,
    data: Vec<Region>,
    children: Vec<TraceSet>,
}

pub fn trace_out<T: Digital>(
    label: &str,
    db: &[(u64, T)],
    range: std::ops::RangeInclusive<u64>,
) -> TraceSet {
    let kind = T::static_kind();
    todo!()
}

pub fn format_as_label(t: &TypedBits) -> Option<String> {
    match t.kind {
        Kind::Array(inner) => {
            let vals = (0..inner.size)
                .flat_map(|i| t.path(&Path::default().index(i)).ok())
                .flat_map(|element| format_as_label(&element))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("[{}]", vals))
        }
        Kind::Tuple(inner) => {
            let vals = inner
                .elements
                .iter()
                .enumerate()
                .flat_map(|(i, _)| t.path(&Path::default().tuple_index(i)).ok())
                .flat_map(|element| format_as_label(&element))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("({})", vals))
        }
        Kind::Struct(inner) => {
            let vals = inner
                .fields
                .iter()
                .flat_map(|field| {
                    t.path(&Path::default().field(&field.name))
                        .map(|x| (field, x))
                        .ok()
                })
                .flat_map(|(name, field)| format_as_label(&field).map(|x| (name, x)))
                .map(|(name, val)| format!("{}: {}", name.name, val))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("{{{}}}", vals))
        }
        Kind::Enum(inner) => {
            let discriminant = t.discriminant().ok()?.as_i64().ok()?;
            let variant = inner
                .variants
                .iter()
                .find(|v| v.discriminant == discriminant)?;
            let payload = t
                .path(&Path::default().payload_by_value(discriminant))
                .ok()?;
            let payload = format_as_label(&payload).unwrap_or_default();
            Some(format!("{}{}", variant.name, payload))
        }
        Kind::Bits(inner) => {
            let mut val: u128 = 0;
            for ndx in 0..inner {
                if t.bits[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            let num_nibbles = inner / 4 + if inner % 4 == 0 { 0 } else { 1 };
            // Format the val as a hex number with the given
            // number of nibbles, with left padding of zeros
            Some(format!("{:0width$x}", val, width = num_nibbles))
        }
        Kind::Signed(inner) => {
            let mut val: i128 = 0;
            for ndx in 0..inner {
                if t.bits[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            if val & (1 << (inner - 1)) != 0 {
                val |= !0 << inner;
            }
            Some(format!("{}", val))
        }
        Kind::Signal(_inner, color) => {
            let val = &t.val();
            let val = format_as_label(val)?;
            Some(format!("{:?}@({})", color, val))
        }
        Kind::Empty => None,
    }
}
