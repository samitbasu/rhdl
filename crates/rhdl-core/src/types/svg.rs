//! The SVG visualizer for RHDL kinds
//!
//! This module provides functionality to generate SVG representations of RHDL kinds.
//! The main function is `svg_grid`, which takes a [Kind](crate::types::kind::Kind)
//! and a name, and produces an SVG document that visually represents the structure of the kind.
use std::ops::Range;

use super::kind::{DiscriminantAlignment, Kind};

#[derive(Clone, Debug)]
enum LayoutLabel {
    Name(String),
    Bits(Range<usize>),
}

impl LayoutLabel {
    fn len(&self) -> usize {
        match self {
            LayoutLabel::Name(x) => x.len(),
            LayoutLabel::Bits(range) => format!("{}:{}", range.start, range.end).len(),
        }
    }
    fn as_lsb(&self) -> LayoutLabel {
        match self {
            LayoutLabel::Name(s) => LayoutLabel::Name(s.into()),
            LayoutLabel::Bits(r) => LayoutLabel::Bits(r.end..r.start),
        }
    }
}

impl std::fmt::Display for LayoutLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutLabel::Name(s) => write!(f, "{s}"),
            LayoutLabel::Bits(b) => {
                if b.end.abs_diff(b.start) == 1 {
                    write!(f, "{}", b.start.min(b.end))
                } else if b.start > b.end {
                    write!(f, "{}:{}", b.start - 1, b.end)
                } else {
                    write!(f, "{}:{}", b.start, b.end - 1)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct KindLayout {
    row: usize,
    size: usize,
    cols: Range<usize>,
    label: Option<LayoutLabel>,
    fill_color: Option<&'static str>,
    stroke_color: Option<&'static str>,
}

fn generate_kind_layout(
    kind: &Kind,
    name: &str,
    mut offset_row: usize,
    mut offset_col: usize,
) -> Vec<KindLayout> {
    match kind {
        Kind::Empty => vec![],
        Kind::Signal(kind, _) => generate_kind_layout(kind, name, offset_row, offset_col),
        Kind::Clock => {
            vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..offset_col + 1,
                label: Some(LayoutLabel::Name(format!("{name} clk"))),
                ..Default::default()
            }]
        }
        Kind::Reset => {
            vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..offset_col + 1,
                label: Some(LayoutLabel::Name(format!("{name} rst"))),
                ..Default::default()
            }]
        }
        Kind::Bits(digits) => {
            vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..offset_col + digits,
                label: Some(LayoutLabel::Name(format!("{name} b{digits}"))),
                ..Default::default()
            }]
        }
        Kind::Signed(digits) => {
            vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..offset_col + digits,
                label: Some(LayoutLabel::Name(format!("{name} s{digits}"))),
                ..Default::default()
            }]
        }
        Kind::Struct(s) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..(offset_col + kind.bits()),
                label: Some(LayoutLabel::Name(format!("{{{name}}}"))),
                ..Default::default()
            }];
            for field in &s.fields {
                result.extend(generate_kind_layout(
                    &field.kind,
                    &format!(".{}", field.name),
                    offset_row + 1,
                    offset_col,
                ));
                offset_col += field.kind.bits();
            }
            result
        }
        Kind::Tuple(t) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..(offset_col + kind.bits()),
                label: Some(LayoutLabel::Name(format!("({name})"))),
                ..Default::default()
            }];
            for (ndx, element) in t.elements.iter().enumerate() {
                let element_layout =
                    generate_kind_layout(element, &format!(".{ndx}"), offset_row + 1, offset_col);
                result.extend(element_layout);
                offset_col += element.bits();
            }
            result
        }
        Kind::Array(a) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                size: 1,
                cols: offset_col..(offset_col + kind.bits()),
                label: Some(LayoutLabel::Name(format!("{name}[{}]", a.size))),
                ..Default::default()
            }];
            for ndx in 0..a.size {
                result.extend(generate_kind_layout(
                    &a.base,
                    &format!("[{ndx}]"),
                    offset_row + 1,
                    offset_col,
                ));
                offset_col += a.base.bits();
            }
            result
        }
        Kind::Enum(e) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                cols: offset_col..(offset_col + kind.bits()),
                size: 1,
                label: Some(LayoutLabel::Name(format!("{name}|{}|", kind.bits()))),
                ..Default::default()
            }];
            let variant_cols = match e.discriminant_layout.alignment {
                DiscriminantAlignment::Lsb => {
                    offset_col..(offset_col + e.discriminant_layout.width)
                }
                DiscriminantAlignment::Msb => {
                    offset_col + kind.bits() - e.discriminant_layout.width
                        ..(offset_col + kind.bits())
                }
            };
            let payload_offset = match e.discriminant_layout.alignment {
                DiscriminantAlignment::Lsb => offset_col + e.discriminant_layout.width,
                DiscriminantAlignment::Msb => offset_col,
            };
            let disc_width = e.discriminant_layout.width;
            for variant in &e.variants {
                let discriminant = if variant.discriminant < 0 {
                    (variant.discriminant as u128) & ((1 << disc_width) - 1)
                } else {
                    variant.discriminant as u128
                };
                result.push(KindLayout {
                    row: offset_row + 1,
                    size: 1,
                    cols: variant_cols.clone(),
                    label: Some(LayoutLabel::Name(format!(
                        "{}({:0width$b})",
                        variant.name,
                        discriminant,
                        width = disc_width
                    ))),
                    ..Default::default()
                });
                let variant_layout = generate_kind_layout(
                    &variant.kind,
                    &variant.name,
                    offset_row + 1,
                    payload_offset,
                );
                let new_offset_row = variant_layout
                    .iter()
                    .map(|x| x.row)
                    .max()
                    .unwrap_or(offset_row + 1);
                result.last_mut().unwrap().size = new_offset_row - offset_row;
                offset_row = new_offset_row;
                result.extend(variant_layout);
            }
            result
        }
    }
}

