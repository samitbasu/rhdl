//! Basic SVG rendering of traces
//!
//! This module provides functionality to render RHDL traces as SVG documents.
//! It defines structures and functions to convert trace data into SVG regions,
//! handle layout options, and generate the final SVG output.  It can be handy for
//! visualizing simulation results without reaching for a separate tool, and can
//! also be handy for generating docs that will embed with `rustdoc`, like this:
//!
#![doc = include_str!("demo.md")]
//!
//! To use the SVG renderer, you typically collect a simulation run into a [Vcd](crate::sim::vcd::Vcd)
//! container, and then call `vcd.dump_svg` with appropriate [SvgOptions](SvgOptions).
//! You can fine tune the rendering to select only certain time windows, and only certain
//! traces to be displayed.
//!
//! # Notes
//! We want to take a series of time/bool values and turn it into an SVG thing.
//! The underlying time series is a set of time/impl Digital values.  So the
//! translation needs to go from a type T to something representable.
//! Because we can only represent scalars in the SVG, we need to slice the
//! data type into it's composite low level parts.
//!
//! A trace consists of two parts.
//!   <label> : - a string that identifies the trace to the reader
//!   [data]  : - a set of data points
//!  A data point can either be a bool, or a vector (string is included as a vector).
//!  A bool region is defined by a start and end time and a value (either true or false).
//!  A data region is defined by a start and end time and a string describing the value.
use crate::{
    BitX, Color, Digital, Kind, TypedBits,
    types::path::{Path, PathElement, sub_kind},
};

#[derive(Clone, Debug)]
pub(crate) struct SvgRegion {
    start_x: i32,
    start_y: i32,
    width: i32,
    tag: String,
    full_tag: String,
    kind: RegionKind,
    color: TraceColor,
}

/// Options to control the SVG rendering
pub struct SvgOptions {
    /// Number of pixels per simulation time unit
    pub pixels_per_time_unit: f32,
    /// Font size in pixels
    pub font_size_in_pixels: f32,
    /// Vertical shim between traces
    pub shim: i32,
    /// Height of each trace region
    pub height: i32,
    /// Width of the label area
    pub label_width: i32,
    /// Minimum glitch filter duration
    pub glitch_filter: Option<u32>,
    /// Optional regex filter for trace names
    pub name_filters: Option<regex::Regex>,
}

impl SvgOptions {
    /// Calculate the vertical spacing between traces   
    fn spacing(&self) -> i32 {
        self.height + self.shim * 2
    }
    /// Set the label width
    pub fn with_label_width(self, width: i32) -> Self {
        Self {
            label_width: width,
            ..self
        }
    }
    /// Set a regex filter for trace names. Only names matching the regex will be included.
    pub fn with_filter(self, regex: &str) -> Self {
        Self {
            name_filters: Some(regex::Regex::new(regex).unwrap()),
            ..self
        }
    }
    /// Set a filter to include only top-level inputs and outputs, clock and reset
    pub fn with_io_filter(self) -> Self {
        Self {
            name_filters: Some(
                regex::Regex::new("(^top.input(.*))|(^top.outputs(.*))|(^top.reset)|(^top.clock)")
                    .unwrap(),
            ),
            ..self
        }
    }
}

