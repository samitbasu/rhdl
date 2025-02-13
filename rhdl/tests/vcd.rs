#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]
use expect_test::expect;
use rhdl::{core::trace::svgx::format_as_label, prelude::*};

#[test]
fn test_vcd_enum() {
    #[derive(PartialEq, Debug, Digital, Default)]
    enum Enum {
        #[default]
        None,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
    }

    let guard = trace_init_db();
    trace_time(0);
    trace("enum", &Enum::None);
    trace("color", &b8(0b10101010));
    trace_time(1_000);
    trace("enum", &Enum::A(bits(42), bits(1024)));
    trace_time(2_000);
    trace("enum", &Enum::B { name: bits(67) });
    trace_time(3_000);
    trace("enum", &Enum::C(true));
    trace_time(4_000);
    trace("enum", &Enum::C(false));
    trace_time(5_000);
    trace("enum", &Enum::B { name: bits(65) });
    trace_time(6_000);
    trace("enum", &Enum::A(bits(21), bits(512)));
    trace_time(7_000);
    trace("enum", &Enum::None);
    trace_time(8_000);
    trace("enum", &Enum::None);
    let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
    guard.take().dump_vcd(&mut vcd_file, None).unwrap();
}

#[test]
fn test_vcd_basic() {
    #[derive(PartialEq, Digital)]
    pub struct Simple {
        a: bool,
        b: Bits<U8>,
    }

    let simple = Simple {
        a: true,
        b: Bits::from(0b10101010),
    };
    let guard = trace_init_db();
    trace_time(0);
    trace("simple", &simple);
    trace_time(1_000);
    let simple = Simple {
        a: false,
        b: Bits::from(0b01010101),
    };
    let mut snapshot = std::fs::File::create("snapshot.vcd").unwrap();
    with_trace_db(|db| db.dump_vcd(&mut snapshot, None).unwrap());
    trace("simple", &simple);
    trace_time(2_000);
    trace("simple", &simple);
    let mut vcd_file = std::fs::File::create("test.vcd").unwrap();
    guard.take().dump_vcd(&mut vcd_file, None).unwrap();
}

#[test]
fn test_label_for_tuple_struct() {
    #[derive(PartialEq, Digital)]
    pub struct TupleStruct(b6, b3);

    let tuple = TupleStruct(bits(13), bits(4));
    let label = format_as_label(&tuple.typed_bits()).unwrap();
    let expect = expect!["{0: 0d, 1: 4}"];
    expect.assert_eq(&label);
}

#[test]
fn test_label_for_struct() {
    #[derive(PartialEq, Digital)]
    pub struct Simple {
        a: b4,
        b: (b4, b4),
        c: [b5; 3],
    }

    let simple = Simple {
        a: bits(6),
        b: (bits(8), bits(9)),
        c: [bits(10), bits(11), bits(12)],
    };

    let label = format_as_label(&simple.typed_bits()).unwrap();
    let expect = expect!["{a: 6, b: (8, 9), c: [0a, 0b, 0c]}"];
    expect.assert_eq(&label);
}

#[test]
fn test_label_for_signed() {
    #[derive(PartialEq, Digital)]
    pub struct Signed {
        a: s8,
        b: b8,
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
    #[derive(PartialEq, Digital, Default)]
    enum Value {
        #[default]
        Empty,
        A(b8, b16),
        B {
            name: b8,
        },
        C(bool),
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
