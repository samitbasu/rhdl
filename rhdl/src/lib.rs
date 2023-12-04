pub mod bits;
pub mod core;

pub use crate::bits::Bits;
pub use crate::core::Digital;
pub use crate::core::Kind;
pub use crate::core::TagID;
pub use rhdl_macro::kernel;
pub use rhdl_macro::Digital;

#[cfg(test)]
mod tests {

    use rhdl_bits::{alias::*, bits, signed};
    use rhdl_core::{
        ascii::render_ast_to_string,
        assign_node::assign_node_ids,
        check_inference,
        compiler::Compiler,
        compiler_gen2::CompilerContext,
        digital_fn::{inspect_digital, DigitalFn},
        display_ast::pretty_print_kernel,
        infer_types::{infer, TypeInference},
        kernel::{ExternalKernelDef, Kernel, KernelFnKind},
        note,
        note_db::{dump_vcd, note_time},
        path::{bit_range, Path},
        typer::infer_type,
        visit::Visitor,
        DiscriminantAlignment, NoteKey,
    };

    use super::*;

    fn test_inference_result(kernel: KernelFnKind) -> anyhow::Result<()> {
        let mut kernel: Kernel = kernel.try_into()?;
        assign_node_ids(&mut kernel)?;
        let ctx = infer(&kernel)?;
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        eprintln!("{}", ast_ascii);
        check_inference(&kernel, &ctx)?;
        Ok(())
    }

    #[test]
    fn test_vcd_enum() {
        #[derive(Clone, Copy, Debug, PartialEq, Default, Digital)]
        enum Enum {
            #[default]
            None,
            A(u8, u16),
            B {
                name: u8,
            },
            C(bool),
        }

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
        dump_vcd(&[], &mut vcd_file).unwrap();
    }

    #[test]
    fn test_vcd_basic() {
        #[derive(Clone, Copy, PartialEq, Default, Digital)]
        pub struct Simple {
            a: bool,
            b: Bits<8>,
        }

        let simple = Simple {
            a: true,
            b: Bits::from(0b10101010),
        };
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
        dump_vcd(&[], &mut vcd_file).unwrap();
    }

    #[test]
    #[allow(dead_code)]
    #[allow(clippy::just_underscores_and_digits)]
    fn test_derive() {
        #[derive(Clone, Copy, PartialEq, Default, Digital)]
        enum Test {
            #[default]
            A,
            B(Bits<16>),
            C {
                a: Bits<32>,
                b: Bits<8>,
            },
        }
        note("test", Test::A);
    }

    #[test]
    #[allow(dead_code)]
    fn test_derive_no_payload() {
        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        pub enum State {
            #[default]
            Init,
            Boot,
            Running,
            Stop,
            Boom,
        }
        note("state", State::Running);
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

        note_time(0);
        note("test", foo_1);
        note_time(1_000);
        note("test", foo_2);
        note_time(2_000);
        note("test", foo_3);
        note_time(3_000);
        note("test", foo_1);
        let mut vcd_file = std::fs::File::create("test_enum.vcd").unwrap();
        dump_vcd(&[], &mut vcd_file).unwrap();
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

        #[kernel]
        fn do_stuff() -> b8 {
            let a = b4(1); // Straight local assignment
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
                b8(e) + b8(f)
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
                State::Init => b3(1),
                State::Run(a) => b3(2),
                State::Boom => b3(3),
            };
            // For loops
            for ndx in 0..8 {
                d = d + bits::<8>(ndx);
            }
            // block expression
            bits::<8>(42)
        }