// Calculate the number of characters per bit in the layout
fn get_chars_per_bit(layout: &[KindLayout]) -> usize {
    layout
        .iter()
        .flat_map(|x| x.label.as_ref().map(|t| (x.cols.clone(), t)))
        .map(|(cols, x)| x.len().div_ceil(cols.len()))
        .max()
        .unwrap_or(0)
}

fn color_layout(iter: impl Iterator<Item = KindLayout>) -> impl Iterator<Item = KindLayout> {
    let soft_palette_colors = [
        "#99FFCC", "#CCCC99", "#CCCCCC", "#CCCCFF", "#CCFF99", "#CCFFCC", "#CCFFFF", "#FFCC99",
        "#FFCCCC", "#FFCCFF", "#FFFF99", "#FFFFCC",
    ];
    iter.zip(soft_palette_colors.into_iter().cycle())
        .map(|(layout, color)| KindLayout {
            fill_color: Some(color),
            stroke_color: Some("gray"),
            ..layout
        })
}

fn make_lsb_kind(layout: &[KindLayout]) -> Vec<KindLayout> {
    let max_cols = layout.iter().map(|l| l.cols.end).max().unwrap_or(1);
    layout
        .iter()
        .cloned()
        .map(|l| KindLayout {
            cols: (max_cols - l.cols.end)..(max_cols - l.cols.start),
            label: l.label.map(|t| t.as_lsb()),
            ..l
        })
        .collect()
}

pub(crate) mod kind_svg {
    //! The SVG visualizer for RHDL kinds
    //! This module provides functionality to generate SVG representations of RHDL kinds.
    use std::{
        collections::{BTreeMap, BTreeSet},
        iter::once,
    };

    use svg::Document;

    // To render the kind into an SVG, we define a grid of cells
    // Each column in the grid is a bit
    // Each row in the grid is an "alternative".
    // Because we do not know a-priori how many rows a given kind
    // will have, we first traverse the Kind tree and build a
    // "layout" tree that captures the row and columns for each
    // entry.
    // Given this layout tree, we can then render it as required.
    use super::*;

