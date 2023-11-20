pub mod basic_logger;
pub mod bits;
pub mod core;

pub use crate::bits::Bits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use crate::core::LogBuilder;
pub use crate::core::LoggerImpl;
pub use crate::core::TagID;
use rhdl_macro::hdl;
pub use rhdl_macro::kernel;
pub use rhdl_macro::Digital;

#[cfg(test)]
mod tests {

    use bits::b4;
    use rhdl_bits::{bits, signed};
    use rhdl_core::{
        ascii::render_ast_to_string,
        assign_node::assign_node_ids,
        compiler::Compiler,
        digital_fn::DigitalFn,
        display_ast::pretty_print_kernel,
        infer_types::TypeInference,
        kernel::Kernel,
        path::{bit_range, Path},
        typer::infer_type,
        DiscriminantAlignment, Logger,
    };

    use super::*;

    #[test]
    fn test_vcd_enum() {
        #[derive(Clone, Copy, Debug, PartialEq, Default)]
        enum Enum {
            #[default]
            None,
            A(u8, u16),
            B {
                name: u8,
            },
            C(bool),
        }

        impl Digital for Enum {
            fn static_kind() -> Kind {
                Kind::make_enum(
                    "Enum",
                    vec![
                        Kind::make_variant("None", Kind::Empty, 0),
                        Kind::make_variant(
                            "A",
                            Kind::make_tuple(vec![Kind::make_bits(8), Kind::make_bits(16)]),
                            1,
                        ),
                        Kind::make_variant(
                            "B",
                            Kind::make_struct(
                                "Enum::B",
                                vec![Kind::make_field("name", Kind::make_bits(8))],
                            ),
                            2,
                        ),
                        Kind::make_variant(
                            "C",
                            Kind::make_struct(
                                "Enum::C",
                                vec![Kind::make_field("a", Kind::make_bits(1))],
                            ),
                            3,
                        ),
                    ],
                    3,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                let raw = match self {
                    Enum::None => rhdl_bits::bits::<2>(0).to_bools(),
                    Enum::A(a, b) => {
                        let mut v = rhdl_bits::bits::<2>(1).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                    Enum::B { name } => {
                        let mut v = rhdl_bits::bits::<2>(2).to_bools();
                        v.extend(name.bin());
                        v
                    }
                    Enum::C(a) => {
                        let mut v = rhdl_bits::bits::<2>(3).to_bools();
                        v.extend(a.bin());
                        v
                    }
                };
                let raw = if raw.len() < self.kind().bits() {
                    let missing = self.kind().bits() - raw.len();
                    raw.into_iter()
                        .chain(std::iter::repeat(false).take(missing))
                        .collect()
                } else {
                    raw
                };
                // if alignment is msb, move the bottom bits to the top
                if DiscriminantAlignment::Msb == DiscriminantAlignment::Msb {
                    let (payload, discriminant) = raw.split_at(2);
                    discriminant.iter().chain(payload.iter()).copied().collect()
                } else {
                    raw
                }
            }
            fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
                // Allocate the enum tag
                builder.namespace("$disc").allocate(tag, 0);
                // For the variants, allocate space for them
                // For the None variant, we do not need to allocate additional space
                // For the A variant, we need to allocate space for the u8 and u16
                {
                    let builder = builder.namespace("A");
                    <u8 as Digital>::allocate(tag, builder.namespace("0"));
                    <u16 as Digital>::allocate(tag, builder.namespace("1"));
                }
                // The struct case must be done inline
                {
                    let builder = builder.namespace("B");
                    <u8 as Digital>::allocate(tag, builder.namespace("name"));
                    <u8 as Digital>::allocate(tag, builder.namespace("name_2"));
                }
                <bool as Digital>::allocate(tag, builder.namespace("C"));
            }
            fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
                match self {
                    Enum::None => {
                        logger.write_string(tag, "None");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::A(t, b) => {
                        logger.write_string(tag, "A");
                        logger.write_bits(tag, *t as u128);
                        logger.write_bits(tag, *b as u128);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::B { name } => {
                        logger.write_string(tag, "B");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        logger.write_bits(tag, *name as u128);
                        logger.write_bits(tag, *name as u128);
                        <bool as Digital>::skip(tag, &mut logger);
                    }
                    Enum::C(a) => {
                        logger.write_string(tag, "C");
                        <(u8, u16) as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        <u8 as Digital>::skip(tag, &mut logger);
                        logger.write_bool(tag, *a);
                    }
                }
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                logger.skip(tag);
                <(u8, u16) as Digital>::skip(tag, &mut logger);
                <u8 as Digital>::skip(tag, &mut logger);
                <bool as Digital>::skip(tag, &mut logger);
            }
        }

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag::<Enum>("enum");
        let tag2 = builder.tag::<u8>("color");
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, Enum::None);
        logger.log(tag2, 0b10101010);
        logger.set_time_in_fs(1_000);
        logger.log(tag, Enum::A(42, 1024));
        logger.set_time_in_fs(2_000);
        logger.log(tag, Enum::B { name: 67 });
        logger.set_time_in_fs(3_000);
        logger.log(tag, Enum::C(true));
        logger.set_time_in_fs(4_000);
        logger.log(tag, Enum::C(false));
        logger.set_time_in_fs(5_000);
        logger.log(tag, Enum::B { name: 65 });
        logger.set_time_in_fs(6_000);
        logger.log(tag, Enum::A(21, 512));
        logger.set_time_in_fs(7_000);
        logger.log(tag, Enum::None);
        logger.set_time_in_fs(8_000);
        logger.log(tag, Enum::None);
        let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
        println!("{:?} {}", Enum::None, Enum::None.binary_string());
        assert_eq!(Enum::None.bin().len(), Enum::static_kind().bits());
        let a = Enum::A(21, 512);
        println!("{:?} {}", a, a.binary_string());
        assert_eq!(a.bin().len(), Enum::static_kind().bits());
    }

    #[test]
    fn test_vcd_basic() {
        #[derive(Clone, Copy, PartialEq, Default)]
        pub struct Simple {
            a: bool,
            b: Bits<8>,
        }

        impl Digital for Simple {
            fn static_kind() -> Kind {
                Kind::make_struct(
                    "Simple",
                    vec![
                        Kind::make_field("a", Kind::make_bits(1)),
                        Kind::make_field("b", Kind::make_bits(8)),
                    ],
                )
            }
            fn bin(self) -> Vec<bool> {
                let mut result = vec![self.a];
                result.extend(self.b.bin());
                result
            }
            fn allocate<T: Digital>(tag: TagID<T>, builder: impl LogBuilder) {
                <bool as Digital>::allocate(tag, builder.namespace("a"));
                <Bits<8> as Digital>::allocate(tag, builder.namespace("b"));
            }
            fn record<T: Digital>(&self, tag: TagID<T>, mut logger: impl LoggerImpl) {
                self.a.record(tag, &mut logger);
                self.b.record(tag, &mut logger);
            }
            fn skip<T: Digital>(tag: TagID<T>, mut logger: impl LoggerImpl) {
                <bool as Digital>::skip(tag, &mut logger);
                <Bits<8> as Digital>::skip(tag, &mut logger);
            }
        }

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag::<Simple>("simple");
        let simple = Simple {
            a: true,
            b: Bits::from(0b10101010),
        };
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, simple);
        logger.set_time_in_fs(1_000);
        let simple = Simple {
            a: false,
            b: Bits::from(0b01010101),
        };
        logger.log(tag, simple);
        logger.set_time_in_fs(2_000);
        logger.log(tag, simple);
        let mut vcd_file = std::fs::File::create("test.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
    }

    #[test]
    #[allow(dead_code)]
    #[allow(clippy::just_underscores_and_digits)]
    fn test_derive() {
        #[derive(Clone, Copy, PartialEq, Default)]
        enum Test {
            #[default]
            A,
            B(Bits<16>),
            C {
                a: Bits<32>,
                b: Bits<8>,
            },
        }

        impl rhdl_core::Digital for Test {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(
                    "Test",
                    vec![
                        Kind::make_variant(stringify!(A), rhdl_core::Kind::Empty, 1),
                        Kind::make_variant(
                            stringify!(B),
                            rhdl_core::Kind::make_tuple(vec![
                                <Bits<16> as rhdl_core::Digital>::static_kind(),
                            ]),
                            2,
                        ),
                        Kind::make_variant(
                            stringify!(C),
                            rhdl_core::Kind::make_struct(
                                "Test::C",
                                vec![
                                    rhdl_core::Kind::make_field(
                                        stringify!(a),
                                        <Bits<32> as rhdl_core::Digital>::static_kind(),
                                    ),
                                    rhdl_core::Kind::make_field(
                                        stringify!(b),
                                        <Bits<8> as rhdl_core::Digital>::static_kind(),
                                    ),
                                ],
                            ),
                            3,
                        ),
                    ],
                    2,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::A => rhdl_bits::bits::<2usize>(0usize as u128).to_bools(),
                    Self::B(_0) => {
                        let mut v = rhdl_bits::bits::<2usize>(1usize as u128).to_bools();
                        v.extend(_0.bin());
                        v
                    }
                    Self::C { a, b } => {
                        let mut v = rhdl_bits::bits::<2usize>(2usize as u128).to_bools();
                        v.extend(a.bin());
                        v.extend(b.bin());
                        v
                    }
                })
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder,
            ) {
                builder.allocate(tag, 0);
                {
                    let builder = builder.namespace(stringify!(B));
                    <Bits<16> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(0)),
                    );
                }
                {
                    let builder = builder.namespace(stringify!(C));
                    <Bits<32> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(a)),
                    );
                    <Bits<8> as rhdl_core::Digital>::allocate(
                        tag,
                        builder.namespace(stringify!(b)),
                    );
                }
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                match self {
                    Self::A => {
                        logger.write_string(tag, stringify!(A));
                        <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                    }
                    Self::B(_0) => {
                        logger.write_string(tag, stringify!(B));
                        _0.record(tag, &mut logger);
                        <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                        <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
                    }
                    Self::C { a, b } => {
                        logger.write_string(tag, stringify!(C));
                        <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                        a.record(tag, &mut logger);
                        b.record(tag, &mut logger);
                    }
                }
            }
            fn skip<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                logger.skip(tag);
                <Bits<16> as rhdl_core::Digital>::skip(tag, &mut logger);
                <Bits<32> as rhdl_core::Digital>::skip(tag, &mut logger);
                <Bits<8> as rhdl_core::Digital>::skip(tag, &mut logger);
            }
        }
    }

    #[test]
    #[allow(dead_code)]
    fn test_derive_no_payload() {
        #[derive(Copy, Clone, PartialEq, Default)]
        pub enum State {
            #[default]
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }
        impl rhdl_core::Digital for State {
            fn static_kind() -> rhdl_core::Kind {
                Kind::make_enum(
                    "State",
                    vec![
                        Kind::make_variant(stringify!(Init), rhdl_core::Kind::Empty, 0),
                        Kind::make_variant(stringify!(Boot), rhdl_core::Kind::Empty, 1),
                        Kind::make_variant(stringify!(Running), rhdl_core::Kind::Empty, 2),
                        Kind::make_variant(stringify!(Stop), rhdl_core::Kind::Empty, 3),
                        Kind::make_variant(stringify!(Boom), rhdl_core::Kind::Empty, 4),
                    ],
                    3,
                    DiscriminantAlignment::Msb,
                )
            }
            fn bin(self) -> Vec<bool> {
                self.kind().pad(match self {
                    Self::Init => rhdl_bits::bits::<3usize>(0usize as u128).to_bools(),
                    Self::Boot => rhdl_bits::bits::<3usize>(1usize as u128).to_bools(),
                    Self::Running => rhdl_bits::bits::<3usize>(2usize as u128).to_bools(),
                    Self::Stop => rhdl_bits::bits::<3usize>(3usize as u128).to_bools(),
                    Self::Boom => rhdl_bits::bits::<3usize>(4usize as u128).to_bools(),
                })
            }
            fn allocate<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                builder: impl rhdl_core::LogBuilder,
            ) {
                builder.allocate(tag, 0);
            }
            fn record<L: rhdl_core::Digital>(
                &self,
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                match self {
                    Self::Init => {
                        logger.write_string(tag, stringify!(Init));
                    }
                    Self::Boot => {
                        logger.write_string(tag, stringify!(Boot));
                    }
                    Self::Running => {
                        logger.write_string(tag, stringify!(Running));
                    }
                    Self::Stop => {
                        logger.write_string(tag, stringify!(Stop));
                    }
                    Self::Boom => {
                        logger.write_string(tag, stringify!(Boom));
                    }
                }
            }
            fn skip<L: rhdl_core::Digital>(
                tag: rhdl_core::TagID<L>,
                mut logger: impl rhdl_core::LoggerImpl,
            ) {
                logger.skip(tag);
            }
        }
    }

    #[test]
    fn test_derive_digital_simple_struct() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
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
        let (range, kind) = bit_range(test_kind, &[Path::Field("b")]).unwrap();
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
        use rhdl_core::path::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
        enum Test {
            #[default]
            A,
            B(b2, b3),
            C {
                a: b8,
                b: b8,
            },
        }

        let foo = Test::B(b2::from(0b10), b3::from(0b101));
        let disc = vec![Path::EnumPayload(stringify!(B)), Path::Index(1)];
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
        let (disc_range, disc_kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant])?;
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

        #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
        enum Test {
            #[default]
            A,
            B(b2, b3),
            C {
                a: b8,
                b: b8,
            },
        }

        let foo_1 = Test::C {
            a: b8::from(0b10101010),
            b: b8::from(0b11010111),
        };

        println!("foo val: {}", foo_1.binary_string());

        let foo_2 = Test::B(b2::from(0b10), b3::from(0b101));

        println!("foo val: {}", foo_2.binary_string());

        let foo_3 = Test::A;

        let mut builder = basic_logger::Builder::default();
        let tag = builder.tag("test");
        let mut logger = builder.build();
        logger.set_time_in_fs(0);
        logger.log(tag, foo_1);
        logger.set_time_in_fs(1_000);
        logger.log(tag, foo_2);
        logger.set_time_in_fs(2_000);
        logger.log(tag, foo_3);
        logger.set_time_in_fs(3_000);
        logger.log(tag, foo_1);
        let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
        logger.vcd(&mut vcd_file).unwrap();
    }

    #[test]
    fn test_derive_enum_explicit_discriminant_width() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
        #[rhdl(discriminant_width = 4)]
        enum Test {
            #[default]
            A,
            B(b2, b3),
            C {
                a: b8,
                b: b8,
            },
        }

        let (range, kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant]).unwrap();
        assert_eq!(range.len(), 4);
        assert_eq!(kind, Kind::make_bits(4));
    }

    #[test]
    fn test_derive_enum_alignment_lsb() {
        use rhdl_bits::alias::*;

        #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
        #[rhdl(discriminant_align = "lsb")]
        enum Test {
            #[default]
            A,
            B(b2, b3),
            C {
                a: b8,
                b: b8,
            },
        }
        let (range, kind) = bit_range(Test::static_kind(), &[Path::EnumDiscriminant]).unwrap();
        assert_eq!(range, 0..2);
        assert_eq!(kind, Kind::make_bits(2));
    }

    #[test]
    fn test_ast_basic_func() {
        use rhdl_bits::alias::*;
        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Foo {
            a: u8,
            b: u16,
            c: [u8; 3],
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum State {
            #[default]
            Init,
            Run(u8),
            Boom,
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Bar(pub u8, pub u8);

        impl rhdl_core::digital_fn::DigitalFn for Bar {
            fn kernel_fn() -> rhdl_core::kernel::KernelFnKind {
                todo!()
            }
        }

        #[kernel]
        fn do_stuff() -> b8 {
            let a: b4 = bits::<4>(1); // Straight local assignment
            let b = !a; // Unary operator
            let c = a + (b - 1); // Binary operator
            let q = (a, b, c); // Tuple valued expression
            let (a, b, c) = q; // Tuple destructuring
            let h = Bar(1, 2); // Tuple struct literal
            let i = h.0; // Tuple struct field access
            let Bar(j, k) = h; // Tuple struct destructuring
            let d = [1, 2, 3]; // Array literal
            let d = Foo {
                a: 1,
                b: 2,
                c: [1, 2, 3],
            }; // Struct literal
            let h = {
                let e = 3;
                let f = 4;
                e + f
            }; // Statement expression
            let Foo { a, b, .. } = d; // Struct destructuring
            let g = d.c[1]; // Array indexing
            let e = d.a; // Struct field access
            let mut d: b8 = bits::<8>(7); // Mutable local
            if d > bits::<8>(0) {
                // if statement
                d = d - bits::<8>(1);
                // early return
                return d;
            }
            // if-else statement (and a statement expression)
            let j = if d < bits::<8>(3) { 7 } else { 9 };
            // Enum literal
            let k = State::Boom;
            // Enum literal with a payload
            let l = State::Run(3);
            // Match expression with enum variants
            let j = match l {
                State::Init => 1,
                State::Run(a) => 2,
                State::Boom => 3,
            };
            // For loops
            for ndx in 0..8 {
                d = d + bits::<8>(ndx);
            }
            // block expression
            bits::<8>(42)
        }

        //        let ast = do_stuff_hdl_kernel();
        //println!("{}", ast);
    }

    #[test]
    fn test_simple_type_inference() {
        #[kernel]
        fn do_stuff() {
            let k = bits::<12>(6);
            let m = bits::<14>(7);
            let c = k + 3;
            let d = if c > k { c } else { k };
            let e = (c, m);
            let (f, g) = e;
            let h = g + 1;
            let k: b4 = bits::<4>(7);
            let q = (bits::<2>(1), (bits::<5>(0), signed::<8>(5)), bits::<12>(6));
            let b = q.1 .1;
            let (q0, (q1, q1b), q2) = q; // Tuple destructuring
            let z = q1b + 4;
            let h = [d, c, f];
            let [i, j, k] = h;
            let o = j;
            let l = {
                let a = bits::<12>(3);
                let b = bits::<12>(4);
                a + b
            };
        }
        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let ctx = TypeInference::default().infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        println!("{}", ast_ascii);
    }

    #[test]
    fn test_struct_inference() {
        use rhdl_bits::alias::*;
        use rhdl_bits::bits;

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Red {
            x: b4,
            y: b6,
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Foo {
            a: b8,
            b: s4,
            c: Red,
        }

        #[kernel]
        fn do_stuff(a: Foo) -> b7 {
            let z = (a.b, a.a);
            let c = a;
            let q = signed::<4>(2);
            let c = Red {
                x: bits::<4>(1),
                y: bits::<6>(2),
            };
            let d = Foo {
                a: bits::<8>(1),
                b: q,
                c,
            };
            let Foo { a: ar, b, c: _ } = d;
            let e = ar;
            bits::<7>(42)
        }
        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        println!("{:?}", kernel);
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        gen.define_kind(Red::static_kind()).unwrap();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        println!("{}", ast_ascii);
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_rebinding() {
        #[kernel]
        fn do_stuff() {
            let q = bits::<12>(6);
            let q = bits::<16>(7);
        }

        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        println!("{:?}", kernel);
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        println!("{}", ast_ascii);
    }

    #[test]
    fn test_importing() {
        use rhdl_bits::alias::*;
        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum Red {
            #[default]
            A,
            B(b4),
            C {
                x: b4,
                y: b6,
            },
        }

        const MY_SPECIAL_NUMBER: b8 = bits(42);

        #[kernel]
        fn do_stuff() {
            let k = Red::A;
            let l = Red::B(bits::<4>(1));
            let c = Red::C {
                x: bits::<4>(1),
                y: bits::<6>(2),
            };
            let d = MY_SPECIAL_NUMBER;
        }

        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        //println!("{:?}", kernel);
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        //println!("{}", ast_ascii);
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_adt_inference() {
        use rhdl_bits::alias::*;
        use rhdl_bits::bits;
        use rhdl_std::*;

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum Red {
            #[default]
            A,
            B(b4),
            C {
                x: b4,
                y: b6,
            },
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Foo {
            a: b8,
            b: s4,
            c: Red,
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum NooState {
            #[default]
            Init,
            Run(b4, b5),
            Walk {
                foo: b5,
            },
            Boom,
        }

        #[kernel]
        fn fifo(b: b8, a: b4) -> b8 {
            b
        }

        const MY_SPECIAL_NUMBER: b8 = bits(42);

        #[kernel]
        fn do_stuff(a: Foo, s: NooState) -> b7 {
            let z = (a.b, a.a + MY_SPECIAL_NUMBER);
            let foo = bits::<12>(6);
            let foo2 = foo + foo;
            let c = a;
            let q = signed::<4>(2);
            let q = Foo {
                a: bits::<8>(1),
                b: q,
                c: Red::A,
            };
            let c = Red::A;
            let d = c;
            let z = fifo(bits::<8>(3), bits::<4>(5));
            let mut q = bits::<4>(1);
            let l = any::<4>(q);
            q = set_bit::<4>(q, 3, true);
            let p = get_bit::<4>(q, 2);
            let p = as_signed::<4>(q);
            if a.a > bits::<8>(0) {
                return bits::<7>(3);
            }
            let e = Red::B(q);
            let x1 = bits::<4>(4);
            let y1 = bits::<6>(6);
            let mut ar = [bits::<4>(1), bits::<4>(1), bits::<4>(3)];
            ar[1] = bits::<4>(2);
            let z: [Bits<4>; 3] = ar;
            let q = ar[1];
            let f: [b4; 5] = [bits::<4>(1); 5];
            let h = f[2];
            let k = NooState::Init;
            let f = Red::C { y: y1, x: x1 };
            let d = match s {
                NooState::Init => NooState::Run(bits::<4>(1), bits::<5>(2)),
                NooState::Run(x, y) => NooState::Walk { foo: y + 3 },
                NooState::Walk { foo: x } => {
                    let q = bits::<5>(1) + x;
                    NooState::Boom
                }
                NooState::Boom => NooState::Init,
                _ => NooState::Boom,
            };
            let k = 42;
            bits::<7>(k)
        }
        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        //println!("{:?}", kernel);
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        println!("{}", ast_ascii);
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_compile() {
        use rhdl_bits::alias::*;
        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Foo {
            a: u8,
            b: u16,
            c: [u8; 3],
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum NooState {
            #[default]
            Init,
            Run(u8, u8, u8),
            Walk {
                foo: u8,
            },
            Boom,
        }

        #[kernel]
        fn do_stuff(mut a: Foo, mut s: NooState) {
            let k = {
                bits::<12>(4);
                bits::<12>(6)
            };
            let mut a: Foo = a;
            let mut s: NooState = s;
            let q = if a.a > 0 {
                bits::<12>(3)
            } else {
                bits::<12>(0)
            };
            let y = bits::<12>(72);
            let t2 = (y, y);
            let q: u8 = 4;
            let z = a.c;
            let w = (a, a);
            a.c[1] = q + 3;
            a.c = [0; 3];
            a.c = [1, 2, 3];
            let q = (1, (0, 5), 6);
            let (q0, (q1, q1b), q2): (u8, (u8, u8), u16) = q; // Tuple destructuring
            a.a = 2 + 3 + q1;
            let z;
            if 1 > 3 {
                z = bits::<4>(2);
            } else {
                z = bits::<4>(5);
            }
            a.b = {
                7 + 9;
                5 + !8
            };
            a.a = if 1 > 3 {
                7
            } else {
                {
                    a.b = 1;
                    a.b = 4;
                }
                9
            };
            let g = 1 > 2;
            let h = 3 != 4;
            let mut i = g && h;
            if z == bits::<4>(3) {
                i = false;
            }
            let c = match z {
                Bits(1) => 2,
                Bits(2) => 3,
                Bits(3) => {
                    a.a = 4;
                    4
                }
                _ => 6,
            };
            let d = match s {
                NooState::Init => {
                    a.a = 1;
                    NooState::Run(1, 2, 3)
                }
                NooState::Run(x, _, y) => {
                    a.a = x + y;
                    NooState::Walk { foo: 7 }
                }
                NooState::Walk { foo: x } => {
                    a.a = x;
                    NooState::Boom
                }
                NooState::Boom => {
                    a.a = a.a + 3;
                    NooState::Init
                }
                _ => {
                    a.a = 2;
                    NooState::Boom
                }
            };
        }

        use NooState::{Init, Run};

        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");

        /*
        let mut ctx = Compiler::default();
        ctx.type_bind("a", Foo::static_kind());
        ctx.type_bind("s", NooState::static_kind());
        ctx.bind("NooState::Boom");
        ctx.bind("NooState::Init");
        ctx.bind("NooState::Run");
        ctx.bind("NooState::Walk");
        let lhs = ctx.compile(ast).unwrap();
        println!("Types before inference: {}", ctx.types_known());
        infer_type(&mut ctx).unwrap();
        println!("Types after inference: {}", ctx.types_known());
        infer_type(&mut ctx).unwrap();
        println!("Types after inference: {}", ctx.types_known());
        infer_type(&mut ctx).unwrap();
        println!("Types after inference: {}", ctx.types_known());
        infer_type(&mut ctx).unwrap();
        println!("Types after inference: {}", ctx.types_known());
        println!("Code:");
        println!("{}", ctx);
        */
    }

    #[test]
    fn test_custom_suffix() {
        #[kernel]
        fn do_stuff(mut a: b4) {
            let b = a + 1;
            let c = bits::<4>(3);
            a = b;
        }
    }

    #[test]
    fn test_match_scrutinee_bits() {
        let z = bits::<4>(0b1010);
        match z {
            rhdl_bits::Bits::<4>(0b0000) => {}
            rhdl_bits::Bits::<4>(0b0001) => {}
            _ => {}
        }
    }
    #[test]
    fn test_macro_output() {
        use rhdl_bits::alias::*;
        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Foo {
            a: u8,
            b: u16,
            c: [u8; 3],
        }

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub enum NooState {
            #[default]
            Init,
            Run(u8, u8, u8),
            Walk {
                foo: u8,
            },
            Boom,
        }

        // Define a macro called b4 that converts the argument into
        // a Bits<4> type
        fn bits4(x: u128) -> Bits<4> {
            Bits::<4>::from(x)
        }
        fn bits6(x: u128) -> Bits<6> {
            Bits::<6>::from(x)
        }

        #[kernel]
        fn do_stuff(mut a: Foo, mut s: NooState) {
            let z = bits::<6>(3);
            let c = match z {
                Bits(4) => bits::<4>(7),
                Bits(3) => bits::<4>(3),
                _ => bits::<4>(8),
            };
            let z = 1;
            let h = NooState::Run(1, z, 3);
        }
        let mut kernel: Kernel = do_stuff::kernel_fn().try_into().unwrap();
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_generics() {
        #[kernel]
        fn do_stuff<T: Digital>(a: T, b: T) -> bool {
            a == b
        }

        let mut kernel: Kernel = do_stuff::<rhdl_bits::alias::s4>::kernel_fn()
            .try_into()
            .unwrap();
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_nested_generics() {
        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        struct Foo<T: Digital> {
            a: T,
            b: T,
        }

        #[kernel]
        fn do_stuff<T: Digital, S: Digital>(x: Foo<T>, y: Foo<S>) -> bool {
            let c = x.a;
            let d = (x.a, y.b);
            let e = Foo::<T> { a: c, b: c };
            e == x
        }

        let mut kernel: Kernel =
            do_stuff::<rhdl_bits::alias::s4, rhdl_bits::alias::b3>::kernel_fn()
                .try_into()
                .unwrap();
        assign_node_ids(&mut kernel).unwrap();
        //println!("{}", kernel.ast);
        let mut gen = TypeInference::default();
        let ctx = gen.infer(&kernel).unwrap();
        let ast_code = pretty_print_kernel(&kernel, &ctx).unwrap();
        println!("{ast_code}");
    }

    #[test]
    fn test_fn_name_stuff() {
        // There are 2 namespaces (type and value).
        // A function is a value.  So this is legal:

        struct add_stuff {}

        impl add_stuff {
            fn args() -> Vec<Kind> {
                vec![Kind::make_bits(8), Kind::make_bits(8)]
            }
            fn ret() -> Kind {
                Kind::make_bits(8)
            }
        }

        fn add_stuff(a: u8, b: u8) -> u8 {
            a + b
        }

        assert_eq!(add_stuff(3_u8, 4_u8), 7_u8);
        assert_eq!(
            add_stuff::args(),
            vec![Kind::make_bits(8), Kind::make_bits(8)]
        );
        assert_eq!(add_stuff::ret(), Kind::make_bits(8));
    }

    #[test]
    fn test_fn_name_generic_stuff() {
        struct add_stuff<T: Digital> {
            _phantom: std::marker::PhantomData<T>,
        }

        impl<T: Digital> add_stuff<T> {
            fn args() -> Vec<Kind> {
                vec![T::static_kind(), T::static_kind()]
            }
            fn ret() -> Kind {
                T::static_kind()
            }
        }

        fn add_stuff<T: Digital>(a: T, b: T) -> T {
            b
        }

        assert_eq!(add_stuff::<b4>(3.into(), 4.into()), bits(4));
        assert_eq!(
            add_stuff::<b4>::args(),
            vec![Kind::make_bits(4), Kind::make_bits(4)]
        );
        assert_eq!(add_stuff::<b4>::ret(), Kind::make_bits(4));
    }

    #[test]
    fn test_enum_constructor_function() {
        // Start with a variant that has a tuple struct argument.
        enum Color {
            Red(u8),
        }

        struct Foo {}

        impl Foo {
            fn args() -> Vec<Kind> {
                vec![Kind::make_bits(8)]
            }
            fn ret() -> Kind {
                Kind::make_enum(
                    "Color",
                    vec![Kind::make_variant(
                        "Red",
                        Kind::make_tuple(vec![Kind::make_bits(8)]),
                        1,
                    )],
                    1,
                    DiscriminantAlignment::Msb,
                )
            }
        }

        trait TypeName {
            fn type_name() -> String;
        }

        impl TypeName for usize {
            fn type_name() -> String {
                "usize".into()
            }
        }

        impl TypeName for String {
            fn type_name() -> String {
                "String".into()
            }
        }

        impl<T: TypeName> TypeName for Vec<T> {
            fn type_name() -> String {
                format!("Vec<{}>", T::type_name())
            }
        }

        impl TypeName for () {
            fn type_name() -> String {
                "Unit".into()
            }
        }

        impl<T1: TypeName, T2: TypeName> TypeName for (T1, T2) {
            fn type_name() -> String {
                format!("({}, {})", T1::type_name(), T2::type_name())
            }
        }

        impl TypeName for u8 {
            fn type_name() -> String {
                "u8".into()
            }
        }

        impl TypeName for Color {
            fn type_name() -> String {
                "Color".into()
            }
        }

        fn inspect_function<F, T1, T2>(_f: F) -> String
        where
            F: Fn(T1) -> T2,
            T1: TypeName,
            T2: TypeName,
        {
            format!("Function: {} -> {}", T1::type_name(), T2::type_name())
        }

        eprintln!("{}", inspect_function(Color::Red));

        // See: https://jsdw.me/posts/rust-fn-traits/
    }

    #[test]
    fn test_module_isolation_idea() {
        mod demo {
            use rhdl_bits::alias::*;
            use rhdl_macro::Digital;

            #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
            pub struct Foo {
                pub a: u8,
                pub b: NooState,
                pub c: [u8; 3],
            }

            #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
            pub enum NooState {
                #[default]
                Init,
                Run(u8, u8, u8),
                Walk {
                    foo: u8,
                },
                Boom,
            }

            mod private {
                use super::Foo;
                use super::NooState;
                use rhdl_bits::bits;
                use rhdl_bits::Bits;
                pub fn do_stuff(mut a: Foo) -> Foo {
                    let z = bits::<6>(3);
                    let c = match z {
                        Bits(4) => bits(7),
                        Bits(3) => bits::<4>(3),
                        _ => bits(8),
                    };
                    a.b = NooState::Boom;
                    a
                }
            }
            pub use private::do_stuff;
        }
        use demo::do_stuff;
        use demo::Foo;
        use demo::NooState;

        let a = Foo {
            a: 1,
            b: NooState::Init,
            c: [1, 2, 3],
        };
        let b = do_stuff(a);
        assert_eq!(b.b, NooState::Boom);
    }
}
