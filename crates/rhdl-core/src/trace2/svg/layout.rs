use crate::{
    Color,
    trace2::svg::{
        color::TraceColor,
        options::SvgOptions,
        region::{RegionKind, SvgWaveform},
    },
};

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

pub(crate) fn make_svg_document(
    waveforms: &[SvgWaveform],
    times: &[u64],
    options: &SvgOptions,
) -> svg::Document {
    let time_delta = options.select_time_delta();
    let Some(start_time) = times.first().copied() else {
        // No times, return empty document
        return svg::Document::new();
    };
    let Some(end_time) = times.last().copied() else {
        // No times, return empty document
        return svg::Document::new();
    };
    let width = waveforms.iter().map(|w| w.width()).max().unwrap_or(0);
    let height = waveforms
        .iter()
        .map(|w| w.height(options.spacing()))
        .max()
        .unwrap_or(0);
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
    for waveform in waveforms {
        for region in &waveform.data {
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
    }
    document
}