    fn text_box(
        pos: (i32, i32, i32, i32),
        text: &str,
        fill_color: &str,
        stroke_color: &str,
        document: svg::Document,
    ) -> svg::Document {
        let (x, y, width, height) = pos;
        let text_x = x + width / 2;
        let text_y = y + height / 2;
        let text = svg::node::element::Text::new(text)
            //            .add(svg::node::Text::new(text))
            .set("x", text_x)
            .set("y", text_y)
            .set("font-family", "monospace")
            .set("font-size", "10px")
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle");
        let rect = svg::node::element::Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", width)
            .set("height", height)
            .set("fill", fill_color)
            .set("stroke", stroke_color);
        document.add(rect).add(text)
    }
    fn svg_grid_from_layout_precolored(layout: &[KindLayout]) -> svg::Document {
        let num_rows = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
        let num_cols = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
        let chars_per_bit = get_chars_per_bit(layout);
        let pixels_per_char = 16;
        let mut document = Document::new().set(
            "viewBox",
            (
                0,
                -(pixels_per_char as i32),
                num_cols * chars_per_bit * pixels_per_char,
                (num_rows + 2) * pixels_per_char,
            ),
        );
        // Provide a background rectangle for the diagram of light gray
        let background = svg::node::element::Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", num_cols * chars_per_bit * pixels_per_char)
            .set("height", num_rows * pixels_per_char)
            .set("fill", "#EEEEEE")
            .set("stroke", "darkblue");
        document = document.add(background);
        // For each cell, add a rectangle to the SVG with the
        // name of the cell centered in the rectangle
        for cell in layout {
            let x = cell.cols.start * chars_per_bit * pixels_per_char;
            let y = cell.row * pixels_per_char;
            let width = cell.cols.len() * chars_per_bit * pixels_per_char;
            let height = pixels_per_char * cell.size;
            if let (Some(fill_color), Some(stroke_color), Some(label)) =
                (cell.fill_color, cell.stroke_color, cell.label.as_ref())
            {
                document = text_box(
                    (x as i32, y as i32, width as i32, height as i32),
                    &label.to_string(),
                    fill_color,
                    stroke_color,
                    document,
                );
            }
        }
        document
    }

    pub(crate) fn svg_grid(kind: &Kind, name: &str) -> svg::Document {
        let mut layout = generate_kind_layout(kind, name, 0, 0);
        let max_rows = layout.iter().map(|x| x.row + x.size).max().unwrap_or(1);
        // Collect the bit breakpoints
        let bit_breaks: BTreeSet<usize> = layout
            .iter()
            .flat_map(|x| once(x.cols.start).chain(once(x.cols.end)))
            .collect();
        let reverse_hash: BTreeMap<usize, usize> = bit_breaks
            .iter()
            .enumerate()
            .map(|(ndx, &bit)| (bit, ndx))
            .collect();
        layout.iter_mut().for_each(|x| {
            x.row += 1;
            x.cols.start = reverse_hash[&x.cols.start];
            x.cols.end = reverse_hash[&x.cols.end];
        });
        let bit_bins: Vec<usize> = bit_breaks.iter().copied().collect();
        let bit_boxes = bit_bins.windows(2).enumerate().map(|(ndx, x)| {
            let end = x[1];
            let start = x[0];
            KindLayout {
                row: 0,
                size: 1,
                cols: ndx..(ndx + 1),
                label: Some(LayoutLabel::Bits(start..end)),
                fill_color: Some("#EEEEEE"),
                stroke_color: Some("darkblue"),
            }
        });
        let bottom_bit_boxes = bit_boxes.clone().map(|x| KindLayout {
            row: max_rows + 1,
            ..x
        });
        let layout = bit_boxes
            .chain(color_layout(layout.iter().cloned()))
            .chain(bottom_bit_boxes)
            .collect::<Vec<_>>();
        let layout = make_lsb_kind(&layout);
        svg_grid_from_layout_precolored(&layout)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        DiscriminantType, TypedBits,
        types::kind::{Field, Variant},
    };

    use super::*;

    fn make_enum_msb_signed_kind() -> Kind {
        Kind::make_enum(
            "Test",
            vec![
                Kind::make_variant("A", Kind::Empty, -1),
                Kind::make_variant("B", Kind::Bits(8), 1),
                Kind::make_variant(
                    "C",
                    Kind::make_tuple(vec![Kind::Bits(8), Kind::Bits(16)].into()),
                    2,
                ),
                Kind::make_variant(
                    "D",
                    Kind::make_struct(
                        "Test::D",
                        vec![
                            Kind::make_field("a", Kind::Bits(8)),
                            Kind::make_field("b", Kind::Bits(16)),
                        ]
                        .into(),
                    ),
                    -3,
                ),
            ],
            Kind::make_discriminant_layout(4, DiscriminantAlignment::Msb, DiscriminantType::Signed),
        )
    }

