#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;
use rhdl_core::trace2::{TraceContainer, session::Session};

#[test]
#[allow(dead_code)]
#[allow(clippy::just_underscores_and_digits)]
fn test_derive() {
    #[derive(PartialEq, Default, Clone, Copy, Digital)]
    enum Test {
        A,
        B(Bits<16>),
        C {
            a: Bits<32>,
            b: Bits<8>,
        },
        #[default]
        D,
    }
    trace("test", &Test::A);
}

#[test]
#[allow(dead_code)]
fn test_derive_no_payload() {
    #[derive(PartialEq, Default, Clone, Copy, Digital)]
    pub enum State {
        Init,
        Boot,
        Running,
        Stop,
        Boom,
        #[default]
        Unknown,
    }
    trace("state", &State::Running);
}

#[test]
fn test_derive_digital_simple_struct() {
    use rhdl::bits::alias::*;

    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    struct Test {
        a: bool,
        b: b8,
    }

    let foo_test = Test {
        a: true,
        b: b8::from(0b10101011),
    };

    println!("foo val: {}", foo_test.binary_string());
    let test_kind = Test::static_kind();
    let (range, kind) = bit_range(test_kind, &Path::default().field("b")).unwrap();
    println!("range: {range:?}");
    println!("kind: {kind:?}");
    assert_eq!(range, 1..9);
    assert_eq!(kind, Kind::make_bits(8));
    let bits = foo_test.bin();
    let bits = &bits[range];
    assert_eq!(bits.len(), 8);
    assert_eq!(
        *bits,
        *bitx_vec(&[true, true, false, true, false, true, false, true])
    );
}

#[test]
#[allow(dead_code)]
fn test_derive_complex_enum_and_decode_with_path() -> anyhow::Result<()> {
    use rhdl::bits::alias::*;
    use rhdl::core::types::path::*;

    #[derive(PartialEq, Debug, Default, Clone, Copy, Digital)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[default]
        D,
    }

    let foo_test = Test::B(b2::from(0b10), b3::from(0b101));
    let disc = Path::default().payload(stringify!(B)).tuple_index(1);
    let index = bit_range(Test::static_kind(), &disc)?;
    println!("{index:?}");
    let bits = foo_test.bin();
    let bits = &bits[index.0];
    println!("Extracted bits: {}", bitx_string(bits));
    let (disc_range, disc_kind) = bit_range(Test::static_kind(), &Path::default().discriminant())?;
    println!("{disc_range:?}");
    println!("{disc_kind:?}");
    let disc_bits = foo_test.bin();
    let disc_bits = &disc_bits[disc_range];
    assert_eq!(disc_bits, [BitX::One, BitX::Zero]);
    Ok(())
}

#[test]
fn test_derive_digital_complex_enum() {
    use rhdl::bits::alias::*;

    #[derive(PartialEq, Debug, Default, Clone, Copy, Digital)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[default]
        D,
    }

    let foo_1 = Test::C {
        a: b8::from(0b10101010),
        b: b8::from(0b11010111),
    };

    println!("foo val: {}", foo_1.binary_string());

    let foo_2 = Test::B(b2::from(0b10), b3::from(0b101));

    println!("foo val: {}", foo_2.binary_string());

    let foo_3 = Test::A;
    let session = Session::default();
    let mut vcd_file = Vcd::new();
    let t0 = session.traced_at_time(0, || trace("test", &foo_1));
    vcd_file.record(&t0).unwrap();
    let t1 = session.traced_at_time(1_000, || trace("test", &foo_2));
    vcd_file.record(&t1).unwrap();
    let t2 = session.traced_at_time(2_000, || trace("test", &foo_3));
    vcd_file.record(&t2).unwrap();
    let t3 = session.traced_at_time(3_000, || trace("test", &foo_1));
    vcd_file.record(&t3).unwrap();
    let mut vcd = vec![];
    vcd_file.finalize(&mut vcd);
    expect_test::expect_file!["expect/derive_enum_vcd.expect"]
        .assert_eq(&String::from_utf8(vcd).unwrap());
}

#[test]
fn test_derive_enum_explicit_discriminant_width() {
    use rhdl::bits::alias::*;

    #[derive(PartialEq, Debug, Default, Clone, Copy, Digital)]
    #[rhdl(discriminant_width = 4)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[default]
        D,
    }

    let (range, kind) = bit_range(Test::static_kind(), &Path::default().discriminant()).unwrap();
    assert_eq!(range.len(), 4);
    assert_eq!(kind, Kind::make_bits(4));
}

#[test]
fn test_derive_enum_alignment_lsb() {
    use rhdl::bits::alias::*;

    #[derive(PartialEq, Debug, Default, Clone, Copy, Digital)]
    #[rhdl(discriminant_align = "lsb")]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[default]
        D,
    }
    let (range, kind) = bit_range(Test::static_kind(), &Path::default().discriminant()).unwrap();
    assert_eq!(range, 0..2);
    assert_eq!(kind, Kind::make_bits(2));
}

#[test]
fn test_derive_generic_struct() {
    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct Request<T, const N: usize>
    where
        T: Digital,
        rhdl_bits::W<N>: BitWidth,
    {
        data: T,
        address: Bits<N>,
    }
}
