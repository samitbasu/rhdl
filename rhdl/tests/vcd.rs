#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;

#[test]
fn test_vcd_enum() {
    #[derive(Clone, Copy, Debug, PartialEq, Digital, Default)]
    enum Enum {
        #[default]
        None,
        A(u8, u16),
        B {
            name: u8,
        },
        C(bool),
    }

    note_init_db();
    note_time(0);
    note("enum", Enum::None);
    note("color", bits::<8>(0b10101010));
    note_time(1_000);
    note("enum", Enum::A(42, 1024));
    note_time(2_000);
    note("enum", Enum::B { name: 67 });
    note_time(3_000);
    note("enum", Enum::C(true));
    note_time(4_000);
    note("enum", Enum::C(false));
    note_time(5_000);
    note("enum", Enum::B { name: 65 });
    note_time(6_000);
    note("enum", Enum::A(21, 512));
    note_time(7_000);
    note("enum", Enum::None);
    note_time(8_000);
    note("enum", Enum::None);
    let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
    note_take().unwrap().dump_vcd(&mut vcd_file).unwrap();
}

#[test]
fn test_vcd_basic() {
    #[derive(Clone, Copy, PartialEq, Digital)]
    pub struct Simple {
        a: bool,
        b: Bits<8>,
    }

    let simple = Simple {
        a: true,
        b: Bits::from(0b10101010),
    };
    note_init_db();
    note_time(0);
    note("simple", simple);
    note_time(1_000);
    let simple = Simple {
        a: false,
        b: Bits::from(0b01010101),
    };
    note("simple", simple);
    note_time(2_000);
    note("simple", simple);
    let mut vcd_file = std::fs::File::create("test.vcd").unwrap();
    note_take().unwrap().dump_vcd(&mut vcd_file).unwrap();
}