        test_inference_result(do_stuff::kernel_fn()).unwrap();
    }

    #[test]
    fn test_method_call_syntax() {
        use rhdl_std::UnsignedMethods;

        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        struct Baz {
            a: u8,
        }

        impl Baz {
            fn any(&self) -> bool {
                false
            }
        }

        #[kernel]
        fn do_stuff() {
            let k = b12(5);
            let h = Baz { a: 3 };
            let j = k.any();
            let i = k.get_bit(3);
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
    }

    #[test]
    fn test_method_call_fails_with_roll_your_own() {
        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        struct Baz {
            a: u8,
        }

        impl Baz {
            fn any(&self) -> bool {
                false
            }
        }

        #[kernel]
        fn do_stuff() {
            let k = b12(5);
            let h = Baz { a: 3 };
            let j = h.any();
        }
        assert!(test_inference_result(do_stuff::kernel_fn()).is_err());
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
                let a = b12(3);
                let b = bits::<12>(4);
                a + b
            };
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
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

        #[derive(PartialEq, Copy, Clone, Digital, Default)]
        pub struct Bar(pub u8, pub u8);

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
            let q = Bar(1, 2);
            let x = NooState::Run(bits::<4>(1), bits::<5>(2));
            let e = ar;
            bits::<7>(42)
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
    }

    #[test]
    fn test_rebinding() {
        #[kernel]
        fn do_stuff() {
            let q = bits::<12>(6);
            let q = bits::<16>(7);
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
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
        test_inference_result(do_stuff::kernel_fn()).unwrap();
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
        test_inference_result(do_stuff::kernel_fn()).unwrap();
    }

    #[test]
    fn test_adt_shadow() {
        use rhdl_bits::alias::*;

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
        fn do_stuff(mut s: NooState) {
            let y = bits::<12>(72);
            let foo = bits::<14>(32);
            let mut a: u8 = 0;
            let d = match s {
                NooState::Init => {
                    a = 1;
                    NooState::Run(1, 2, 3)
                }
                NooState::Walk { foo: x } => {
                    a = x;
                    NooState::Boom
                }
                NooState::Run(x, _, y) => {
                    a = x + y;
                    NooState::Walk { foo: 7 }
                }
                NooState::Boom => {
                    a = a + 3;
                    NooState::Init
                }
                _ => {
                    a = 2;
                    NooState::Boom
                }
            };
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
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

        const CONST_PATH: b4 = bits(4);
        #[kernel]
        fn do_stuff(mut a: Foo, mut s: NooState) {
            let k = {
                bits::<12>(4) + 6;
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
            match a {
                Foo {
                    a: 1,
                    b: 2,
                    c: [1, 2, 3],
                } => {}
                Foo {
                    a: 3,
                    b: 4,
                    c: [1, 2, 3],
                } => {}
                _ => {}
            }
            let c = bits::<4>(match z {
                CONST_PATH => 1,
                Bits::<4>(1) => 2,
                Bits::<4>(2) => 3,
                Bits::<4>(3) => {
                    a.a = 4;
                    4
                }
                _ => 6,
            });
            let d = match s {
                NooState::Init => {
                    a.a = 1;
                    NooState::Run(1, 2, 3)
                }
                NooState::Walk { foo: x } => {
                    a.a = x;
                    NooState::Boom
                }
                NooState::Run(x, _, y) => {
                    a.a = x + y;
                    NooState::Walk { foo: 7 }
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
        test_inference_result(do_stuff::kernel_fn()).unwrap();
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

        #[derive(PartialEq, Copy, Clone, Default, Digital)]
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
                Bits::<6>(4) => bits::<4>(7),
                Bits::<6>(3) => bits::<4>(3),
                _ => bits::<4>(8),
            };
            let z = 1;
            let h = NooState::Run(1, z, 3);
        }
        test_inference_result(do_stuff::kernel_fn()).unwrap();
    }

    #[test]
    fn test_generics() {
        #[kernel]
        fn do_stuff<T: Digital>(a: T, b: T) -> bool {
            a == b
        }

        test_inference_result(do_stuff::<rhdl_bits::alias::s4>::kernel_fn()).unwrap();
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

        test_inference_result(do_stuff::<rhdl_bits::alias::s4, rhdl_bits::alias::b3>::kernel_fn())
            .unwrap()
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
    fn test_signature_for_associated_functions() {
        fn add(a: u8, b: u8) -> u8 {
            todo!()
        }

        let sig = inspect_digital(add);
        println!("{:?}", sig);

        // Suppose we have a method call syntax.
        // Such as y = a.any();
        // One way to support this is to transform
        // it into
        //  y = any(a);
        // and then use the existing function call
        // syntax.  The problem is the polymorphism
        // of the function call.  For example, the `any` method
        // _could_ exist on both signed and unsigned bit vectors.
        // But we can only take one of them as an argument to
        // the `any` function.  Unless we make it generic over
        // the type of the argument.
        //
        // Another option is to make the methods trait methods
        // on the Digital trait itself.
        //
        // That would allow (expr).any(), for example, since
        // .any() was by definition part of the Digital trait,
        // and any expr must impl Digital.  We could then rewrite
        // it as any(expr: impl Digital) -> bool.
        //
        // For performance, we can provide custom implementations
        // for signed and unsigned bit vecs.
        //
        // Unfortunately, this does not really make sense.  What does
        // (enum).all() mean?
        //
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

    #[test]
    fn test_basic_compile() {
        #[kernel]
        fn add(a: b4, b: b4) -> b4 {
            let c = a;
            a + b + c
        }

        let mut kernel: Kernel = add::kernel_fn().try_into().unwrap();
        assign_node_ids(&mut kernel).unwrap();
        let ctx = infer(&kernel).unwrap();
        let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
        eprintln!("{}", ast_ascii);
        check_inference(&kernel, &ctx).unwrap();
        let mut compiler = CompilerContext::new(ctx);
        compiler.visit_kernel_fn(&kernel.ast).unwrap();
        eprintln!("{}", compiler);
    }
}
