use crate::{
    prelude::{BitX, Digital, Kind, Path},
    rhdl_core::{types::path::PathElement, TypedBits},
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
//  A data point can either be a bool, or a vector (string is included as a vector).
//  A bool region is defined by a start and end time and a value (either true or false).
//  A data region is defined by a start and end time and a string describing the value.

#[derive(Clone, Debug)]
pub struct SvgRegion {
    start_x: i32,
    start_y: i32,
    width: i32,
    tag: String,
    full_tag: String,
    kind: RegionKind,
}

pub struct SvgOptions {
    pub pixels_per_time_unit: f32,
    pub font_size_in_pixels: f32,
    pub spacing: i32,
}

// Vertically stack a set of svg regions with the given gap
fn stack_svg_regions(regions: &[Box<[SvgRegion]>], options: &SvgOptions) -> Box<[SvgRegion]> {
    let mut start_y = 0;
    let mut result = Vec::new();
    for region in regions {
        for r in region.iter() {
            let mut r = r.clone();
            r.start_y = start_y;
            result.push(r);
        }
        start_y += options.spacing;
    }
    result.into()
}

// Convert a sequence of Regions into a sequence of SVG regions
// The pixels_per_time_unit scales a delta_t in the simulation units to the number of "pixels" that
// will be used in the SVG.
fn regions_to_svg_regions(regions: &[Region], options: &SvgOptions) -> Box<[SvgRegion]> {
    regions
        .iter()
        .map(|r| {
            let width = ((r.end - r.start) as f32 * options.pixels_per_time_unit) as i32;
            let start_x = (r.start as f32 * options.pixels_per_time_unit) as i32;
            let kind = r.kind;
            let width_in_characters = (width as f32 / options.font_size_in_pixels) as usize;
            let mut tag = r.tag.clone().unwrap_or_default();
            let full_tag = tag.clone();
            if tag.len() > width_in_characters {
                while !tag.is_empty() && tag.len() + 3 > width_in_characters {
                    tag.pop();
                }
                if !tag.is_empty() {
                    tag += "...";
                }
            }
            let region = SvgRegion {
                start_x,
                start_y: 0,
                full_tag,
                width,
                tag,
                kind,
            };
            region
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct Region {
    start: u64,
    end: u64,
    tag: Option<String>,
    kind: RegionKind,
}

#[derive(Copy, Clone, Debug)]
enum RegionKind {
    True,
    False,
    Multibit,
    Label,
}

#[derive(Debug)]
pub struct Trace {
    pub label: String,
    pub data: Box<[Region]>,
}

fn render_trace_to_svg(trace: &Trace, options: &SvgOptions) -> Box<[SvgRegion]> {
    regions_to_svg_regions(&trace.data, options)
}

pub fn render_traces_to_svg(traces: &[Trace], options: &SvgOptions) -> Box<[SvgRegion]> {
    stack_svg_regions(
        &traces
            .iter()
            .map(|trace| render_trace_to_svg(trace, options))
            .collect::<Vec<_>>(),
        options,
    )
}

// TODO - remove the duplication
pub fn render_traces_as_svg_document(traces: &[Trace], options: &SvgOptions) -> svg::Document {
    let regions = render_traces_to_svg(traces, options);
    let width = regions
        .iter()
        .map(|r| r.start_x + r.width)
        .max()
        .unwrap_or_default();
    let height = regions
        .iter()
        .map(|r| r.start_y + options.spacing)
        .max()
        .unwrap_or_default();
    let mut document = svg::Document::new().set("viewBox", (0, 0, width, height));
    // Provide a background rectangle for the diagram of light gray
    let background = svg::node::element::Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", width)
        .set("height", height)
        .set("fill", "#EEEEEE")
        .set("stroke", "darkblue");
    document = document.add(background);
    // For each cell, add a rectangle to the SVG with the
    // name of the cell centered in the rectangle
    for region in regions {
        let x = region.start_x;
        let y = region.start_y;
        let width = region.width;
        let height = options.spacing;
        let fill_color = match region.kind {
            RegionKind::True => "green",
            RegionKind::False => "red",
            RegionKind::Multibit => "blue",
            RegionKind::Label => "black",
        };
        let stroke_color = "black";
        let text = region.tag.clone();
        eprintln!("{x} {y} {full_tag}", full_tag = region.full_tag);
        document = crate::rhdl_core::types::svg::kind_svg::text_box(
            (x, y, width, height),
            &text,
            fill_color,
            stroke_color,
            document,
        );
    }
    document
}

pub fn trace_out<T: Digital>(label: &str, db: &[(u64, T)]) -> Box<[Trace]> {
    let kind = T::static_kind();
    pretty_leaf_paths(&kind, Path::default())
        .into_iter()
        .map(|path| {
            let data = build_time_trace(db, &path);
            Trace {
                label: format!("{label}{:?}", path),
                data,
            }
        })
        .collect()
}

pub fn build_time_trace<T: Digital>(data: &[(u64, T)], path: &Path) -> Box<[Region]> {
    slice_by_path_and_bucketize(data, path)
        .iter()
        .map(map_bucket_to_region)
        .collect()
}

fn slice_by_path_and_bucketize<T: Digital>(data: &[(u64, T)], path: &Path) -> Box<[Bucket]> {
    let sliced = data
        .iter()
        .map(|(time, value)| (*time, value.typed_bits()))
        .map(|(time, tb)| (time, try_path(&tb, path)));
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

fn bucketize(data: impl IntoIterator<Item = (u64, Option<TypedBits>)>) -> Box<[Bucket]> {
    let mut buckets = Vec::new();
    let mut last_time = !0;
    let mut last_data = None;
    let mut start_time = !0;
    for (time, data) in data.into_iter() {
        if last_time == !0 {
            last_time = time;
            start_time = time;
            last_data = data.clone();
        } else {
            if !last_data.eq(&data) {
                if let Some(data) = last_data {
                    buckets.push(Bucket {
                        start: start_time,
                        end: time,
                        data: data.clone(),
                    });
                }
                start_time = time;
                last_data = data.clone();
            }
            last_time = time;
        }
    }
    if last_time != !0 {
        if let Some(data) = last_data {
            buckets.push(Bucket {
                start: start_time,
                end: last_time,
                data,
            });
        }
    }
    buckets.into()
}

pub fn format_as_label(t: &TypedBits) -> Option<String> {
    (t.bits.len() != 1).then(|| format_as_label_inner(t))?
}

// Construct the leaf paths of the current object.  This version is a customized
// copy of [leaf_paths], which is meant to make the enumerated paths easier to
// understand for readability.
fn pretty_leaf_paths_inner(kind: &Kind, base: Path) -> Vec<Path> {
    // Special case base is a payload, and kind is a single-element tuple.  This happens
    // with enums, where the payload is a single-element tuple.  For readability, we
    // project through the payload to the tuple.
    if matches!(
        base.elements.iter().last(),
        Some(PathElement::EnumPayload(_))
    ) {
        match kind {
            Kind::Tuple(tuple) if tuple.elements.len() == 1 => {
                return pretty_leaf_paths_inner(&tuple.elements[0], base.clone().tuple_index(0));
            }
            _ => {}
        }
    }
    let mut root = vec![base.clone()];
    let branch = match kind {
        Kind::Array(array) => (0..array.size)
            .flat_map(|i| pretty_leaf_paths_inner(&array.base, base.clone().index(i)))
            .collect(),
        Kind::Tuple(tuple) => tuple
            .elements
            .iter()
            .enumerate()
            .flat_map(|(i, k)| pretty_leaf_paths_inner(k, base.clone().tuple_index(i)))
            .collect(),
        Kind::Struct(structure) => structure
            .fields
            .iter()
            .flat_map(|field| pretty_leaf_paths_inner(&field.kind, base.clone().field(&field.name)))
            .collect(),
        Kind::Signal(root, _) => pretty_leaf_paths_inner(root, base.clone().signal_value()),
        Kind::Enum(enumeration) => enumeration
            .variants
            .iter()
            .flat_map(|variant| {
                pretty_leaf_paths_inner(&variant.kind, base.clone().payload(&variant.name))
            })
            .collect(),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty => vec![],
    };
    root.extend(branch);
    root
}

pub fn pretty_leaf_paths(kind: &Kind, base: Path) -> Vec<Path> {
    // Remove all instances of #variant followed by #variant.0 - the
    // first does not add any value when pretty printing
    pretty_leaf_paths_inner(kind, base)
}

// Apply a path sequence to a TypedBits object, but use None instead of blindly
// assuming the path is valid.  There is only one case in which the path may yield
// a None, and that is when the path requests the EnumPayload but the payload does
// not match the discriminant of the enum.  All other cases, can be forwarded to
// the regular path method.
pub fn try_path(t: &TypedBits, path: &Path) -> Option<TypedBits> {
    let mut t = t.clone();
    for element in &path.elements {
        match element {
            PathElement::EnumPayload(tag) => {
                let discriminant = t.discriminant().ok()?.as_i64().ok()?;
                let tag_discriminant = t
                    .kind
                    .get_discriminant_for_variant_by_name(tag)
                    .ok()?
                    .as_i64()
                    .ok()?;
                if discriminant == tag_discriminant {
                    t = t.path(&Path::default().payload(tag)).ok()?;
                } else {
                    return None;
                }
            }
            PathElement::EnumPayloadByValue(tag_discriminant) => {
                let tag_discriminant = *tag_discriminant;
                let discriminant = t.discriminant().ok()?.as_i64().ok()?;
                if discriminant == tag_discriminant {
                    t = t
                        .path(&Path::default().payload_by_value(tag_discriminant))
                        .ok()?;
                } else {
                    return None;
                }
            }
            x => {
                t = t.path(&Path::with_element(x.clone())).ok()?;
            }
        }
    }
    Some(t)
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
    use crate::prelude::{b4, b8};

    use super::*;

    #[test]
    fn test_bucket_empty() {
        let data = [];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 0);
    }

    #[test]
    fn test_bucket_with_options() {
        let data = [
            (0, None),
            (5, Some(b4(3).typed_bits())),
            (10, None),
            (12, None),
            (15, Some(b4(3).typed_bits())),
            (20, None),
        ];
        let buckets = bucketize(data);
        assert_eq!(buckets.len(), 2);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 5,
                end: 10,
                data: b4(3).typed_bits()
            }
        );
        assert_eq!(
            buckets[1],
            Bucket {
                start: 15,
                end: 20,
                data: b4(3).typed_bits()
            }
        );
    }

    #[test]
    fn test_bucket_single() {
        let data = [(0, Some(b8(8).typed_bits()))];
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
        let data = [
            (0, Some(n8.clone())),
            (1, Some(n8.clone())),
            (2, Some(n8.clone())),
        ];
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
            (0, Some(b8(8).typed_bits())),
            (1, Some(b8(4).typed_bits())),
            (3, Some(b8(5).typed_bits())),
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
