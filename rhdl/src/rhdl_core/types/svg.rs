use std::{iter::repeat, ops::Range};

use super::kind::{DiscriminantAlignment, Kind};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvgOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SvgOptions {
    pub orientation: SvgOrientation,
}

#[derive(Clone, Debug)]
struct KindLayout {
    row: usize,
    depth: usize,
    cols: Range<usize>,
    name: String,
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
        Kind::Bits(digits) => {
            vec![KindLayout {
                row: offset_row,
                depth: 1,
                cols: offset_col..offset_col + digits,
                name: format!("{name} b{digits}"),
            }]
        }
        Kind::Signed(digits) => {
            vec![KindLayout {
                row: offset_row,
                depth: 1,
                cols: offset_col..offset_col + digits,
                name: format!("{name} s{digits}"),
            }]
        }
        Kind::Struct(s) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                depth: 1,
                cols: offset_col..(offset_col + kind.bits()),
                name: format!("{{{name}}}"),
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
                depth: 1,
                cols: offset_col..(offset_col + kind.bits()),
                name: format!("({name})"),
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
                depth: 1,
                cols: offset_col..(offset_col + kind.bits()),
                name: format!("{name}[{}]", a.size),
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
                depth: 1,
                name: format!("{name}|{}|", kind.bits()),
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
                    depth: 1,
                    cols: variant_cols.clone(),
                    name: format!(
                        "{}({:0width$b})",
                        variant.name,
                        discriminant,
                        width = disc_width
                    ),
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
                result.last_mut().unwrap().depth = new_offset_row - offset_row;
                offset_row = new_offset_row;
                result.extend(variant_layout);
            }
            result
        }
    }
}

// Validate a layout
fn is_layout_valid(layout: &[KindLayout]) -> bool {
    // Get the range of rows and colums
    // For each row, check that the columns do not overlap
    // For each column, check that the rows do not overlap
    let num_cols = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
    let num_rows = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
    let mut grid = vec![vec![false; num_cols]; num_rows];
    for entry in layout {
        if grid[entry.row][entry.cols.start..entry.cols.end]
            .iter()
            .cloned()
            .any(|x| x)
        {
            println!("Overlap: {:#?}", entry);
            return false;
        }
        grid[entry.row][entry.cols.start..entry.cols.end]
            .iter_mut()
            .for_each(|x| *x = true);
    }
    // Dump the grid to the console
    for row in grid {
        for col in row {
            print!("{}", if col { "X" } else { "." });
        }
        println!();
    }
    true
}

// Calculate the number of characters per bit in the layout
fn get_chars_per_bit(layout: &[KindLayout]) -> usize {
    layout
        .iter()
        .map(|x| x.name.len().div_ceil(x.cols.len()))
        .max()
        .unwrap_or(0)
}

// Generate a string (text) representation of the layout
pub fn text_grid(kind: &Kind, name: &str) -> String {
    let layout = generate_kind_layout(kind, name, 0, 0);
    assert!(is_layout_valid(&layout));
    let chars_per_bit = get_chars_per_bit(&layout);
    let mut result = String::new();
    let num_rows = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
    for row in 0..num_rows {
        let row_layout = layout.iter().filter(|x| x.row == row);
        let mut col_cursor = 0;
        for entry in row_layout {
            if entry.cols.start > col_cursor {
                result.extend(repeat('.').take((entry.cols.start - col_cursor) * chars_per_bit));
            }
            result.extend(
                entry
                    .name
                    .chars()
                    .chain(repeat('+'))
                    .take(entry.cols.len() * chars_per_bit),
            );
            col_cursor = entry.cols.end;
        }
        result.push('\n');
    }
    result
}

#[cfg(feature = "svg")]
pub mod kind_svg {
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