    fn make_enum_kind() -> Kind {
        Kind::make_enum(
            "Test",
            vec![
                Kind::make_variant("A", Kind::Empty, 0),
                Kind::make_variant("B", Kind::Bits(8), 1),
                Kind::make_variant(
                    "C",
                    Kind::make_tuple([Kind::Bits(8), Kind::Bits(16)].into()),
                    2,
                ),
                Kind::make_variant(
                    "D",
                    Kind::make_struct(
                        "Test::D",
                        [
                            Kind::make_field("a", Kind::Bits(8)),
                            Kind::make_field("b", Kind::Bits(16)),
                        ]
                        .into(),
                    ),
                    3,
                ),
            ],
            Kind::make_discriminant_layout(
                4,
                DiscriminantAlignment::Lsb,
                DiscriminantType::Unsigned,
            ),
        )
    }

    #[test]
    fn test_enum_template_is_correct() {
        let kind = make_enum_kind();
        let len = kind.bits();
        let template = kind.enum_template("B").unwrap();
        let disc: TypedBits = 1_u64.into();
        assert_eq!(template.bits(), disc.unsigned_cast(len).unwrap().bits());
        let template = kind.enum_template("C").unwrap();
        let disc: TypedBits = 2_u64.into();
        assert_eq!(template.bits(), disc.unsigned_cast(len).unwrap().bits());
        let template = kind.enum_template("D").unwrap();
        let disc: TypedBits = 3_u64.into();
        assert_eq!(template.bits(), disc.unsigned_cast(len).unwrap().bits());
    }

    #[test]
    fn test_enum_template_with_signed_msb_is_correct() {
        let kind = make_enum_msb_signed_kind();
        let template = kind.enum_template("A").unwrap();
        let disc: TypedBits = (-1_i64).into();
        let disc = disc.signed_cast(4).unwrap();
        let pad = kind.pad(disc.bits().to_vec());
        assert_eq!(template.bits(), pad.to_vec());
        let template = kind.enum_template("B").unwrap();
        let disc: TypedBits = 1_i64.into();
        let disc = disc.signed_cast(4).unwrap();
        let pad = kind.pad(disc.bits().to_vec());
        assert_eq!(template.bits(), pad.to_vec());
    }

