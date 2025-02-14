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

struct Region {
    start: u64,
    end: u64,
    tag: Option<String>,
    kind: RegionKind,
}

enum RegionKind {
    True,
    False,
    Multibit,
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

fn build_time_trace<T: Digital>(data: &[(u64, T)], path: &Path) -> Vec<Region> {
    slice_by_path_and_bucketize(data, path)
        .iter()
        .map(map_bucket_to_region)
        .collect()
}

fn slice_by_path_and_bucketize<T: Digital>(data: &[(u64, T)], path: &Path) -> Vec<Bucket> {
    let sliced = data
        .iter()
        .map(|(time, value)| (*time, value.typed_bits()))
        .flat_map(|(time, tb)| tb.path(path).ok().map(|tb| (time, tb)));
    bucketize(sliced)
}

fn path_slice<T: Digital>(data: &[(u64, T)], path: &Path) -> Vec<(u64, TypedBits)> {
    data.iter()
        .map(|(time, value)| (*time, value.typed_bits()))
        .flat_map(|(time, tb)| tb.path(path).ok().map(|tb| (time, tb)))
        .collect()
}

fn map_bucket_to_region(bucket: &Bucket) -> Region {
    let kind = match bucket.data.bits.len() {
        1 => match bucket.data.bits[0] {
            BitX::Zero => RegionKind::False,
            BitX::One => RegionKind::True,
            _ => RegionKind::Multibit,
        },
        _ => RegionKind::Multibit,
    };
    Region {
        start: bucket.start,
        end: bucket.end,
        tag: format_as_label(&bucket.data),
        kind,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Bucket {
    start: u64,
    end: u64,
    data: TypedBits,
}

fn bucketize(data: impl IntoIterator<Item = (u64, TypedBits)>) -> Vec<Bucket> {
    let mut buckets = Vec::new();
    let mut last_time = !0;
    let mut last_data = TypedBits::EMPTY;
    let mut start_time = !0;
    for (time, data) in data.into_iter() {
        if last_time == !0 {
            last_time = time;
            start_time = time;
            last_data = data.clone();
        } else {
            if !last_data.eq(&data) {
                buckets.push(Bucket {
                    start: start_time,
                    end: time,
                    data: last_data.clone(),
                });
                start_time = time;
                last_data = data.clone();
            }
            last_time = time;
        }
    }
    if last_time != !0 {
        buckets.push(Bucket {
            start: start_time,
            end: last_time,
            data: last_data,
        });
    }
    buckets
}

pub fn format_as_label(t: &TypedBits) -> Option<String> {
    (t.bits.len() != 1).then(|| format_as_label_inner(t))?
}

fn format_as_label_inner(t: &TypedBits) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use crate::prelude::b8;

    use super::*;

    #[test]
    fn test_bucket_empty() {
        let data = [];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 0);
    }

    #[test]
    fn test_bucket_single() {
        let data = [(0, b8(8).typed_bits())];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 1);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 0,
                data: b8(8).typed_bits()
            }
        );
    }

    #[test]
    fn test_buckets_no_transition() {
        let n8 = b8(8).typed_bits();
        let data = [(0, n8.clone()), (1, n8.clone()), (2, n8.clone())];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 1);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 2,
                data: n8
            }
        );
    }

    #[test]
    fn test_buckets() {
        let data = [
            (0, b8(8).typed_bits()),
            (1, b8(4).typed_bits()),
            (3, b8(5).typed_bits()),
        ];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 3);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 1,
                data: b8(8).typed_bits()
            }
        );
        assert_eq!(
            buckets[1],
            Bucket {
                start: 1,
                end: 3,
                data: b8(4).typed_bits()
            }
        );
        assert_eq!(
            buckets[2],
            Bucket {
                start: 3,
                end: 3,
                data: b8(5).typed_bits()
            }
        );
    }
}
