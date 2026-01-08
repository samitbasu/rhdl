// ANCHOR: prelude
use rhdl::prelude::*;
// ANCHOR_END: prelude

// ANCHOR: derive
#[derive(Copy, PartialEq, Clone)] // 👈 Needed by Digital
// ANCHOR: def
pub enum OpCode {
    Nop,
    Add(b8, b8),
    Sub(b8, b8),
    Not(b8),
}
// ANCHOR_END: def
// ANCHOR_END: derive

// ANCHOR: BITS
impl Digital for OpCode {
    const BITS: usize = 18;
    // ANCHOR_END: BITS
    // ANCHOR: static_kind
    fn static_kind() -> Kind {
        let nop_variant = Kind::make_variant("Nop", <() as Digital>::static_kind(), 0);
        let add_variant = Kind::make_variant("Add", <(b8, b8) as Digital>::static_kind(), 1);
        let sub_variant = Kind::make_variant("Sub", <(b8, b8) as Digital>::static_kind(), 2);
        let not_variant = Kind::make_variant("Not", <b8 as Digital>::static_kind(), 3);
        let alignment = rhdl::core::DiscriminantAlignment::Msb;
        let ty = rhdl::core::DiscriminantType::Unsigned;
        let layout = Kind::make_discriminant_layout(2, alignment, ty);
        Kind::make_enum(
            "OpCode",
            [nop_variant, add_variant, sub_variant, not_variant].into(),
            layout,
        )
    }
    // ANCHOR_END: static_kind
    // ANCHOR: bin
    fn bin(self) -> Box<[BitX]> {
        let mut bits = Vec::with_capacity(Self::BITS);
        match self {
            OpCode::Nop => {
                bits.extend(().bin());
                bits.extend(b16(0).bin());
                bits.extend(b2(0b00).bin());
            }
            OpCode::Add(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b01).bin());
            }
            OpCode::Sub(a, b) => {
                bits.extend((a, b).bin());
                bits.extend(b2(0b10).bin());
            }
            OpCode::Not(a) => {
                bits.extend(a.bin());
                bits.extend(b8(0).bin());
                bits.extend(b2(0b11).bin());
            }
        }
        bits.into_boxed_slice()
    }
    // ANCHOR_END: bin
    // ANCHOR: dont_care
    fn dont_care() -> Self {
        Self::Nop
    }
    // ANCHOR_END: dont_care
}

#[test]
// ANCHOR: test_opcode_layout
fn test_opcode_layout() {
    let svg = OpCode::static_kind().svg("OpCode");
    if !std::path::Path::new("opcode.svg").exists() {
        std::fs::write("opcode.svg", svg.to_string()).unwrap();
    }
}
// ANCHOR_END: test_opcode_layout