    pub fn svg_grid_vertical(kind: &Kind, name: &str) -> svg::Document {
        let layout = generate_kind_layout(kind, name, 0, 0);
        let num_cols = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
        let num_bits = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
        let pixels_per_char = 16_usize;
        let col_widths = (0..num_cols)
            .map(|col| {
                layout
                    .iter()
                    .filter(|x| x.row == col)
                    .map(|x| x.name.len())
                    .max()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();
        // Accumulate these widths to get the start position of each column
        let col_starts: Vec<usize> = col_widths
            .iter()
            .scan(0, |state, x| {
                let result = *state;
                *state += x;
                Some(result)
            })
            .collect();
        let total_col_width = col_widths.iter().sum::<usize>() as i32;
        let bit_digits = (num_bits as f32).log10().ceil().max(1.0) as i32;
        let mut document = Document::new().set(
            "viewBox",
            (
                -bit_digits * pixels_per_char as i32,
                0,
                (2 * bit_digits + total_col_width) * pixels_per_char as i32,
                num_bits * pixels_per_char,
            ),
        );
        let soft_palette_colors = [
            "#99FFCC", "#CCCC99", "#CCCCCC", "#CCCCFF", "#CCFF99", "#CCFFCC", "#CCFFFF", "#FFCC99",
            "#FFCCCC", "#FFCCFF", "#FFFF99", "#FFFFCC",
        ];
        // Provide a background rectangle for the diagram of light gray
        let background = svg::node::element::Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", total_col_width * pixels_per_char as i32)
            .set("height", num_bits * pixels_per_char)
            .set("fill", "#EEEEEE")
            .set("stroke", "darkblue");
        document = document.add(background);
        // Add bit rectangles to each row, and horizontal faint gray dashed grid
        // lines
        for bit in 0..num_bits {
            let x = -bit_digits * pixels_per_char as i32;
            let y = (bit * pixels_per_char) as i32;
            let width = bit_digits * pixels_per_char as i32;
            let height = pixels_per_char as i32;
            document = text_box(
                (x, y, width, height),
                &format!("{}", bit),
                "#EEEEEE",
                "darkblue",
                document,
            );
            let x = total_col_width * pixels_per_char as i32;
            document = text_box(
                (x, y, width, height),
                &format!("{}", bit),
                "#EEEEEE",
                "darkblue",
                document,
            );
            // Add a grid line in a faint dashed gray
            let line = svg::node::element::Line::new()
                .set("x1", 0)
                .set("y1", y)
                .set("x2", total_col_width * pixels_per_char as i32)
                .set("y2", y)
                .set("stroke", "#DFDFDF")
                .set("stroke-width", 1)
                .set("stroke-dasharray", "1,1");
            document = document.add(line);
        }
        // For each cell add a rectangle, where
        // the x coordinate is
        for (cell, color) in layout.iter().zip(soft_palette_colors.iter().cycle()) {
            let x = col_starts[cell.row] * pixels_per_char;
            let y = cell.cols.start * pixels_per_char;
            let width: usize = col_widths[cell.row..(cell.row + cell.depth)]
                .iter()
                .sum::<usize>()
                * pixels_per_char;
            let height = pixels_per_char * cell.cols.len();
            document = text_box(
                (x as i32, y as i32, width as i32, height as i32),
                &cell.name,
                color,
                "gray",
                document,
            );
        }
        document
    }

    pub fn svg_grid(kind: &Kind, name: &str) -> svg::Document {
        let layout = generate_kind_layout(kind, name, 0, 0);
        let num_rows = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
        let num_cols = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
        let chars_per_bit = get_chars_per_bit(&layout);
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
        let soft_palette_colors = [
            "#99FFCC", "#CCCC99", "#CCCCCC", "#CCCCFF", "#CCFF99", "#CCFFCC", "#CCFFFF", "#FFCC99",
            "#FFCCCC", "#FFCCFF", "#FFFF99", "#FFFFCC",
        ];
        // Provide a background rectangle for the diagram of light gray
        let background = svg::node::element::Rectangle::new()
            .set("x", 0)
            .set("y", 0)
            .set("width", num_cols * chars_per_bit * pixels_per_char)
            .set("height", num_rows * pixels_per_char)
            .set("fill", "#EEEEEE")
            .set("stroke", "darkblue");
        document = document.add(background);
        // Add a rectangle for each bit indicating the bit number.  One at the
        // top and one along the bottom of the diagram.
        for bit in 0..num_cols {
            let x = bit * chars_per_bit * pixels_per_char;
            let y = -(pixels_per_char as i32);
            let width = chars_per_bit * pixels_per_char;
            let height = pixels_per_char as i32;
            document = text_box(
                (x as i32, y, width as i32, height),
                &format!("{}", bit),
                "#EEEEEE",
                "darkblue",
                document,
            );
            let y = (num_rows * pixels_per_char) as i32;
            document = text_box(
                (x as i32, y, width as i32, height),
                &format!("{}", bit),
                "#EEEEEE",
                "darkblue",
                document,
            );
            // Add a grid line in a faint dashed gray
            let line = svg::node::element::Line::new()
                .set("x1", x)
                .set("y1", 0)
                .set("x2", x)
                .set("y2", (num_rows * pixels_per_char) as i32)
                .set("stroke", "#DFDFDF")
                .set("stroke-width", 1)
                .set("stroke-dasharray", "1,1");
            document = document.add(line);
        }
        // For each cell, add a rectangle to the SVG with the
        // name of the cell centered in the rectangle
        for (cell, color) in layout.iter().zip(soft_palette_colors.iter().cycle()) {
            let x = cell.cols.start * chars_per_bit * pixels_per_char;
            let y = cell.row * pixels_per_char;
            let width = cell.cols.len() * chars_per_bit * pixels_per_char;
            let height = pixels_per_char * cell.depth;
            document = text_box(
                (x as i32, y as i32, width as i32, height as i32),
                &cell.name,
                color,
                "gray",
                document,
            );
        }
        document
    }
}

#[cfg(test)]
mod test {
    use crate::rhdl_core::{
        types::kind::{Field, Variant},
        DiscriminantType, TypedBits,
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
                    Kind::make_tuple(vec![Kind::Bits(8), Kind::Bits(16)]),
                    2,
                ),
                Kind::make_variant(
                    "D",
                    Kind::make_struct(
                        "Test::D",
                        vec![
                            Kind::make_field("a", Kind::Bits(8)),
                            Kind::make_field("b", Kind::Bits(16)),
                        ],
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
                    Kind::make_tuple(vec![Kind::Bits(8), Kind::Bits(16)]),
                    2,
                ),
                Kind::make_variant(
                    "D",
                    Kind::make_struct(
                        "Test::D",
                        vec![
                            Kind::make_field("a", Kind::Bits(8)),
                            Kind::make_field("b", Kind::Bits(16)),
                        ],
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
        assert_eq!(template.bits, disc.unsigned_cast(len).unwrap().bits);
        let template = kind.enum_template("C").unwrap();
        let disc: TypedBits = 2_u64.into();
        assert_eq!(template.bits, disc.unsigned_cast(len).unwrap().bits);
        let template = kind.enum_template("D").unwrap();
        let disc: TypedBits = 3_u64.into();
        assert_eq!(template.bits, disc.unsigned_cast(len).unwrap().bits);
    }

    #[test]
    fn test_enum_template_with_signed_msb_is_correct() {
        let kind = make_enum_msb_signed_kind();
        let template = kind.enum_template("A").unwrap();
        let disc: TypedBits = (-1_i64).into();
        let disc = disc.signed_cast(4).unwrap();
        let pad = kind.pad(disc.bits);
        assert_eq!(template.bits, pad);
        let template = kind.enum_template("B").unwrap();
        let disc: TypedBits = 1_i64.into();
        let disc = disc.signed_cast(4).unwrap();
        let pad = kind.pad(disc.bits);
        assert_eq!(template.bits, pad);
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
                    name: "A".to_string(),
                    discriminant: 0,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "B".to_string(),
                    discriminant: 1,
                    kind: Kind::make_bits(8),
                },
                Variant {
                    name: "C".to_string(),
                    discriminant: 2,
                    kind: Kind::make_tuple(vec![Kind::make_bits(8), Kind::make_bits(16)]),
                },
                Variant {
                    name: "D".to_string(),
                    discriminant: 3,
                    kind: Kind::make_struct(
                        "Crazy::D",
                        vec![
                            Field {
                                name: "a".to_string(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string(),
                                kind: Kind::make_bits(16),
                            },
                        ],
                    ),
                },
                Variant {
                    name: "E".to_string(),
                    discriminant: 4,
                    kind: Kind::make_array(Kind::make_bits(8), 4),
                },
                Variant {
                    name: "F".to_string(),
                    discriminant: 5,
                    kind: Kind::make_struct(
                        "Crazy::F",
                        vec![
                            Field {
                                name: "a".to_string(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string(),
                                kind: Kind::make_array(Kind::make_bits(8), 4),
                            },
                        ],
                    ),
                },
                Variant {
                    name: "G".to_string(),
                    discriminant: 6,
                    kind: Kind::make_struct(
                        "Crazy::G",
                        vec![
                            Field {
                                name: "a".to_string(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "b".to_string(),
                                kind: Kind::make_array(Kind::make_bits(8), 4),
                            },
                            Field {
                                name: "c".to_string(),
                                kind: Kind::make_bits(16),
                            },
                        ],
                    ),
                },
                Variant {
                    name: "H".to_string(),
                    discriminant: 8,
                    kind: Kind::make_enum(
                        "Crazy::H",
                        vec![
                            Variant {
                                name: "A".to_string(),
                                discriminant: 0,
                                kind: Kind::Empty,
                            },
                            Variant {
                                name: "B".to_string(),
                                discriminant: 1,
                                kind: Kind::Bits(4),
                            },
                            Variant {
                                name: "C".to_string(),
                                discriminant: 2,
                                kind: Kind::Empty,
                            },
                            Variant {
                                name: "Unknown".to_string(),
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
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        assert!(is_layout_valid(&layout));
        println!("Chars per bit {}", get_chars_per_bit(&layout));
        println!("{}", text_grid(&kind, "value"));
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&kind, "value");
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_vertical(&kind, "value");
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }
    #[test]
    fn test_layout_of_struct() {
        let kind = Kind::make_struct(
            "Foo",
            vec![
                Field {
                    name: "a".to_string(),
                    kind: Kind::make_bits(8),
                },
                Field {
                    name: "b".to_string(),
                    kind: Kind::make_bits(16),
                },
                Field {
                    name: "c".to_string(),
                    kind: Kind::make_bits(32),
                },
            ],
        );
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&kind, "value");
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_vertical(&kind, "value");
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }
    #[test]
    fn test_layout_of_struct_with_nesting() {
        let kind = Kind::make_struct(
            "Foo",
            vec![
                Field {
                    name: "a".to_string(),
                    kind: Kind::make_bits(8),
                },
                Field {
                    name: "b".to_string(),
                    kind: Kind::make_bits(16),
                },
                Field {
                    name: "c".to_string(),
                    kind: Kind::make_struct(
                        "Foo:c",
                        vec![
                            Field {
                                name: "d".to_string(),
                                kind: Kind::make_bits(8),
                            },
                            Field {
                                name: "e".to_string(),
                                kind: Kind::make_bits(16),
                            },
                        ],
                    ),
                },
            ],
        );
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&kind, "value");
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_vertical(&kind, "value");
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }

    #[test]
    fn test_layout_of_simple_enum() {
        let kind = Kind::make_enum(
            "Simple",
            vec![
                Variant {
                    name: "A".to_string(),
                    discriminant: 0,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "B".to_string(),
                    discriminant: 1,
                    kind: Kind::Empty,
                },
                Variant {
                    name: "C".to_string(),
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
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&kind, "value");
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_vertical(&kind, "value");
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }

    #[test]
    fn test_result_recognized() {
        use crate::rhdl_bits::alias::*;
        use crate::rhdl_core::Digital;
        let a = std::result::Result::<b8, b8>::Ok(b8(42)).typed_bits();
        assert!(a.kind.is_result());
        let b = std::result::Result::<b4, ()>::Err(()).typed_bits();
        assert!(b.kind.is_result());
    }
}