impl Default for SvgOptions {
    fn default() -> Self {
        SvgOptions {
            pixels_per_time_unit: 1.0,
            font_size_in_pixels: 10.0,
            shim: 3,
            height: 14,
            label_width: 20,
            glitch_filter: Some(2),
            name_filters: None,
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
        .filter_map(|r| {
            if r.start > r.end {
                return None;
            }
            let len = r.end - r.start;
            if let Some(min_time) = options.glitch_filter
                && len < min_time as u64
            {
                return None;
            }
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
            Some(SvgRegion {
                start_x,
                start_y: 0,
                full_tag,
                width,
                tag,
                kind,
                color: r.color,
            })
        })
        .collect()
}

#[derive(Clone, Debug)]
struct Region {
    start: u64,
    end: u64,
    tag: Option<String>,
    kind: RegionKind,
    color: TraceColor,
}

#[derive(Copy, Clone, Debug)]
enum RegionKind {
    True,
    False,
    Multibit,
    Label,
}

#[derive(Debug)]
pub(crate) struct Trace {
    label: String,
    hint: String,
    data: Box<[Region]>,
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
        color: TraceColor::MultiColor,
    };
    let data_regions = regions_to_svg_regions(&trace.data, options).to_vec();
    std::iter::once(label_region)
        .chain(data_regions.into_iter().map(|mut x| {
            x.start_x += label_width;
            x
        }))
        .collect()
}

fn render_traces_to_svg(traces: &[Trace], options: &SvgOptions) -> Box<[SvgRegion]> {
    stack_svg_regions(
        &traces
            .iter()
            .filter(|t| {
                options
                    .name_filters
                    .as_ref()
                    .map(|f| f.is_match(&t.hint))
                    .unwrap_or(true)
            })
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

// The Clock to color map
fn stroke_color(color: TraceColor) -> &'static str {
    match color {
        TraceColor::Single(Color::Red) => "#D62246",
        TraceColor::Single(Color::Orange) => "#FF7F11",
        TraceColor::Single(Color::Yellow) => "#F7B32B",
        TraceColor::Single(Color::Green) => "#56C126",
        TraceColor::Single(Color::Blue) => "#5C95FF",
        TraceColor::Single(Color::Indigo) => "#9000B3",
        TraceColor::Single(Color::Violet) => "#672856",
        TraceColor::MultiColor => "#E7ECEF",
    }
}

fn fill_color(color: TraceColor) -> &'static str {
    match color {
        TraceColor::Single(Color::Red) => "#470B17",
        TraceColor::Single(Color::Orange) => "#552A05",
        TraceColor::Single(Color::Yellow) => "#523B0E",
        TraceColor::Single(Color::Green) => "#1C400C",
        TraceColor::Single(Color::Blue) => "#1E3155",
        TraceColor::Single(Color::Indigo) => "#30003B",
        TraceColor::Single(Color::Violet) => "#220D1C",
        TraceColor::MultiColor => "#4D4E4F",
    }
}

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
pub(crate) fn render_traces_as_svg_document(
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
    let grid_start = (start_time / time_delta)
        + if !start_time.is_multiple_of(time_delta) {
            1
        } else {
            0
        };
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
        let fill_color = fill_color(region.color);
        let stroke_color = stroke_color(region.color);
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
        let shim = shim.min(width / 2);
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
                        .set("fill", fill_color)
                        .set("stroke", "none"),
                );
                document = document.add(
                    svg::node::element::Path::new()
                        .set(
                            "d",
                            format!("M {x1} {y1} L {x2} {y2} L {x3} {y3} L {x4} {y4}"),
                        )
                        .set("fill", "none")
                        .set("stroke", stroke_color)
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
                        .set("stroke", stroke_color)
                        .set("stroke-width", 1),
                );
            }
            RegionKind::Multibit => {
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
                    .set("stroke", stroke_color)
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
        }
    }
    document
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum TraceColor {
    Single(Color),
    MultiColor,
}

impl Default for TraceColor {
    fn default() -> Self {
        TraceColor::Single(Color::Green)
    }
}

impl TraceColor {
    fn merge(self, other: TraceColor) -> TraceColor {
        match (self, other) {
            (TraceColor::Single(s), TraceColor::Single(o)) if s == o => TraceColor::Single(s),
            _ => TraceColor::MultiColor,
        }
    }
}

fn color_merged(list: impl Iterator<Item = TraceColor>) -> Option<TraceColor> {
    let mut ret = None;
    for color in list {
        ret = match (ret, color) {
            (None, x) => Some(x),
            (Some(c), d) => Some(c.merge(d)),
        }
    }
    ret
}

fn compute_trace_color(kind: Kind) -> Option<TraceColor> {
    match kind {
        Kind::Signal(_, color) => Some(TraceColor::Single(color)),
        Kind::Bits(_) | Kind::Signed(_) | Kind::Empty | Kind::Enum(_) => None,
        Kind::Struct(inner) => color_merged(
            inner
                .fields
                .iter()
                .flat_map(|x| compute_trace_color(x.kind)),
        ),
        Kind::Tuple(inner) => {
            color_merged(inner.elements.iter().flat_map(|x| compute_trace_color(*x)))
        }
        Kind::Array(inner) => compute_trace_color(*inner.base),
    }
}

pub(crate) fn trace_out<T: Digital>(
    label: &str,
    db: &[(u64, T)],
    time_set: std::ops::RangeInclusive<u64>,
) -> Box<[Trace]> {
    if T::BITS == 0 {
        return Default::default();
    }
    let kind = T::static_kind();
    pretty_leaf_paths(&kind, Path::default())
        .into_iter()
        .map(|path| {
            let data = build_time_trace(db, &path, time_set.clone());
            Trace {
                label: format!("{label}{path:?}"),
                hint: Default::default(),
                data,
            }
        })
        .collect()
}

fn build_time_trace<T: Digital>(
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
    let trace_color = compute_trace_color_from_path(T::static_kind(), path).unwrap_or_default();
    let sliced = data
        .iter()
        .map(|(time, value)| (*time, value.typed_bits()))
        .map(|(time, tb)| (time, try_path(&tb, path)));
    bucketize(sliced, time_set, trace_color)
}

