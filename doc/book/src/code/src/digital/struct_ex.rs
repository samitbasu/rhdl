// ANCHOR: prelude
use rhdl::prelude::*;
// ANCHOR_END: prelude

// ANCHOR: struct-def-with-derive
// 👇 - new
#[derive(Copy, PartialEq, Clone)]
// ANCHOR: struct-definition
pub struct Things {
    pub count: b4,
    pub valid: bool,
    pub coordinates: (s6, s4),
    pub zst: (),
}
// ANCHOR_END: struct-definition
// ANCHOR_END: struct-def-with-derive

// ANCHOR: struct-BITS
impl Digital for Things {
    const BITS: usize = 15;
    // ANCHOR_END: struct-BITS

    // ANCHOR: struct-static_kind
    fn static_kind() -> Kind {
        let count_field = Kind::make_field("count", <b4 as Digital>::static_kind());
        let valid_field = Kind::make_field("valid", bool::static_kind());
        let coordinates_field =
            Kind::make_field("coordinates", <(s6, s4) as Digital>::static_kind());
        let zst_field = Kind::make_field("zst", <() as Digital>::static_kind());
        Kind::make_struct(
            "Things",
            [count_field, valid_field, coordinates_field, zst_field].into(),
        )
    }
    // ANCHOR_END: struct-static_kind
    // ANCHOR: struct-bin
    fn bin(self) -> Box<[BitX]> {
        let mut bits = Vec::with_capacity(Self::BITS);
        bits.extend(self.count.bin());
        bits.extend(self.valid.bin());
        bits.extend(self.coordinates.bin());
        bits.extend(().bin());
        bits.into_boxed_slice()
    }
    // ANCHOR_END: struct-bin
    // ANCHOR: struct-dont_care
    fn dont_care() -> Self {
        Self {
            count: b4::dont_care(),
            valid: bool::dont_care(),
            coordinates: <(s6, s4)>::dont_care(),
            zst: (),
        }
    }
    // ANCHOR_END: struct-dont_care
}

#[test]
// ANCHOR: struct-layout
fn test_struct_layout() {
    let svg = Things::static_kind().svg("Things");
    std::fs::write("things.svg", svg.to_string()).unwrap();
}
// ANCHOR_END: struct-layout
