use std::{iter::repeat, ops::Range};

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Array(Array),
    Tuple(Tuple),
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    Bits(usize),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub base: Box<Kind>,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    pub elements: Vec<Kind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscriminantAlignment {
    Msb,
    Lsb,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub variants: Vec<Variant>,
    pub discriminant_width: Option<usize>,
    pub discriminant_alignment: DiscriminantAlignment,
}

impl Enum {
    fn discriminant_width(&self) -> usize {
        self.discriminant_width.unwrap_or_else(|| {
            self.variants
                .iter()
                .map(|x| x.discriminant)
                .max()
                .map(|x| clog2(x.max(1) + 1))
                .unwrap_or(0)
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub discriminant: usize,
    pub kind: Kind,
}

pub const fn clog2(t: usize) -> usize {
    let mut p = 0;
    let mut b = 1;
    while b < t {
        p += 1;
        b *= 2;
    }
    p
}

impl Kind {
    pub fn make_array(base: Kind, size: usize) -> Self {
        Self::Array(Array {
            base: Box::new(base),
            size,
        })
    }
    pub fn make_tuple(elements: Vec<Kind>) -> Self {
        Self::Tuple(Tuple { elements })
    }
    pub fn make_struct(fields: Vec<Field>) -> Self {
        Self::Struct(Struct { fields })
    }
    pub fn make_union(fields: Vec<Field>) -> Self {
        Self::Union(Union { fields })
    }
    pub fn make_enum(
        variants: Vec<Variant>,
        discriminant_width: Option<usize>,
        discriminant_alignment: DiscriminantAlignment,
    ) -> Self {
        Self::Enum(Enum {
            variants,
            discriminant_width,
            discriminant_alignment,
        })
    }
    pub fn make_bits(digits: usize) -> Self {
        Self::Bits(digits)
    }
    pub fn bits(&self) -> usize {
        match self {
            Kind::Array(array) => array.base.bits() * array.size,
            Kind::Tuple(tuple) => tuple.elements.iter().map(|x| x.bits()).sum(),
            Kind::Struct(kind) => kind.fields.iter().map(|x| x.kind.bits()).sum(),
            Kind::Union(kind) => kind.fields.iter().map(|x| x.kind.bits()).max().unwrap_or(0),
            Kind::Enum(kind) => {
                kind.discriminant_width()
                    + kind
                        .variants
                        .iter()
                        .map(|x| x.kind.bits())
                        .max()
                        .unwrap_or(0)
            }
            Kind::Bits(digits) => *digits,
            Kind::Empty => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KindLayout {
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
        Kind::Bits(digits) => {
            vec![KindLayout {
                row: offset_row,
                depth: 1,
                cols: offset_col..offset_col + digits,
                name: format!("{name} b{digits}"),
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
        Kind::Union(u) => {
            let mut result = vec![KindLayout {
                row: offset_row,
                depth: 1,
                cols: offset_col..(offset_col + kind.bits()),
                name: format!("{name} {}", kind.bits()),
            }];
            for field in &u.fields {
                let variant =
                    generate_kind_layout(&field.kind, &field.name, offset_row + 1, offset_col);
                offset_row = variant
                    .iter()
                    .map(|x| x.row)
                    .max()
                    .unwrap_or(offset_row + 1);
                result.extend(variant);
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
            let variant_cols = match e.discriminant_alignment {
                DiscriminantAlignment::Lsb => offset_col..(offset_col + e.discriminant_width()),
                DiscriminantAlignment::Msb => {
                    offset_col + kind.bits() - e.discriminant_width()..(offset_col + kind.bits())
                }
            };
            let payload_offset = match e.discriminant_alignment {
                DiscriminantAlignment::Lsb => offset_col + e.discriminant_width(),
                DiscriminantAlignment::Msb => offset_col,
            };
            let disc_width = e.discriminant_width();
            for variant in &e.variants {
                result.push(KindLayout {
                    row: offset_row + 1,
                    depth: 1,
                    cols: variant_cols.clone(),
                    name: format!(
                        "{}({:0width$b})",
                        variant.name,
                        variant.discriminant,
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
pub fn is_layout_valid(layout: &[KindLayout]) -> bool {
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
        .map(|x| (x.name.len() + x.cols.len() - 1) / x.cols.len())
        .max()
        .unwrap_or(0)
}

// Generate a string (text) representation of the layout
pub fn text_grid(layout: &[KindLayout]) -> String {
    let chars_per_bit = get_chars_per_bit(layout);
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

    pub fn svg_grid_veritcal(layout: &[KindLayout]) -> svg::Document {
        let num_cols = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
        let num_bits = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
        let pixels_per_char = 16;
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
        let total_col_width = col_widths.iter().sum::<usize>();
        dbg!(&col_widths);
        dbg!(&col_starts);
        dbg!(total_col_width);
        let mut document = Document::new().set(
            "viewBox",
            (
                0,
                0,
                total_col_width * pixels_per_char,
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
            .set("width", total_col_width * pixels_per_char)
            .set("height", num_bits * pixels_per_char)
            .set("fill", "#EEEEEE")
            .set("stroke", "darkblue");
        document = document.add(background);
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
            let text_x = x + width / 2;
            let text_y = y + height / 2;
            let text = svg::node::element::Text::new()
                .add(svg::node::Text::new(cell.name.clone()))
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
                .set("fill", *color)
                .set("stroke", "gray");
            document = document.add(rect).add(text);
        }
        document
    }

    pub fn svg_grid(layout: &[KindLayout]) -> svg::Document {
        let num_rows = layout.iter().map(|x| x.row).max().unwrap_or(0) + 1;
        let num_cols = layout.iter().map(|x| x.cols.end).max().unwrap_or(0);
        let chars_per_bit = get_chars_per_bit(layout);
        let pixels_per_char = 16;
        let mut document = Document::new().set(
            "viewBox",
            (
                0,
                0,
                num_cols * chars_per_bit * pixels_per_char,
                num_rows * pixels_per_char,
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
        // For each cell, add a rectangle to the SVG with the
        // name of the cell centered in the rectangle
        for (cell, color) in layout.iter().zip(soft_palette_colors.iter().cycle()) {
            let x = cell.cols.start * chars_per_bit * pixels_per_char;
            let y = cell.row * pixels_per_char;
            let width = cell.cols.len() * chars_per_bit * pixels_per_char;
            let height = pixels_per_char * cell.depth;
            let text_x = x + width / 2;
            let text_y = y + height / 2;
            let text = svg::node::element::Text::new()
                .add(svg::node::Text::new(cell.name.clone()))
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
                .set("fill", *color)
                .set("stroke", "gray");
            document = document.add(rect).add(text);
        }
        document
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
                    kind: Kind::make_struct(vec![
                        Field {
                            name: "a".to_string(),
                            kind: Kind::make_bits(8),
                        },
                        Field {
                            name: "b".to_string(),
                            kind: Kind::make_bits(16),
                        },
                    ]),
                },
                Variant {
                    name: "E".to_string(),
                    discriminant: 4,
                    kind: Kind::make_array(Kind::make_bits(8), 4),
                },
                Variant {
                    name: "F".to_string(),
                    discriminant: 5,
                    kind: Kind::make_struct(vec![
                        Field {
                            name: "a".to_string(),
                            kind: Kind::make_bits(8),
                        },
                        Field {
                            name: "b".to_string(),
                            kind: Kind::make_array(Kind::make_bits(8), 4),
                        },
                    ]),
                },
                Variant {
                    name: "F2".to_string(),
                    discriminant: 7,
                    kind: Kind::make_union(vec![
                        Field {
                            name: "op_code".to_string(),
                            kind: Kind::make_bits(4),
                        },
                        Field {
                            name: "count".to_string(),
                            kind: Kind::make_array(Kind::make_bits(2), 4),
                        },
                    ]),
                },
                Variant {
                    name: "G".to_string(),
                    discriminant: 6,
                    kind: Kind::make_struct(vec![
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
                    ]),
                },
                Variant {
                    name: "H".to_string(),
                    discriminant: 8,
                    kind: Kind::make_enum(
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
                        ],
                        None,
                        DiscriminantAlignment::Msb,
                    ),
                },
            ],
            None,
            DiscriminantAlignment::Lsb,
        )
    }

    #[test]
    fn test_layout_of_complex_kind() {
        let kind = make_complex_kind();
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        assert!(is_layout_valid(&layout));
        println!("Chars per bit {}", get_chars_per_bit(&layout));
        println!("{}", text_grid(&layout));
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&layout);
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_veritcal(&layout);
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }
    #[test]
    fn test_layout_of_struct() {
        let kind = Kind::make_struct(vec![
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
        ]);
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&layout);
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_veritcal(&layout);
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }
    #[test]
    fn test_layout_of_struct_with_nesting() {
        let kind = Kind::make_struct(vec![
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
                kind: Kind::make_struct(vec![
                    Field {
                        name: "d".to_string(),
                        kind: Kind::make_bits(8),
                    },
                    Field {
                        name: "e".to_string(),
                        kind: Kind::make_bits(16),
                    },
                ]),
            },
        ]);
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&layout);
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_veritcal(&layout);
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }

    #[test]
    fn test_layout_of_union() {
        let kind = Kind::make_union(vec![
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
        ]);
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&layout);
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_veritcal(&layout);
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }

    #[test]
    fn test_layout_of_simple_enum() {
        let kind = Kind::make_enum(
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
            None,
            DiscriminantAlignment::Lsb,
        );
        let layout = generate_kind_layout(&kind, "value", 0, 0);
        println!("{:#?}", layout);
        #[cfg(feature = "svg")]
        {
            let svg = kind_svg::svg_grid(&layout);
            svg::save("test.svg", &svg).unwrap();
            let svg = kind_svg::svg_grid_veritcal(&layout);
            svg::save("test_vertical.svg", &svg).unwrap();
        }
    }
}