fn map_bucket_to_region(bucket: &Bucket) -> Region {
    let kind = match bucket.data.len() {
        1 => match bucket.data.bits()[0] {
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
        color: bucket.color,
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Bucket {
    start: u64,
    end: u64,
    data: TypedBits,
    color: TraceColor,
}

fn bucketize(
    data: impl IntoIterator<Item = (u64, Option<TypedBits>)>,
    time_set: std::ops::RangeInclusive<u64>,
    color: TraceColor,
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
                if let Some(data) = last_data
                    && time_set.contains(&start_time)
                    && start_time != time
                {
                    buckets.push(Bucket {
                        start: start_time - min_time,
                        end: time - min_time,
                        data: data.clone(),
                        color,
                    });
                }
                start_time = time;
                last_data = data.clone();
            }
            last_time = time;
        }
    }
    if start_time != end_time
        && let Some(data) = last_data
    {
        buckets.push(Bucket {
            start: start_time - min_time,
            end: end_time - min_time,
            color,
            data,
        });
    }
    buckets.into()
}

fn format_as_label(t: &TypedBits) -> Option<String> {
    (t.len() != 1).then(|| format_as_label_inner(t))?
}

// Construct the leaf paths of the current object.  This version is a customized
// copy of [leaf_paths], which is meant to make the enumerated paths easier to
// understand for readability.
fn pretty_leaf_paths_inner(kind: &Kind, base: Path) -> Vec<Path> {
    // Special case base is a payload, and kind is a single-element tuple.  This happens
    // with enums, where the payload is a single-element tuple.  For readability, we
    // project through the payload to the tuple.
    if matches!(base.iter().last(), Some(PathElement::EnumPayload(_))) {
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

fn pretty_leaf_paths(kind: &Kind, base: Path) -> Vec<Path> {
    // Remove all instances of #variant followed by #variant.0 - the
    // first does not add any value when pretty printing
    pretty_leaf_paths_inner(kind, base)
}

// Compute the color of a path applied to a TypedBits.  It may be None if
// no colors are encountered.  Otherwise, it is the color "closest" to the
// path in question (closest ancestor).
fn compute_trace_color_from_path(t: Kind, path: &Path) -> Option<TraceColor> {
    let mut my_color = None;
    let mut path = path.clone();
    while my_color.is_none() {
        let my_kind = sub_kind(t, &path).ok()?;
        my_color = compute_trace_color(my_kind);
        if path.is_empty() {
            break;
        }
        path.pop();
    }
    my_color
}

// Apply a path sequence to a TypedBits object, but use None instead of blindly
// assuming the path is valid.  There is only one case in which the path may yield
// a None, and that is when the path requests the EnumPayload but the payload does
// not match the discriminant of the enum.  All other cases, can be forwarded to
// the regular path method.
fn try_path(t: &TypedBits, path: &Path) -> Option<TypedBits> {
    let mut t = t.clone();
    for element in path.iter() {
        match element {
            PathElement::EnumPayload(tag) => {
                let discriminant = t.discriminant().ok()?.as_i64().ok()?;
                let tag_discriminant = t
                    .kind()
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
                t = t.path(&Path::with_element(*x)).ok()?;
            }
        }
    }
    Some(t)
}

fn format_as_label_inner(t: &TypedBits) -> Option<String> {
    match t.kind() {
        Kind::Array(inner) => {
            let vals = (0..inner.size)
                .flat_map(|i| t.path(&Path::default().index(i)).ok())
                .flat_map(|element| format_as_label_inner(&element))
                .collect::<Vec<_>>()
                .join(", ");
            Some(format!("[{vals}]"))
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
            Some(format!("({vals})"))
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
            Some(format!("{{{vals}}}"))
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
                if t.bits()[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            let num_nibbles = inner / 4 + if inner % 4 == 0 { 0 } else { 1 };
            // Format the val as a hex number with the given
            // number of nibbles, with left padding of zeros
            Some(format!("{val:0num_nibbles$x}"))
        }
        Kind::Signed(inner) => {
            let mut val: i128 = 0;
            for ndx in 0..inner {
                if t.bits()[ndx] == BitX::One {
                    // TODO - handle other BitX values
                    val |= 1 << ndx;
                }
            }
            if val & (1 << (inner - 1)) != 0 {
                val |= !0 << inner;
            }
            Some(format!("{val}"))
        }
        Kind::Signal(_inner, color) => {
            let val = &t.val();
            let val = format_as_label_inner(val)?;
            Some(format!("{color:?}@({val})"))
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
            if label.starts_with(labels[ndx - 1 - i])
                && let Some(char) = label.chars().nth(break_char)
                && ['.', '#', '['].contains(&char)
            {
                parents[ndx] = ndx - 1 - i;
                break;
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
    if !indentation.is_empty() {
        indentation[0] = 1;
    }
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

    use rhdl_bits::{alias::*, bits, signed};

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
        let buckets = bucketize(data, 0..=20, Default::default());
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
        let buckets = bucketize(data, 0..=20, Default::default());
        assert_eq!(buckets.len(), 2);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 5,
                end: 10,
                data: b4(3).typed_bits(),
                color: TraceColor::default()
            }
        );
        assert_eq!(
            buckets[1],
            Bucket {
                start: 15,
                end: 20,
                data: b4(3).typed_bits(),
                color: TraceColor::default()
            }
        );
    }

    #[test]
    fn test_bucket_single() {
        let data = [(0, Some(b8(8).typed_bits()))];
        let buckets = bucketize(data, 0..=20, Default::default());
        assert_eq!(buckets.len(), 1);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 20,
                data: b8(8).typed_bits(),
                color: TraceColor::default()
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
        let buckets = bucketize(data, 0..=2, Default::default());
        assert_eq!(buckets.len(), 1);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 2,
                data: n8,
                color: TraceColor::default()
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
        let buckets = bucketize(data, 0..=4, Default::default());
        assert_eq!(buckets.len(), 3);
        assert_eq!(
            buckets[0],
            Bucket {
                start: 0,
                end: 1,
                data: b8(8).typed_bits(),
                color: TraceColor::default()
            }
        );
        assert_eq!(
            buckets[1],
            Bucket {
                start: 1,
                end: 3,
                data: b8(4).typed_bits(),
                color: TraceColor::default()
            }
        );
        assert_eq!(
            buckets[2],
            Bucket {
                start: 3,
                end: 4,
                data: b8(5).typed_bits(),
                color: TraceColor::default()
            }
        );
    }

    #[test]
    fn test_parent_map() {
        let sample_paths = &[
            "top.clock",                             // 0 0
            "top.drainer.drain_kernel.data",         // 0 1
            "top.drainer.drain_kernel.data_matches", // 0 2
            "top.drainer.drain_kernel.valid",        // 0 3
            "top.drainer.input",                     // 0 4
            "top.drainer.input.data",                // 4 5
            "top.drainer.input.data#None",           // 5 6
            "top.drainer.input.data#Some.0",         // 5 7
            "top.drainer.input.data#Some.0.foo",     // 7 8
            "top.fifo.input",                        // 0 9
            "top.fifo.input.data",                   // 9 10
            "top.fifo.input.data#None",              // 10 11
            "top.fifo.input.data#Some.0",            // 10 12
        ];
        let parent_map = build_parent_map(sample_paths);
        assert_eq!(*parent_map, [0, 0, 0, 0, 0, 4, 5, 5, 7, 0, 9, 10, 10]);
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

    #[test]
    fn test_compute_trace_color() {
        let k = Kind::make_bool();
        assert_eq!(compute_trace_color(k), None);
        let k = Kind::make_signal(Kind::make_bool(), Color::Indigo);
        assert_eq!(
            compute_trace_color(k),
            Some(TraceColor::Single(Color::Indigo))
        );
        let k = Kind::make_signal(Kind::make_bool(), Color::Blue);
        let y = Kind::make_signal(Kind::make_bool(), Color::Green);
        let k = Kind::make_tuple(vec![k, y].into());
        assert_eq!(compute_trace_color(k), Some(TraceColor::MultiColor));
        let k = Kind::make_signal(Kind::make_bool(), Color::Blue);
        let k = Kind::make_tuple(vec![k, k].into());
        assert_eq!(
            compute_trace_color(k),
            Some(TraceColor::Single(Color::Blue))
        );
    }
    #[test]
    fn test_label_for_tuple_struct() {
        #[derive(PartialEq, Clone, Copy)]
        pub struct TupleStruct(b6, b3);

        impl crate::Digital for TupleStruct {
            const BITS: usize = <b6 as crate::Digital>::BITS + <b3 as crate::Digital>::BITS;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_struct(
                    concat!(module_path!(), "::", stringify!(TupleStruct)),
                    [
                        crate::Kind::make_field(
                            stringify!(0),
                            <b6 as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(1),
                            <b3 as crate::Digital>::static_kind(),
                        ),
                    ]
                    .into(),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                [self.0.bin(), self.1.bin()].concat().into()
            }
            fn dont_care() -> Self {
                Self(
                    <b6 as crate::Digital>::dont_care(),
                    <b3 as crate::Digital>::dont_care(),
                )
            }
        }

        let tuple = TupleStruct(bits(13), bits(4));
        let label = format_as_label(&tuple.typed_bits()).unwrap();
        let expect = expect!["{0: 0d, 1: 4}"];
        expect.assert_eq(&label);
    }

    #[test]
    fn test_label_for_struct() {
        #[derive(PartialEq, Clone, Copy)]
        pub struct Simple {
            a: b4,
            b: (b4, b4),
            c: [b5; 3],
            d: bool,
        }

        impl crate::Digital for Simple {
            const BITS: usize = <b4 as crate::Digital>::BITS
                + <(b4, b4) as crate::Digital>::BITS
                + <[b5; 3] as crate::Digital>::BITS
                + <bool as crate::Digital>::BITS;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_struct(
                    concat!(module_path!(), "::", stringify!(Simple)),
                    [
                        crate::Kind::make_field(
                            stringify!(a),
                            <b4 as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(b),
                            <(b4, b4) as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(c),
                            <[b5; 3] as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(d),
                            <bool as crate::Digital>::static_kind(),
                        ),
                    ]
                    .into(),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                [self.a.bin(), self.b.bin(), self.c.bin(), self.d.bin()]
                    .concat()
                    .into()
            }
            fn dont_care() -> Self {
                Self {
                    a: <b4 as crate::Digital>::dont_care(),
                    b: <(b4, b4) as crate::Digital>::dont_care(),
                    c: <[b5; 3] as crate::Digital>::dont_care(),
                    d: <bool as crate::Digital>::dont_care(),
                }
            }
        }

        let simple = Simple {
            a: bits(6),
            b: (bits(8), bits(9)),
            c: [bits(10), bits(11), bits(12)],
            d: false,
        };

        let label = format_as_label(&simple.typed_bits()).unwrap();
        let expect = expect!["{a: 6, b: (8, 9), c: [0a, 0b, 0c], d: 0}"];
        expect.assert_eq(&label);
    }

    #[test]
    fn test_label_for_signed() {
        #[derive(PartialEq, Clone, Copy)]
        pub struct Signed {
            a: s8,
            b: b8,
        }

        impl crate::Digital for Signed {
            const BITS: usize = <s8 as crate::Digital>::BITS + <b8 as crate::Digital>::BITS;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_struct(
                    concat!(module_path!(), "::", stringify!(Signed)),
                    [
                        crate::Kind::make_field(
                            stringify!(a),
                            <s8 as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(b),
                            <b8 as crate::Digital>::static_kind(),
                        ),
                    ]
                    .into(),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                [self.a.bin(), self.b.bin()].concat().into()
            }
            fn dont_care() -> Self {
                Self {
                    a: <s8 as crate::Digital>::dont_care(),
                    b: <b8 as crate::Digital>::dont_care(),
                }
            }
        }

        let signed = Signed {
            a: signed(-42),
            b: bits(42),
        };
        let label = format_as_label(&signed.typed_bits()).unwrap();
        let expect = expect!["{a: -42, b: 2a}"];
        expect.assert_eq(&label);
    }

    #[test]
    fn test_label_for_enum() {
        #[derive(PartialEq, Default, Clone, Copy)]
        enum Value {
            #[default]
            Empty,
            A(b8, b16),
            B {
                name: b8,
            },
            C(bool),
        }

        impl crate::Digital for Value {
            const BITS: usize = 2usize + 24usize;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Value)),
                    [
                        crate::Kind::make_variant(stringify!(Empty), crate::Kind::Empty, 0i64),
                        crate::Kind::make_variant(
                            stringify!(A),
                            crate::Kind::make_tuple(
                                [
                                    <b8 as crate::Digital>::static_kind(),
                                    <b16 as crate::Digital>::static_kind(),
                                ]
                                .into(),
                            ),
                            1i64,
                        ),
                        crate::Kind::make_variant(
                            stringify!(B),
                            crate::Kind::make_struct(
                                stringify!(_Value__B),
                                [crate::Kind::make_field(
                                    stringify!(name),
                                    <b8 as crate::Digital>::static_kind(),
                                )]
                                .into(),
                            ),
                            2i64,
                        ),
                        crate::Kind::make_variant(
                            stringify!(C),
                            crate::Kind::make_tuple(
                                [<bool as crate::Digital>::static_kind()].into(),
                            ),
                            3i64,
                        ),
                    ]
                    .into(),
                    crate::Kind::make_discriminant_layout(
                        2usize,
                        crate::DiscriminantAlignment::Msb,
                        crate::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                let mut raw = match self {
                    Self::Empty => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(0i64 as u128).to_bools()).to_vec()
                    }
                    Self::A(field0, field1) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(1i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field0.bin());
                        v.extend(field1.bin());
                        v
                    }
                    Self::B { name } => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(2i64 as u128).to_bools())
                            .to_vec();
                        v.extend(name.bin());
                        v
                    }
                    Self::C(field) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(3i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field.bin());
                        v
                    }
                }
                .to_vec();
                raw.resize(Self::BITS, crate::BitX::Zero);
                (crate::move_nbits_to_msb(&raw, 2usize)).into()
            }
            fn discriminant(self) -> crate::TypedBits {
                match self {
                    Self::Empty => rhdl_bits::bits::<2>(0i64 as u128).typed_bits(),
                    Self::A(_, _) => rhdl_bits::bits::<2>(1i64 as u128).typed_bits(),
                    Self::B { name: _ } => rhdl_bits::bits::<2>(2i64 as u128).typed_bits(),
                    Self::C(_) => rhdl_bits::bits::<2>(3i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> crate::Kind {
                match self {
                    Self::Empty => crate::Kind::Empty,
                    Self::A(_, _) => crate::Kind::make_tuple(
                        [
                            <b8 as crate::Digital>::static_kind(),
                            <b16 as crate::Digital>::static_kind(),
                        ]
                        .into(),
                    ),
                    Self::B { name: _ } => crate::Kind::make_struct(
                        stringify!(_Value__B),
                        [crate::Kind::make_field(
                            stringify!(name),
                            <b8 as crate::Digital>::static_kind(),
                        )]
                        .into(),
                    ),
                    Self::C(_) => {
                        crate::Kind::make_tuple([<bool as crate::Digital>::static_kind()].into())
                    }
                }
            }
            fn dont_care() -> Self {
                <Self as Default>::default()
            }
        }
        let val_array = [
            Value::Empty,
            Value::A(bits(42), bits(1024)),
            Value::B { name: bits(67) },
            Value::C(true),
        ];

        let label = format_as_label(&val_array.typed_bits()).unwrap();
        let expect = expect!["[Empty, A(2a, 0400), B{name: 43}, C(1)]"];
        expect.assert_eq(&label);
    }

    mod value_enum {
        use super::*;
        #[derive(PartialEq, Default, Clone, Copy)]
        enum Value {
            #[default]
            Empty,
            A(Option<bool>),
            B,
        }

        impl crate::Digital for Value {
            const BITS: usize = 2usize + 2usize;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Value)),
                    [
                        crate::Kind::make_variant(stringify!(Empty), crate::Kind::Empty, 0i64),
                        crate::Kind::make_variant(
                            stringify!(A),
                            crate::Kind::make_tuple(
                                [<Option<bool> as crate::Digital>::static_kind()].into(),
                            ),
                            1i64,
                        ),
                        crate::Kind::make_variant(stringify!(B), crate::Kind::Empty, 2i64),
                    ]
                    .into(),
                    crate::Kind::make_discriminant_layout(
                        2usize,
                        crate::DiscriminantAlignment::Msb,
                        crate::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                let mut raw = match self {
                    Self::Empty => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(0i64 as u128).to_bools()).to_vec()
                    }
                    Self::A(field) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(1i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field.bin());
                        v
                    }
                    Self::B => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(2i64 as u128).to_bools()).to_vec()
                    }
                }
                .to_vec();
                raw.resize(Self::BITS, crate::BitX::Zero);
                (crate::move_nbits_to_msb(&raw, 2usize)).into()
            }
            fn discriminant(self) -> crate::TypedBits {
                match self {
                    Self::Empty => rhdl_bits::bits::<2>(0i64 as u128).typed_bits(),
                    Self::A(_) => rhdl_bits::bits::<2>(1i64 as u128).typed_bits(),
                    Self::B => rhdl_bits::bits::<2>(2i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> crate::Kind {
                match self {
                    Self::Empty => crate::Kind::Empty,
                    Self::A(_) => crate::Kind::make_tuple(
                        [<Option<bool> as crate::Digital>::static_kind()].into(),
                    ),
                    Self::B => crate::Kind::Empty,
                }
            }
            fn dont_care() -> Self {
                <Self as Default>::default()
            }
        }

        #[test]
        fn test_leaf_paths_for_slicing() {
            let expect = expect![[r#"
        [
            ,
            #Empty,
            #A.0,
            #A.0#None,
            #A.0#Some.0,
            #B,
        ]
    "#]];
            let actual = pretty_leaf_paths(&Value::static_kind(), Path::default());
            expect.assert_debug_eq(&actual);
        }

        #[test]
        fn test_time_slice_with_nested_enums() {
            let val_array = [
                Value::Empty.typed_bits(),
                Value::A(Some(true)).typed_bits(),
                Value::B.typed_bits(),
            ];

            let path = Path::default().payload("A").tuple_index(0).payload("Some");
            let mapped = val_array
                .iter()
                .map(|v| try_path(v, &path))
                .collect::<Vec<_>>();
            let expect = expect![[r#"
        [
            None,
            Some(
                (true),
            ),
            None,
        ]
    "#]];
            expect.assert_debug_eq(&mapped);
        }
    }

    #[test]
    fn test_time_slice_for_enum_with_discriminant() {
        #[derive(PartialEq, Default, Clone, Copy)]
        enum Value {
            #[default]
            Empty,
            A(bool),
            B,
        }

        impl crate::Digital for Value {
            const BITS: usize = 2usize + 1_usize;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Value)),
                    [
                        crate::Kind::make_variant(stringify!(Empty), crate::Kind::Empty, 0i64),
                        crate::Kind::make_variant(
                            stringify!(A),
                            crate::Kind::make_tuple(
                                [<bool as crate::Digital>::static_kind()].into(),
                            ),
                            1i64,
                        ),
                        crate::Kind::make_variant(stringify!(B), crate::Kind::Empty, 2i64),
                    ]
                    .into(),
                    crate::Kind::make_discriminant_layout(
                        2usize,
                        crate::DiscriminantAlignment::Msb,
                        crate::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                let mut raw = match self {
                    Self::Empty => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(0i64 as u128).to_bools()).to_vec()
                    }
                    Self::A(field) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(1i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field.bin());
                        v
                    }
                    Self::B => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(2i64 as u128).to_bools()).to_vec()
                    }
                }
                .to_vec();
                raw.resize(Self::BITS, crate::BitX::Zero);
                (crate::move_nbits_to_msb(&raw, 2usize)).into()
            }
            fn discriminant(self) -> crate::TypedBits {
                match self {
                    Self::Empty => rhdl_bits::bits::<2>(0i64 as u128).typed_bits(),
                    Self::A(_) => rhdl_bits::bits::<2>(1i64 as u128).typed_bits(),
                    Self::B => rhdl_bits::bits::<2>(2i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> crate::Kind {
                match self {
                    Self::Empty => crate::Kind::Empty,
                    Self::A(_) => {
                        crate::Kind::make_tuple([<bool as crate::Digital>::static_kind()].into())
                    }
                    Self::B => crate::Kind::Empty,
                }
            }
            fn dont_care() -> Self {
                <Self as Default>::default()
            }
        }

        let val_array = [
            Value::Empty.typed_bits(),
            Value::A(true).typed_bits(),
            Value::B.typed_bits(),
        ];

        let path = Path::default().payload("A");
        let mapped = val_array
            .iter()
            .map(|v| try_path(v, &path))
            .collect::<Vec<_>>();
        let expect = expect![[r#"
        [
            None,
            Some(
                (true),
            ),
            None,
        ]
    "#]];
        expect.assert_debug_eq(&mapped);
    }

    mod second_value {
        use super::*;

        #[derive(PartialEq, Default, Clone, Copy)]
        enum Value {
            #[default]
            Empty,
            A(b8, b16),
            B {
                name: b8,
            },
            C(bool),
        }

        impl crate::Digital for Value {
            const BITS: usize = 2usize + 24_usize;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_enum(
                    concat!(module_path!(), "::", stringify!(Value)),
                    [
                        crate::Kind::make_variant(stringify!(Empty), crate::Kind::Empty, 0i64),
                        crate::Kind::make_variant(
                            stringify!(A),
                            crate::Kind::make_tuple(
                                [
                                    <b8 as crate::Digital>::static_kind(),
                                    <b16 as crate::Digital>::static_kind(),
                                ]
                                .into(),
                            ),
                            1i64,
                        ),
                        crate::Kind::make_variant(
                            stringify!(B),
                            crate::Kind::make_struct(
                                stringify!(_Value__B),
                                [crate::Kind::make_field(
                                    stringify!(name),
                                    <b8 as crate::Digital>::static_kind(),
                                )]
                                .into(),
                            ),
                            2i64,
                        ),
                        crate::Kind::make_variant(
                            stringify!(C),
                            crate::Kind::make_tuple(
                                [<bool as crate::Digital>::static_kind()].into(),
                            ),
                            3i64,
                        ),
                    ]
                    .into(),
                    crate::Kind::make_discriminant_layout(
                        2usize,
                        crate::DiscriminantAlignment::Msb,
                        crate::DiscriminantType::Unsigned,
                    ),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                let mut raw = match self {
                    Self::Empty => {
                        crate::bitx_vec(&rhdl_bits::bits::<2>(0i64 as u128).to_bools()).to_vec()
                    }
                    Self::A(field0, field1) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(1i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field0.bin());
                        v.extend(field1.bin());
                        v
                    }
                    Self::B { name } => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(2i64 as u128).to_bools())
                            .to_vec();
                        v.extend(name.bin());
                        v
                    }
                    Self::C(field) => {
                        let mut v = crate::bitx_vec(&rhdl_bits::bits::<2>(3i64 as u128).to_bools())
                            .to_vec();
                        v.extend(field.bin());
                        v
                    }
                }
                .to_vec();
                raw.resize(Self::BITS, crate::BitX::Zero);
                (crate::move_nbits_to_msb(&raw, 2usize)).into()
            }
            fn discriminant(self) -> crate::TypedBits {
                match self {
                    Self::Empty => rhdl_bits::bits::<2>(0i64 as u128).typed_bits(),
                    Self::A(_, _) => rhdl_bits::bits::<2>(1i64 as u128).typed_bits(),
                    Self::B { name: _ } => rhdl_bits::bits::<2>(2i64 as u128).typed_bits(),
                    Self::C(_) => rhdl_bits::bits::<2>(3i64 as u128).typed_bits(),
                }
            }
            fn variant_kind(self) -> crate::Kind {
                match self {
                    Self::Empty => crate::Kind::Empty,
                    Self::A(_, _) => crate::Kind::make_tuple(
                        [
                            <b8 as crate::Digital>::static_kind(),
                            <b16 as crate::Digital>::static_kind(),
                        ]
                        .into(),
                    ),
                    Self::B { name: _ } => crate::Kind::make_struct(
                        stringify!(_Value__B),
                        [crate::Kind::make_field(
                            stringify!(name),
                            <b8 as crate::Digital>::static_kind(),
                        )]
                        .into(),
                    ),
                    Self::C(_) => {
                        crate::Kind::make_tuple([<bool as crate::Digital>::static_kind()].into())
                    }
                }
            }
            fn dont_care() -> Self {
                <Self as Default>::default()
            }
        }

        #[test]
        fn test_trace_out_for_enum() {
            let val_array = [
                (0, Value::Empty),
                (5, Value::A(bits(42), bits(1024))),
                (10, Value::B { name: bits(67) }),
                (15, Value::C(true)),
                (20, Value::C(true)),
            ];

            let traces = trace_out("val", &val_array, 0..=25);
            let options = SvgOptions::default();
            let _svg = render_traces_to_svg(&traces, &options);
            let svg = render_traces_as_svg_document(0, traces, &options);
            expect_test::expect_file!("expect/test_enum_svg.expect").assert_eq(&svg.to_string());
        }
        #[test]
        fn test_time_slice_for_enum() {
            let val_array = [
                (0, Value::Empty),
                (5, Value::A(bits(42), bits(1024))),
                (10, Value::B { name: bits(67) }),
                (15, Value::C(true)),
            ];

            let label = build_time_trace(&val_array, &Default::default(), 0..=20);
            let expect = expect_file!["expect/time_slice_for_enum.expect"];
            expect.assert_debug_eq(&label);
        }
    }

    #[test]
    fn test_time_slice_for_struct() {
        #[derive(PartialEq, Clone, Copy)]
        pub struct Simple {
            a: b4,
            b: (b4, b4, bool),
            c: [b5; 3],
        }

        // Recursive expansion of Digital macro
        // =====================================

        impl crate::Digital for Simple {
            const BITS: usize = <b4 as crate::Digital>::BITS
                + <(b4, b4, bool) as crate::Digital>::BITS
                + <[b5; 3] as crate::Digital>::BITS;
            fn static_kind() -> crate::Kind {
                crate::Kind::make_struct(
                    concat!(module_path!(), "::", stringify!(Simple)),
                    [
                        crate::Kind::make_field(
                            stringify!(a),
                            <b4 as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(b),
                            <(b4, b4, bool) as crate::Digital>::static_kind(),
                        ),
                        crate::Kind::make_field(
                            stringify!(c),
                            <[b5; 3] as crate::Digital>::static_kind(),
                        ),
                    ]
                    .into(),
                )
            }
            fn bin(self) -> Box<[crate::BitX]> {
                [self.a.bin(), self.b.bin(), self.c.bin()].concat().into()
            }
            fn dont_care() -> Self {
                Self {
                    a: <b4 as crate::Digital>::dont_care(),
                    b: <(b4, b4, bool) as crate::Digital>::dont_care(),
                    c: <[b5; 3] as crate::Digital>::dont_care(),
                }
            }
        }

        let bld = |a, b, c, d| Simple {
            a: bits(a),
            b: (bits(b), bits(b + 1), d),
            c: [bits(c), bits(c + 1), bits(c + 2)],
        };

        let data = [
            (0, bld(2, 4, 1, false)),
            (10, bld(2, 5, 3, true)),
            (15, bld(4, 5, 1, true)),
            (20, bld(3, 0, 2, false)),
        ];

        let path = Path::default().field("b").tuple_index(2);
        let time_trace = build_time_trace(&data, &path, 0..=25);
        let expect = expect_file!["expect/time_slice_for_struct.expect"];
        expect.assert_debug_eq(&time_trace);
        let path = Path::default().field("b");
        let time_trace = build_time_trace(&data, &path, 0..=25);
        let expect = expect_file!["expect/time_trace_for_struct_b.expect"];
        expect.assert_debug_eq(&time_trace);
        let time_trace = build_time_trace(&data, &Default::default(), 0..=25);
        let expect = expect_file!["expect/time_trace_for_struct_root.expect"];
        expect.assert_debug_eq(&time_trace);
    }
}
