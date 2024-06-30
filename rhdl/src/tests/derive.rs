use rhdl_bits::Bits;
use rhdl_core::{
    note, note_init_db, note_take, note_time,
    types::path::{bit_range, Path},
    Digital, Kind,
};
use rhdl_macro::Digital;

#[test]
#[allow(dead_code)]
#[allow(clippy::just_underscores_and_digits)]
fn test_derive() {
    #[derive(Clone, Copy, PartialEq, Digital)]
    enum Test {
        A,
        B(Bits<16>),
        C {
            a: Bits<32>,
            b: Bits<8>,
        },
        #[rhdl(unmatched)]
        D,
    }
    note("test", Test::A);
}

#[test]
#[allow(dead_code)]
fn test_derive_no_payload() {
    #[derive(Copy, Clone, PartialEq, Digital)]
    pub enum State {
        Init,
        Boot,
        Running,
        Stop,
        Boom,
        #[rhdl(unmatched)]
        Unknown,
    }
    note("state", State::Running);
}

#[test]
fn test_derive_digital_simple_struct() {
    use rhdl_bits::alias::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    struct Test {
        a: bool,
        b: b8,
    }

    let foo = Test {
        a: true,
        b: b8::from(0b10101011),
    };

    println!("foo val: {}", foo.binary_string());
    let test_kind = Test::static_kind();
    let (range, kind) = bit_range(test_kind, &Path::default().field("b")).unwrap();
    println!("range: {:?}", range);
    println!("kind: {:?}", kind);
    assert_eq!(range, 1..9);
    assert_eq!(kind, Kind::make_bits(8));
    let bits = foo.bin();
    let bits = &bits[range];
    assert_eq!(bits.len(), 8);
    assert_eq!(bits, [true, true, false, true, false, true, false, true]);
}

#[test]
#[allow(dead_code)]
fn test_derive_complex_enum_and_decode_with_path() -> anyhow::Result<()> {
    use rhdl_bits::alias::*;
    use rhdl_core::types::path::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[rhdl(unmatched)]
        D,
    }

    let foo = Test::B(b2::from(0b10), b3::from(0b101));
    let disc = Path::default().payload(stringify!(B)).tuple_index(1);
    let index = bit_range(Test::static_kind(), &disc)?;
    println!("{:?}", index);
    let bits = foo.bin();
    let bits = &bits[index.0];
    println!(
        "Extracted bits: {}",
        bits.iter()
            .rev()
            .map(|x| if *x { '1' } else { '0' })
            .collect::<String>()
    );
    let (disc_range, disc_kind) = bit_range(Test::static_kind(), &Path::default().discriminant())?;
    println!("{:?}", disc_range);
    println!("{:?}", disc_kind);
    let disc_bits = foo.bin();
    let disc_bits = &disc_bits[disc_range];
    assert_eq!(disc_bits, [true, false]);
    Ok(())
}

#[test]
fn test_derive_digital_complex_enum() {
    use rhdl_bits::alias::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[rhdl(unmatched)]
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
    note_init_db();
    note_time(0);
    note("test", foo_1);
    note_time(1_000);
    note("test", foo_2);
    note_time(2_000);
    note("test", foo_3);
    note_time(3_000);
    note("test", foo_1);
    let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_derive_enum_explicit_discriminant_width() {
    use rhdl_bits::alias::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    #[rhdl(discriminant_width = 4)]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[rhdl(unmatched)]
        D,
    }

    let (range, kind) = bit_range(Test::static_kind(), &Path::default().discriminant()).unwrap();
    assert_eq!(range.len(), 4);
    assert_eq!(kind, Kind::make_bits(4));
}

#[test]
fn test_derive_enum_alignment_lsb() {
    use rhdl_bits::alias::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    #[rhdl(discriminant_align = "lsb")]
    enum Test {
        A,
        B(b2, b3),
        C {
            a: b8,
            b: b8,
        },
        #[rhdl(unmatched)]
        D,
    }
    let (range, kind) = bit_range(Test::static_kind(), &Path::default().discriminant()).unwrap();
    assert_eq!(range, 0..2);
    assert_eq!(kind, Kind::make_bits(2));
}
