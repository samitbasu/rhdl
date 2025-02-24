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
    pub shim: i32,
    pub height: i32,
    pub label_width: i32,
}

impl SvgOptions {
    pub fn spacing(&self) -> i32 {
        self.height + self.shim * 2
    }
}

impl Default for SvgOptions {
    fn default() -> Self {
        SvgOptions {
            pixels_per_time_unit: 10.0,
            font_size_in_pixels: 10.0,
            shim: 3,
            height: 14,
            label_width: 40,
        }
    }
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
        start_y += options.spacing();
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
            SvgRegion {
                start_x,
                start_y: 0,
                full_tag,
                width,
                tag,
                kind,
            }
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
    pub hint: String,
    pub data: Box<[Region]>,
}

fn render_trace_to_svg(trace: &Trace, options: &SvgOptions) -> Box<[SvgRegion]> {
    let label_width = options.font_size_in_pixels as i32 * options.label_width;
    let label_region = SvgRegion {
        start_x: 0,
        start_y: 0,
        width: label_width,
        tag: trace.label.clone(),
        full_tag: trace.hint.clone(),
        kind: RegionKind::Label,
    };
    let data_regions = regions_to_svg_regions(&trace.data, options).to_vec();
    std::iter::once(label_region)
        .chain(data_regions.into_iter().map(|mut x| {
            x.start_x += label_width;
            x
        }))
        .collect()
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

fn rewrite_trace_names_into_tree(mut traces: Box<[Trace]>) -> Box<[Trace]> {
    let labels = traces.iter().map(|t| t.label.as_str()).collect::<Vec<_>>();
    let tree: Vec<IndentedLabel> = tree_view("top", &labels).into();
    traces.iter_mut().zip(tree).for_each(|(trace, label)| {
        trace.label = label.compute_label();
        trace.hint = label.full_text;
    });
    traces
}

const GREEN: &str = "#56C126";

// Generate an iterator that yields 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, etc.
//    This works by decomposing the number into
//  n       m    n / 3   n % 3
//  0       1      0       0
//  1       2      0       1
//  2       5      0       2
//  3      10      1       0
//  4      20      1       1
//  5      50      1       2
fn candidate_deltas() -> impl Iterator<Item = u64> {
    const SEQ: &[u64] = &[1, 2, 5];
    (0..).map(|n| {
        let m = n / 3;
        let n = n % 3;
        10u64.pow(m) * SEQ[n as usize]
    })
}

fn select_time_delta(options: &SvgOptions) -> u64 {
    // Need 10 characters * options.font_size_in_pixels < pixels_per_time_unit * time_delta
    candidate_deltas()
        .find(|x| (*x as f32 * options.pixels_per_time_unit) >= 10.0 * options.font_size_in_pixels)
        .unwrap()
}

// TODO - remove the duplication
pub fn render_traces_as_svg_document(
    start_time: u64,
    traces: Box<[Trace]>,
    options: &SvgOptions,
) -> svg::Document {
    let time_delta = select_time_delta(options);
    let traces = rewrite_trace_names_into_tree(traces);
    let mut regions = render_traces_to_svg(&traces, options);
    // Shift the traces down so we can fit the timeline in at the top
    regions
        .iter_mut()
        .for_each(|r| r.start_y += options.spacing());
    let width = regions
        .iter()
        .map(|r| r.start_x + r.width)
        .max()
        .unwrap_or_default();
    let height = regions
        .iter()
        .map(|r| r.start_y + options.spacing())
        .max()
        .unwrap_or_default();
    let mut document = svg::Document::new().set("viewBox", (0, 0, width, height));
    document = document.add(
        svg::node::element::Definitions::new().add(
            svg::node::element::ClipPath::new().set("id", "clip").add(
                svg::node::element::Rectangle::new()
                    .set("x", 0)
                    .set("y", 0)
                    .set("width", width)
                    .set("height", height),
            ),
        ),
    );
    // Provide a background rectangle for the diagram of light gray
    let background = svg::node::element::Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", width)
        .set("height", height)
        .set("fill", "#0B151D")
        .set("stroke", "darkblue");
    document = document.add(background);

    // Add a set of time labels
    // The start time may not lie on the grid.  E.g., we may start at time 77, but the grid is 50
    // We will start at the first grid point after the start time (or equal if it lies on a grid point)
    let grid_start = (start_time / time_delta) + if start_time % time_delta != 0 { 1 } else { 0 };
    let mut ndx = grid_start;
    let label_end = options.label_width as f32 * options.font_size_in_pixels;
    while (ndx * time_delta - start_time) as f32 * options.pixels_per_time_unit
        <= width as f32 - label_end
    {
        let x = (ndx * time_delta - start_time) as f32 * options.pixels_per_time_unit + label_end;
        let text = svg::node::element::Text::new(format!("{}", ndx * time_delta));
        document = document.add(
            svg::node::element::Line::new()
                .set("x1", x)
                .set("y1", 0)
                .set("x2", x)
                .set("y2", height)
                .set("stroke", "#333333")
                .set("stroke-width", 1.0),
        );
        document = document.add(
            text.set("x", x)
                .set("y", options.spacing() / 2)
                .set("font-family", "monospace")
                .set("font-size", "10px")
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle")
                .set("fill", "#D4D4D4")
                .set("clip-path", "url(#clip)"),
        );
        ndx += 1;
    }

    document = document.add(
        svg::node::element::Text::new("Time:")
            .set("x", options.shim)
            .set("y", options.spacing() / 2)
            .set("font-family", "monospace")
            .set("font-size", "10px")
            .set("text-anchor", "start")
            .set("dominant-baseline", "middle")
            .set("fill", "#D4D4D4"),
    );

    // For each cell, add a rectangle to the SVG with the
    // name of the cell centered in the rectangle
    for region in regions {
        let x = region.start_x;
        let y = region.start_y;
        let width = region.width;
        let height = options.spacing();
        let shim = options.shim;
        let fill_color = match region.kind {
            RegionKind::True => "green",
            RegionKind::False => "red",
            RegionKind::Multibit => "blue",
            RegionKind::Label => "white",
        };
        let stroke_color = "black";
        let text = region.tag.clone();
        let tip = region.full_tag.clone();
        let text_x = if matches!(region.kind, RegionKind::Label) {
            x + shim
        } else {
            x + width / 2
        };
        let text_y = y + height / 2;
        let text = svg::node::element::Text::new(text)
            .set("x", text_x)
            .set("y", text_y)
            .set("xml:space", "preserve")
            .set("font-family", "monospace")
            .set("font-size", "10px")
            .set("fill", "#D4D4D4");
        let text = if matches!(region.kind, RegionKind::Label) {
            text.set("text-anchor", "start")
        } else {
            text.set("text-anchor", "middle")
        };
        let text = text.set("dominant-baseline", "middle");
        match region.kind {
            RegionKind::True => {
                let x1 = x;
                let y1 = y + height / 2;
                let x2 = x;
                let y2 = y + shim;
                let x3 = x + width;
                let y3 = y + shim;
                let x4 = x + width;
                let y4 = y + height / 2;
                document = document.add(
                    svg::node::element::Rectangle::new()
                        .set("x", x + 1)
                        .set("y", y + shim)
                        .set("width", width - 2)
                        .set("height", height - shim * 2)
                        .set("fill", "#1a381f")
                        .set("stroke", "none"),
                );
                document = document.add(
                    svg::node::element::Path::new()
                        .set(
                            "d",
                            format!("M {x1} {y1} L {x2} {y2} L {x3} {y3} L {x4} {y4}"),
                        )
                        .set("fill", "none")
                        .set("stroke", GREEN)
                        .set("stroke-width", 1),
                );
            }
            RegionKind::False => {
                let x1 = x;
                let y1 = y + height / 2;
                let x2 = x;
                let y2 = y + height - shim;
                let x3 = x + width;
                let y3 = y + height - shim;
                let x4 = x + width;
                let y4 = y + height / 2;
                document = document.add(
                    svg::node::element::Path::new()
                        .set(
                            "d",
                            format!("M {x1} {y1} L {x2} {y2} L {x3} {y3} L {x4} {y4}"),
                        )
                        .set("fill", "none")
                        .set("stroke", GREEN)
                        .set("stroke-width", 1),
                );
            }
            RegionKind::Multibit => {
                let shim = shim.min(width / 2);
                let x1 = x;
                let y1 = y + height / 2;
                let x2 = x + shim;
                let y2 = y + shim;
                let x3 = x + width - shim;
                let y3 = y + shim;
                let x4 = x + width;
                let y4 = y + height / 2;
                let x5 = x + width - shim;
                let y5 = y + height - shim;
                let x6 = x + shim;
                let y6 = y + height - shim;
                document = document.add(
                    svg::node::element::Path::new()
                    .set("d", format!("M {x1} {y1} L {x2} {y2} L {x3} {y3} L {x4} {y4} L {x5} {y5} L {x6} {y6} Z"))
                    .set("fill", "none")
                    .set("stroke", GREEN)
                    .set("stroke-width", 1)
                );
                let title = svg::node::element::Title::new(tip);
                let text = text.add(title);
                document = document.add(text);
            }
            RegionKind::Label => {
                let title = svg::node::element::Title::new(tip);
                let text = text.add(title);
                document = document.add(text);
            }
            _ => {}
        }
        /*         let rect = svg::node::element::Rectangle::new()
                   .set("x", x)
                   .set("y", y)
                   .set("width", width)
                   .set("height", height)
                   .set("fill", fill_color)
                   .set("stroke", stroke_color);
               let title = svg::node::element::Title::new(tip);
               let rect = rect.add(title);
               document = document.add(rect).add(text);
        */
    }
    document
}

pub fn trace_out<T: Digital>(
    label: &str,
    db: &[(u64, T)],
    time_set: std::ops::RangeInclusive<u64>,
) -> Box<[Trace]> {
    let kind = T::static_kind();
    pretty_leaf_paths(&kind, Path::default())
        .into_iter()
        .map(|path| {
            let data = build_time_trace(db, &path, time_set.clone());
            Trace {
                label: format!("{label}{:?}", path),
                hint: Default::default(),
                data,
            }
        })
        .collect()
}

pub fn build_time_trace<T: Digital>(
    data: &[(u64, T)],
    path: &Path,
    time_set: std::ops::RangeInclusive<u64>,
) -> Box<[Region]> {
    slice_by_path_and_bucketize(data, path, time_set)
        .iter()
        .map(map_bucket_to_region)
        .collect()
}

fn slice_by_path_and_bucketize<T: Digital>(
    data: &[(u64, T)],
    path: &Path,
    time_set: std::ops::RangeInclusive<u64>,
) -> Box<[Bucket]> {
    let sliced = data
        .iter()
        .map(|(time, value)| (*time, value.typed_bits()))
        .map(|(time, tb)| (time, try_path(&tb, path)));
    bucketize(sliced, time_set)
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

fn bucketize(
    data: impl IntoIterator<Item = (u64, Option<TypedBits>)>,
    time_set: std::ops::RangeInclusive<u64>,
) -> Box<[Bucket]> {
    let mut buckets = Vec::new();
    let mut last_time = !0;
    let mut last_data = None;
    let mut start_time = !0;
    let min_time = *time_set.start();
    let end_time = *time_set.end();
    for (time, data) in data.into_iter() {
        if last_time == !0 {
            last_time = time;
            start_time = time;
            last_data = data.clone();
        } else {
            if !last_data.eq(&data) {
                if let Some(data) = last_data {
                    if time_set.contains(&start_time) && start_time != time {
                        buckets.push(Bucket {
                            start: start_time - min_time,
                            end: time - min_time,
                            data: data.clone(),
                        });
                    }
                }
                start_time = time;
                last_data = data.clone();
            }
            last_time = time;
        }
    }
    if start_time != end_time {
        if let Some(data) = last_data {
            buckets.push(Bucket {
                start: start_time - min_time,
                end: end_time - min_time,
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
                .flat_map(|element| format_as_label_inner(&element))
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
                .flat_map(|element| format_as_label_inner(&element))
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
                .flat_map(|(name, field)| format_as_label_inner(&field).map(|x| (name, x)))
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
            let payload = format_as_label_inner(&payload).unwrap_or_default();
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
            let val = format_as_label_inner(val)?;
            Some(format!("{:?}@({})", color, val))
        }
        Kind::Empty => None,
    }
}

#[derive(Debug, Clone)]
struct IndentedLabel {
    text: String,
    indent: usize,
    full_text: String,
}

impl IndentedLabel {
    fn compute_label(&self) -> String {
        (0..self.indent.saturating_sub(1))
            .map(|_| "   ")
            .chain(std::iter::once(self.text.as_str()))
            .collect()
    }
}

// Compute a map indicating which time series in the list
// is a "parent" of this one.  The parent is defined as
// a time series with a path that is a prefix of the current
// path.  We search backwards to find the closest anscestor.
fn build_parent_map(labels: &[&str]) -> Box<[usize]> {
    // Every node starts out parented to the root
    let mut parents = vec![0; labels.len()];
    // Work down the list
    for (ndx, label) in labels.iter().enumerate() {
        for i in 0..ndx {
            let break_char = labels[ndx - 1 - i].len();
            if label.starts_with(labels[ndx - 1 - i]) {
                if let Some(char) = label.chars().nth(break_char) {
                    if ['.', '#', '['].contains(&char) {
                        parents[ndx] = ndx - 1 - i;
                        break;
                    }
                }
            }
        }
    }
    parents.into()
}

// Given a list of parents, compute the indentation for each
// label.  The indentation is defined as the number of
// ancestors of the label.  Because we scan the list forward,
// we can keep track of the indentation level for a parent
// and increment it for each child.
fn compute_indentation(parents: &[usize]) -> Box<[usize]> {
    let mut indentation = vec![0; parents.len()];
    for ndx in 1..parents.len() {
        indentation[ndx] = indentation[parents[ndx]] + 1;
    }
    // Fix up the first entry
    indentation[0] = 1;
    indentation.into()
}

fn tree_view(root: &str, labels: &[&str]) -> Box<[IndentedLabel]> {
    let parent_map = build_parent_map(labels);
    let indentation = compute_indentation(&parent_map);
    labels
        .iter()
        .enumerate()
        .map(|(ndx, label)| {
            let mut text = label.to_string();
            let parent_text = if parent_map[ndx] == 0 {
                root
            } else {
                labels[parent_map[ndx]]
            };
            if text.starts_with(parent_text) {
                text = text.replacen(parent_text, "", 1);
            }
            IndentedLabel {
                text,
                indent: indentation[ndx],
                full_text: label.to_string(),
            }
        })
        .collect()
}

// The time bar can have steps of
// [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000], etc.
// To determine the step, we calculate the

#[cfg(test)]
mod tests {
    use expect_test::{expect, expect_file};

    use crate::prelude::{b4, b8};

    use super::*;

    #[test]
    fn test_format() {
        let label = "foo.bar.baz";
        let x = format!("{:>width$}", ">", width = label.len());
        assert_eq!(x, "          >");
    }

    #[test]
    fn test_bucket_empty() {
        let data = [];
        let buckets = bucketize(data, 0..=20);
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
        let buckets = bucketize(data, 0..=20);
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
        let buckets = bucketize(data, 0..=20);
        assert_eq!(buckets.len(), 1);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 20,
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
        let buckets = bucketize(data, 0..=2);
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
        let buckets = bucketize(data, 0..=4);
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
                end: 4,
                data: b8(5).typed_bits()
            }
        );
    }

    #[test]
    fn test_parent_map() {
        let sample_paths = &[
            "top.clock",                             // 0
            "top.drainer.drain_kernel.data",         // 0
            "top.drainer.drain_kernel.data_matches", // 0
            "top.drainer.drain_kernel.valid",        // 0
            "top.drainer.input",                     // 0
            "top.drainer.input.data",                // 3
            "top.drainer.input.data#None",           // 4
            "top.drainer.input.data#Some.0",         // 4
            "top.drainer.input.data#Some.0.foo",     // 6
            "top.fifo.input",                        // 0
            "top.fifo.input.data",                   // 8
            "top.fifo.input.data#None",              // 9
            "top.fifo.input.data#Some.0",            // 9
        ];
        let parent_map = build_parent_map(sample_paths);
        assert_eq!(*parent_map, [0, 0, 0, 0, 0, 3, 4, 4, 6, 0, 8, 9, 9]);
    }

    #[test]
    fn test_indentation_calculation() {
        let parent_map = [0, 0, 0, 0, 3, 4, 4, 6, 0, 8, 9, 9];
        let indentation = compute_indentation(&parent_map);
        assert_eq!(*indentation, [1, 1, 1, 1, 2, 3, 3, 4, 1, 2, 3, 3]);
    }

    #[test]
    fn test_candidate_deltas() {
        let candidates = candidate_deltas().take(10).collect::<Vec<_>>();
        assert_eq!(candidates, [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000]);
    }

    #[test]
    fn test_tree_view() {
        let sample_paths = &[
            "top.clock",                         // 0
            "top.drainer.drain_kernel.data",     // 0
            "top.drainer.drain_kernel.valid",    // 0
            "top.drainer.input",                 // 0
            "top.drainer.input.data",            // 3
            "top.drainer.input.data#None",       // 4
            "top.drainer.input.data#Some.0",     // 4
            "top.drainer.input.data#Some.0.foo", // 6
            "top.fifo.input",                    // 0
            "top.fifo.input.data",               // 8
            "top.fifo.input.data#None",          // 9
            "top.fifo.input.data#Some.0",        // 9
        ];
        let tree = tree_view("top", sample_paths);
        let expect = expect_file!["test_tree_view.expect"];
        expect.assert_debug_eq(&tree);
    }
}