    // Create a complex kind for testing that
    // has all allowed types.  It is equivalent to
    // an enum:
    // enum Crazy {
    //     A, // Enum with empty variant
    //     B(u8), // Enum with single variant
    //     C(u8, u16), // Enum with tuple variant
    //     D { a: u8, b: u16 }, // Enum with struct variant
    //     E([u8; 4]), // Enum with array variant
    //     F { a: u8, b: [u8; 4] }, // Enum with struct variant containing array
    //     G { a: u8, b: [u8; 4], c: u16 }, // Enum with struct variant containing array and other fields
    // }
    fn make_complex_kind() -> Kind {
        Kind::make_enum(
            "Crazy",
            vec![
                Variant {
                    name: "A".to_string().into(),
                    discriminant: 0,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "B".to_string().into(),
                    discriminant: 1,
                    kind: Kind::make_bits(8),
                },
                Variant {
                    name: "C".to_string().into(),
                    discriminant: 2,
                    kind: Kind::make_tuple([Kind::make_bits(8), Kind::make_bits(16)].into()),
                },
                Variant {
                    name: "D".to_string().into(),
                    discriminant: 3,
                    kind: Kind::make_struct(
                        "Crazy::D",
                        [
                            Field {
                                name: "a".to_string().into(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string().into(),
                                kind: Kind::make_bits(16),
                            },
                        ]
                        .into(),
                    ),
                },
                Variant {
                    name: "E".to_string().into(),
                    discriminant: 4,
                    kind: Kind::make_array(Kind::make_bits(8), 4),
                },
                Variant {
                    name: "F".to_string().into(),
                    discriminant: 5,
                    kind: Kind::make_struct(
                        "Crazy::F",
                        [
                            Field {
                                name: "a".to_string().into(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string().into(),
                                kind: Kind::make_array(Kind::make_bits(8), 4),
                            },
                        ]
                        .into(),
                    ),
                },
                Variant {
                    name: "G".to_string().into(),
                    discriminant: 6,
                    kind: Kind::make_struct(
                        "Crazy::G",
                        [
                            Field {
                                name: "a".to_string().into(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string().into(),
                                kind: Kind::make_array(Kind::make_bits(8), 4),
                            },
                            Field {
                                name: "c".to_string().into(),
                                kind: Kind::make_bits(16),
                            },
                        ]
                        .into(),
                    ),
                },
                Variant {
                    name: "H".to_string().into(),
                    discriminant: 8,
                    kind: Kind::make_enum(
                        "Crazy::H",
                        vec![
                            Variant {
                                name: "A".to_string().into(),
                                discriminant: 0,
                                kind: Kind::Empty,
                            },
                            Variant {
                                name: "B".to_string().into(),
                                discriminant: 1,
                                kind: Kind::Bits(4),
                            },
                            Variant {
                                name: "C".to_string().into(),
                                discriminant: 2,
                                kind: Kind::Empty,
                            },
                            Variant {
                                name: "Unknown".to_string().into(),
                                discriminant: 3,
                                kind: Kind::Empty,
                            },
                        ],
                        Kind::make_discriminant_layout(
                            2,
                            DiscriminantAlignment::Msb,
                            DiscriminantType::Unsigned,
                        ),
                    ),
                },
            ],
            Kind::make_discriminant_layout(
                4,
                DiscriminantAlignment::Lsb,
                DiscriminantType::Unsigned,
            ),
        )
    }

    #[test]
    fn test_layout_of_complex_kind() {
        let kind = make_complex_kind();
        let svg = kind_svg::svg_grid(&kind, "value");
        expect_test::expect_file!("expect/test_complex_kind_svg.expect")
            .assert_eq(&svg.to_string());
    }
    #[test]
    fn test_layout_of_struct() {
        let kind = Kind::make_struct(
            "Foo",
            [
                Field {
                    name: "a".to_string().into(),
                    kind: Kind::make_bits(8),
                },
                Field {
                    name: "b".to_string().into(),
                    kind: Kind::make_bits(16),
                },
                Field {
                    name: "c".to_string().into(),
                    kind: Kind::make_bits(32),
                },
            ]
            .into(),
        );
        let svg = kind_svg::svg_grid(&kind, "value");
        expect_test::expect_file!("expect/test_struct_svg.expect").assert_eq(&svg.to_string());
    }
    #[test]
    fn test_layout_of_struct_with_nesting() {
        let kind = Kind::make_struct(
            "Foo",
            [
                Field {
                    name: "a".to_string().into(),
                    kind: Kind::make_bits(8),
                },
                Field {
                    name: "b".to_string().into(),
                    kind: Kind::make_bits(16),
                },
                Field {
                    name: "c".to_string().into(),
                    kind: Kind::make_struct(
                        "Foo:c",
                        [
                            Field {
                                name: "d".to_string().into(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "e".to_string().into(),
                                kind: Kind::make_bits(16),
                            },
                        ]
                        .into(),
                    ),
                },
            ]
            .into(),
        );
        let svg = kind_svg::svg_grid(&kind, "value");
        expect_test::expect_file!("expect/test_struct_with_nesting_svg.expect")
            .assert_eq(&svg.to_string());
    }

    #[test]
    fn test_layout_of_simple_enum() {
        let kind = Kind::make_enum(
            "Simple",
            vec![
                Variant {
                    name: "A".to_string().into(),
                    discriminant: 0,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "B".to_string().into(),
                    discriminant: 1,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "C".to_string().into(),
                    discriminant: 2,
                    kind: Kind::Empty,
                },
            ],
            Kind::make_discriminant_layout(
                2,
                DiscriminantAlignment::Lsb,
                DiscriminantType::Unsigned,
            ),
        );
        let svg = kind_svg::svg_grid(&kind, "value");
        expect_test::expect_file!("expect/test_simple_enum_svg.expect").assert_eq(&svg.to_string());
    }

    #[test]
    fn test_result_recognized() {
        use crate::Digital;
        use rhdl_bits::alias::*;
        let a = std::result::Result::<b8, b8>::Ok(b8(42)).typed_bits();
        assert!(a.kind().is_result());
        let b = std::result::Result::<b4, ()>::Err(()).typed_bits();
        assert!(b.kind().is_result());
    }
}
