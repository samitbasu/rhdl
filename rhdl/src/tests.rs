#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]
use itertools::iproduct;
use rand::Rng;
use rhdl_bits::{alias::*, bits, signed, Bits, SignedBits};
use rhdl_core::{
    compile_design,
    compiler::{mir_pass::compile_mir, mir_type_infer::infer},
    digital_fn::DigitalFn,
    note,
    note_db::note_time,
    note_init_db, note_take,
    path::{bit_range, Path},
    rhif::vm::execute_function,
    test_kernel_vm_and_verilog,
    types::{
        clock::{Green, Red},
        timed::signal,
    },
    Clock, Digital, KernelFnKind, Kind, Sig,
};
use rhdl_macro::{kernel, Digital};
use rhdl_std::UnsignedMethods;

#[test]
fn test_vcd_enum() {
    #[derive(Clone, Copy, Debug, PartialEq, Digital)]
    enum Enum {
        None,
        A(u8, u16),
        B { name: u8 },
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
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
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
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
#[allow(dead_code)]
#[allow(clippy::just_underscores_and_digits)]
fn test_derive() {
    #[derive(Clone, Copy, PartialEq, Digital)]
    enum Test {
        A,
        B(Bits<16>),
        C { a: Bits<32>, b: Bits<8> },
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
    use rhdl_core::path::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    enum Test {
        A,
        B(b2, b3),
        C { a: b8, b: b8 },
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
        C { a: b8, b: b8 },
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
        C { a: b8, b: b8 },
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
        C { a: b8, b: b8 },
    }
    let (range, kind) = bit_range(Test::static_kind(), &Path::default().discriminant()).unwrap();
    assert_eq!(range, 0..2);
    assert_eq!(kind, Kind::make_bits(2));
}

#[test]
fn test_struct_expr_not_adt() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[kernel]
    fn do_stuff(a: u8) -> Foo {
        let d = Foo {
            a,
            b: 2,
            c: [1, 2, 3],
        }; // Struct literal
        d
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_u8()).unwrap();
}

#[test]
fn test_adt_use() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Foo {
        Red(u8, bool),
        Green(u8, bool),
    }

    impl Default for Foo {
        fn default() -> Self {
            Foo::Red(0, false)
        }
    }

    #[kernel]
    fn get_color(a: Foo, c: bool) -> bool {
        c && match a {
            Foo::Red(x, z) => z,
            Foo::Green(x, z) => true,
        }
    }

    test_kernel_vm_and_verilog::<get_color, _, _, _>(
        get_color,
        [(Foo::Red(3, true), false), (Foo::Green(4, true), true)]
            .iter()
            .cloned(),
    )
    .unwrap();
}

#[test]
fn test_struct_expr_adt() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Foo {
        A,
        B(u8),
        C { a: u8, b: u16 },
    }

    #[kernel]
    fn do_stuff(a: u8) -> Foo {
        if a < 10 {
            Foo::A
        } else if a < 20 {
            Foo::B(a)
        } else {
            Foo::C { a, b: 0 }
        }
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_u8()).unwrap();
}

#[test]
fn test_phi_if_consts() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let j = if a.any() { 3 } else { 7 };
        bits::<8>(j)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_phi_if_consts_inferred_len() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let j = if a.any() { 3 } else { 7 };
        bits(j)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_func_with_structured_args() {
    #[kernel]
    fn do_stuff((a, b): (b8, b8)) -> b8 {
        let c = (a, b);
        let d = c.0;
        a + b
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, [((b8(0), b8(3)),)].into_iter())
        .unwrap();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func_inferred_bits() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum State {
        Init,
        Run(u8),
        Boom,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[kernel]
    fn do_stuff(arg: b4) -> b8 {
        let a = arg; // Straight local assignment
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
        let p = Foo { a: 4, ..d };
        let h = {
            let e = 3;
            let f = 4;
            b8(e) + b8(f)
        }; // Statement expression
        let Foo { a, b, .. } = d; // Struct destructuring
        let g = d.c[1]; // Array indexing
        let e = d.a; // Struct field access
        let mut d: b8 = bits(7); // Mutable local
        if d > bits(0) {
            // if statement
            d = d - bits(1);
            // early return
            return d;
        }
        // if-else statement (and a statement expression)
        let j = if d < bits(3) { 7 } else { 9 };
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
            d = d + bits(ndx);
        }
        // block expression
        bits(42)
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum State {
        Init,
        Run(u8),
        Boom,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[kernel]
    fn do_stuff(arg: b4) -> b8 {
        let a = arg; // Straight local assignment
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
        let p = Foo { a: 4, ..d };
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

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_method_call_syntax() {
    use rhdl_std::UnsignedMethods;

    #[kernel]
    fn do_stuff(a: b8) -> (bool, bool, bool, s8) {
        let any = a.any();
        let all = a.all();
        let xor = a.xor();
        let s = a.as_signed();
        (any, all, xor, s)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_method_call_fails_with_roll_your_own() {
    #[derive(Copy, Clone, PartialEq, Digital)]
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

    let Some(KernelFnKind::Kernel(kernel)) = do_stuff::kernel_fn() else {
        panic!("Kernel not found");
    };
    assert!(compile_design::<do_stuff>().is_err());
}

#[test]
fn test_signal_const_binop_inference() -> anyhow::Result<()> {
    #[kernel]
    fn do_stuff<C: Clock>(a: Sig<b8, C>) -> Sig<b8, C> {
        a + b8(4)
    }
    compile_design::<do_stuff<Red>>()?;
    Ok(())
}

#[test]
fn test_bits_inference_with_type() {
    #[kernel]
    fn do_stuff(a: b8) -> b8 {
        let y: b8 = bits(3);
        let r = 3;
        let z = y << r;
        a
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_signal_cross_clock_select_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock, D: Clock>(x: Sig<b8, C>, y: Sig<b8, D>) -> Sig<b8, C> {
        if y.val().any() {
            x
        } else {
            x + 2
        }
    }
    assert!(compile_design::<add::<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cast_works() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock>(x: Sig<b8, C>, y: Sig<b8, C>) -> Sig<b8, C> {
        let z = x + y;
        signal::<b8, C>(z.val())
    }
    let obj = compile_design::<add<Red>>()?;
    eprintln!("{:?}", obj);
    Ok(())
}

#[test]
fn test_signal_cast_cross_clocks_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock, D: Clock>(x: Sig<b8, C>) -> Sig<b8, D> {
        signal(x.val() + 3)
    }
    assert!(compile_design::<add<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_shifting_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock, D: Clock>(x: Sig<b8, C>) -> Sig<b8, D> {
        let p = 4;
        let y: b8 = bits(7);
        let z = y << p;
        signal(x.val() << z)
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_indexing_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock, D: Clock>(x: Sig<[b8; 8], C>, y: Sig<b3, D>) -> Sig<b8, C> {
        let z = x[y.val()];
        signal(z)
    }
    assert!(compile_design::<add::<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_ops_inference() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Clock, D: Clock>(
        x: Sig<b8, C>,
        y: Sig<b8, C>,
        z: Sig<b8, D>,
        w: Sig<b8, D>,
        ndx: b8,
    ) -> Sig<b8, D> {
        // c, x, y are C
        let c = x + y;
        // d is C
        let d = x > y;
        // bx is C
        let bx = x.val();
        // zz is C
        let zz = 2 < bx;
        // e is C
        let e = d && (!d ^ d);
        // q is D
        let q = z > w;
        // x is C
        let x = [c, c, c];
        // z2 is C
        let z2 = x[ndx];
        // res is D
        let res = if q { w } else { z };
        // h is D
        let h = z.val();
        // qq is Illegal!
        let qq = h + y.val();
        match (z + 1).val() {
            Bits::<8>(0) => z,
            _ => w,
        }
    }
    assert!(compile_design::<add::<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_simple_type_inference() {
    #[kernel]
    fn do_stuff(a: b12) -> b12 {
        let k = a;
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
            let b = bits(4);
            a + b
        };
        l + k
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_struct_inference_inferred_lengths() {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Red {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Red,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn do_stuff(a: Foo) -> (b8, b8, NooState, Foo) {
        let z = (a.b, a.a);
        let c = a;
        let q = signed(-2);
        let c = Red {
            x: bits(1),
            y: bits(2),
        };
        let d = Foo {
            a: bits(1),
            b: q,
            c,
        };
        let Foo { a: ar, b, c: _ } = d;
        let q = Bar(1, 2);
        let x = NooState::Run(bits(1), bits(2));
        let e = ar;
        (e, ar, x, d)
    }
    let inputs = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter().map(|x| (x,)))
        .unwrap();
}

#[test]
fn test_struct_inference() {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Red {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Red,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn do_stuff(a: Foo) -> (b8, b8, NooState, Foo) {
        let z = (a.b, a.a);
        let c = a;
        let q = signed::<4>(-2);
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
        (e, ar, x, d)
    }
    let inputs = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter().map(|x| (x,)))
        .unwrap();
}

#[test]
#[allow(clippy::let_and_return)]
fn test_rebinding() {
    #[kernel]
    fn do_stuff(a: b8) -> b16 {
        let q = a;
        let q = bits::<12>(6);
        let q = bits::<16>(7);
        q
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_missing_register_inferred_types() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let mut c = bits(0);
        match a {
            Bits::<1>(0) => c = bits(2),
            Bits::<1>(1) => c = bits(3),
            _ => {}
        }
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_missing_register() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let mut c = bits::<8>(0);
        match a {
            Bits::<1>(0) => c = bits::<8>(2),
            Bits::<1>(1) => c = bits::<8>(3),
            _ => {}
        }
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_phi_missing_register_signed_inference() {
    #[kernel]
    fn do_stuff(a: b1) -> s8 {
        let mut c = signed(0);
        match a {
            Bits::<1>(0) => c = signed(2),
            Bits::<1>(1) => c = signed(3),
            _ => {}
        }
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_phi_missing_register() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let mut c = bits::<8>(0);
        if a.any() {
            c = bits::<8>(1);
        }
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_phi_inferred_lengths() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let mut c = bits(0);
        /*
        match a {
            Bits::<1>(0) => c = bits(2),
            Bits::<1>(1) => c = bits(3),
            _ => {}
        }
        */
        let d = c;
        if a.any() {
            //            c = bits(1);
            c = bits(2);
        } else {
            //c = bits(3);
            c = bits(4);
            /*             if a.all() {
                c = bits(5);
            }*/
        }
        let y = c;
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_phi() {
    #[kernel]
    fn do_stuff(a: b1) -> b8 {
        let mut c = bits::<8>(0);
        match a {
            Bits::<1>(0) => c = bits::<8>(2),
            Bits::<1>(1) => c = bits::<8>(3),
            _ => {}
        }
        let d = c;
        if a.any() {
            c = bits::<8>(1);
            c = bits::<8>(2);
        } else {
            c = bits::<8>(3);
            c = bits::<8>(4);
            if a.all() {
                c = bits::<8>(5);
            }
        }
        let y = c;
        c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ssa() {
    #[kernel]
    fn do_stuff(a: b8) -> b8 {
        let mut q = a;
        q = q + a;
        q = a;
        q
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_importing() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Red {
        A,
        B(b4),
        C { x: b4, y: b6 },
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff(a: b4) -> (Red, Red, Red, b8) {
        let k = Red::A;
        let l = Red::B(bits::<4>(1));
        let c = Red::C {
            x: bits::<4>(1),
            y: bits::<6>(2),
        };
        let d = MY_SPECIAL_NUMBER;
        (k, l, c, d)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_adt_inference_subset() {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Red {
        A,
        B(b4),
        C { x: b4, y: b6 },
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Red,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn fifo(b: b8, a: b4) -> b8 {
        b
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff(a: Foo, s: NooState) -> (NooState, b7) {
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
        (NooState::Init, bits::<7>(3))
    }

    let foos = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::A,
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::B(bits::<4>(1)),
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::C {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits::<4>(1), bits::<5>(2)),
        NooState::Walk { foo: bits::<5>(3) },
    ];
    let inputs = iproduct!(foos.into_iter(), noos.into_iter()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter()).unwrap();
}

#[test]
fn test_adt_inference() {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;
    use rhdl_std::*;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Red {
        A,
        B(b4),
        C { x: b4, y: b6 },
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Red,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn fifo(b: b8, a: b4) -> b8 {
        b
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff(a: Foo, s: NooState) -> (NooState, b7) {
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
        if a.a > bits::<8>(12) {
            return (NooState::Boom, bits::<7>(3));
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
        };
        let k = 42;
        (d, bits::<7>(k))
    }

    let foos = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::A,
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::B(bits::<4>(1)),
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Red::C {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits::<4>(1), bits::<5>(2)),
        NooState::Walk { foo: bits::<5>(3) },
    ];
    let inputs = iproduct!(foos.into_iter(), noos.into_iter()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter()).unwrap();
}

#[test]
fn test_adt_shadow() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(u8, u8, u8),
        Walk { foo: u8 },
        Boom,
    }

    #[kernel]
    fn do_stuff(mut s: NooState) -> (u8, NooState) {
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
        };
        (a, d)
    }
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, noos.into_iter().map(|x| (x,)))
        .unwrap();
}

#[test]
fn test_compile() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(u8, u8, u8),
        Walk { foo: u8 },
        Boom,
    }

    const CONST_PATH: b4 = bits(4);
    #[kernel]
    fn do_stuff(mut a: Foo, mut s: NooState) -> Foo {
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
            NooState::Run(x, t, y) => {
                a.a = x + y;
                NooState::Walk { foo: 7 }
            }
            NooState::Boom => {
                a.a += 3;
                NooState::Init
            }
        };
        a
    }
    let foos = [
        Foo {
            a: 1,
            b: 2,
            c: [1, 2, 3],
        },
        Foo {
            a: 4,
            b: 5,
            c: [4, 5, 6],
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    let inputs = iproduct!(foos.into_iter(), noos.into_iter()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter()).unwrap();
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
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(u8, u8, u8),
        Walk { foo: u8 },
        Boom,
    }

    #[kernel]
    fn do_stuff(mut a: Foo, mut s: NooState) -> NooState {
        let z = bits::<6>(3);
        let c = match z {
            Bits::<6>(4) => bits::<4>(7),
            Bits::<6>(3) => bits::<4>(3),
            _ => bits::<4>(8),
        };
        let z = 1;
        let h = NooState::Run(1, z, 3);
        h
    }

    let foos = [
        Foo {
            a: 1,
            b: 2,
            c: [1, 2, 3],
        },
        Foo {
            a: 4,
            b: 5,
            c: [4, 5, 6],
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    let inputs = iproduct!(foos.into_iter(), noos.into_iter()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter()).unwrap();
}

#[test]
fn test_generics() {
    #[kernel]
    fn do_stuff<T: Digital>(a: T, b: T) -> bool {
        a == b
    }

    let a = [
        signed::<4>(1),
        signed::<4>(2),
        signed::<4>(3),
        signed::<4>(-1),
        signed::<4>(-3),
    ];
    let inputs = iproduct!(a.iter().cloned(), a.iter().cloned()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4>, _, _, _>(do_stuff, inputs.into_iter()).unwrap();
}

#[test]
fn test_bit_inference_works() {
    #[kernel]
    fn do_stuff(a: b8) -> b8 {
        let b = a + 1;
        let c = bits(3);
        b + c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_nested_generics() {
    #[derive(PartialEq, Copy, Clone, Digital)]
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

    let a = [
        signed::<4>(1),
        signed::<4>(2),
        signed::<4>(3),
        signed::<4>(-1),
        signed::<4>(-3),
    ];
    let b: Vec<b3> = exhaustive();
    let inputs = iproduct!(
        a.into_iter().map(|x| Foo { a: x, b: x }),
        b.into_iter().map(|x| Foo { a: x, b: x })
    )
    .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, b3>, _, _, _>(do_stuff::<s4, b3>, inputs.into_iter())
        .unwrap();
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
fn test_for_loop() {
    #[kernel]
    fn looper(a: b8) -> bool {
        let mut ret: bool = false;
        for i in 0..8 {
            ret ^= rhdl_std::get_bit::<8>(a, i);
        }
        ret
    }

    test_kernel_vm_and_verilog::<looper, _, _, _>(looper, tuple_exhaustive()).unwrap();
}

#[test]
fn test_rebind_compile() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub enum SimpleEnum {
        Init,
        Run(u8),
        Point { x: b4, y: u8 },
        Boom,
    }

    const B6: b6 = bits(6);

    #[kernel]
    fn add(state: SimpleEnum) -> u8 {
        let x = state;
        match x {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        }
    }

    let inputs = [
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(5),
        SimpleEnum::Point { x: bits(7), y: 11 },
        SimpleEnum::Point { x: bits(7), y: 13 },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(|x| (x,))).unwrap();
}

#[test]
fn test_basic_compile() {
    use itertools::iproduct;

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: b4,
        b: b4,
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct TupStruct(b4, b4);

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub enum Bar {
        A,
        B(b4),
        C { x: b4, y: b4 },
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub enum SimpleEnum {
        Init,
        Run(u8),
        Point { x: b4, y: u8 },
        Boom,
    }

    #[kernel]
    fn nib_add(a: b4, b: b4) -> b4 {
        a + b
    }

    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);
    const MOMO: u8 = 15;

    #[kernel]
    fn add(mut a: b4, b: [b4; 3], state: SimpleEnum) -> b4 {
        let (d, c) = (1, 3);
        let p = a + c;
        let q = p;
        let q = b[2];
        let p = [q; 3];
        let k = (q, q, q, q);
        let mut p = k.2;
        if p > 2 {
            return p;
        }
        p = a - 1;
        let mut q = Foo { a: a, b: b[2] };
        let Foo { a: x, b: y } = q;
        q.a += p;
        let mut bb = b;
        bb[2] = p;
        let z: b4 = p + nib_add(x, y);
        let q = TupStruct(x, y);
        let TupStruct(x, y) = q;
        let h = Bar::A;
        let h = Bar::B(p);
        let h = Bar::C { x: p, y: p };
        let k: Bar = Bar::A;
        match x {
            ONE => {}
            TWO => {}
            Bits::<4>(3) => {}
            _ => {}
        }
        let count = match state {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        };
        a + c + z
    }

    let a_set = exhaustive();
    let b_set: Vec<[b4; 3]> = iproduct!(a_set.iter(), a_set.iter(), a_set.iter())
        .map(|x| [*x.0, *x.1, *x.2])
        .collect();
    let state_set = [
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(5),
        SimpleEnum::Point { x: bits(7), y: 11 },
        SimpleEnum::Point { x: bits(7), y: 13 },
        SimpleEnum::Boom,
    ];
    let inputs =
        iproduct!(a_set.into_iter(), b_set.into_iter(), state_set.into_iter()).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter()).unwrap();
}

#[test]
fn test_enum_match() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub enum SimpleEnum {
        Init,
        Run(u8),
        Point { x: b4, y: u8 },
        Boom,
    }

    #[kernel]
    fn add(state: SimpleEnum) -> u8 {
        let x = state;
        match x {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        }
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(2),
        SimpleEnum::Run(3),
        SimpleEnum::Point { x: bits(1), y: 2 },
        SimpleEnum::Point { x: bits(1), y: 9 },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, samples.into_iter().map(|x| (x,))).unwrap();
}

#[test]
fn test_enum_match_signed_discriminant() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    #[rhdl(discriminant_width = 4)]
    #[repr(i8)]
    pub enum SimpleEnum {
        Init = 1,
        Run(u8) = 2,
        Point { x: b4, y: u8 } = 3,
        Boom = -2,
    }

    #[kernel]
    fn add(state: SimpleEnum) -> u8 {
        let x = state;
        match x {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        }
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(2),
        SimpleEnum::Run(3),
        SimpleEnum::Point { x: bits(1), y: 2 },
        SimpleEnum::Point { x: bits(1), y: 9 },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, samples.into_iter().map(|x| (x,))).unwrap();
}

#[test]
fn test_const_literal_match() {
    #[kernel]
    fn add(a: u8) -> u8 {
        match a {
            1 => 1,
            2 => 2,
            _ => 3,
        }
    }
    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap();
}

#[test]
fn test_const_literal_captured_match() {
    const ZERO: b4 = bits(0);
    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);

    #[kernel]
    fn do_stuff(a: b4) -> b4 {
        match a {
            ONE => TWO,
            TWO => ONE,
            _ => ZERO,
        }
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive()).unwrap();
}

#[test]
fn test_struct_literal_match() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: u8,
        b: u8,
    }

    #[kernel]
    fn add(a: Foo) -> u8 {
        match a {
            Foo { a: 1, b: 2 } => 1,
            Foo { a: 3, b: 4 } => 2,
            _ => 3,
        }
    }

    let test_vec = (0..4)
        .flat_map(|a| (0..4).map(move |b| (Foo { a, b },)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter()).unwrap();
}

#[test]
fn test_nested_tuple_init() {
    #[kernel]
    fn add(a: u8) -> u8 {
        let b = (1, (2, 3), 4);
        let (c, (d, e), f) = b;
        c + d + e + f
    }

    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap();
}

#[test]
fn test_nested_tuple_array_init() {
    #[kernel]
    fn add(a: u8) -> u8 {
        let b = [(1, (2, 3), 4); 3];
        let (c, (d, e), f) = b[1];
        let [g, (h0, (h1a, h1b), h2), i] = b;
        c + d + e + f + g.0 + h0 + h1a + h1b + h2 + i.1 .0
    }

    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap();
}

#[test]
fn test_tuple_destructure_in_args() {
    #[kernel]
    fn add((b, c): (u8, u8)) -> u8 {
        b + c
    }

    let test_vec = (0..4)
        .flat_map(|a| (0..4).map(move |b| ((a, b),)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter()).unwrap();
}

#[test]
fn test_tuple_struct_nested_init() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: u8,
        b: u8,
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Bar {
        a: u8,
        b: Foo,
    }

    #[kernel]
    fn add(a: u8) -> u8 {
        let b = Bar {
            a: 1,
            b: Foo { a: 2, b: 3 },
        };
        let Bar {
            a,
            b: Foo { a: c, b: d },
        } = b;
        a + c + d
    }

    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap()
}

#[test]
fn test_tuplestruct_nested_init() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Wrap(u8, (u8, u8), u8);

    #[kernel]
    fn add(a: u8) -> u8 {
        let b = Wrap(1, (2, 3), 4);
        let Wrap(c, (d, e), f) = b;
        c + d + e + f
    }
    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap()
}

#[test]
fn test_link_to_bits_fn() {
    use rhdl_std::UnsignedMethods;

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    struct Tuplo(b4, s6);

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    enum NooState {
        Init,
        Run(b4, s6),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn add_two(a: b4) -> b4 {
        a + 2
    }

    #[kernel]
    fn add_one(a: b4) -> b4 {
        add_two(a)
    }

    #[kernel]
    fn add(a: b4) -> b4 {
        let b = bits::<4>(3);
        let d = signed::<6>(11);
        let c = b + a;
        let k = c.any();
        let h = Tuplo(c, d);
        let p = h.0;
        let q = NooState::Run(c, d);
        c + add_one(p)
    }

    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_exhaustive()).unwrap();
}

#[test]
fn test_vm_simple_function() {
    #[kernel]
    fn pass(a: b8) -> b8 {
        a
    }

    test_kernel_vm_and_verilog::<pass, _, _, _>(pass, tuple_exhaustive()).unwrap();
}

#[test]
fn test_vm_simple_function_with_invalid_args_causes_ice() {
    #[kernel]
    fn pass(a: u8) -> u8 {
        a
    }
    let design = compile_design::<pass>().unwrap();
    eprintln!("design: {}", design);
    let res = execute_function(&design, vec![(42_u16).typed_bits()]);
    assert!(res.is_err());
}

#[test]
fn test_vm_simple_binop_function() {
    #[kernel]
    fn add(a: b12, b: b12) -> b12 {
        a + b + b
    }

    let tests = [
        (bits(3), bits(17)),
        (bits(1), bits(42)),
        (bits(1000), bits(32)),
    ];

    test_kernel_vm_and_verilog::<add, _, _, _>(add, tests.into_iter()).unwrap();
}

fn exhaustive<const N: usize>() -> Vec<Bits<N>> {
    (0..(1 << N)).map(bits).collect()
}

fn tuple_exhaustive<const N: usize>() -> impl Iterator<Item = (Bits<N>,)> + Clone {
    exhaustive::<N>().into_iter().map(|x| (x,))
}

fn tuple_u8() -> impl Iterator<Item = (u8,)> + Clone {
    (0_u8..255_u8).map(|x| (x,))
}

fn tuple_pair_b8() -> impl Iterator<Item = (b8, b8)> + Clone {
    exhaustive::<8>()
        .into_iter()
        .flat_map(|x| exhaustive::<8>().into_iter().map(move |y| (x, y)))
}

fn tuple_pair_s8() -> impl Iterator<Item = (s8, s8)> + Clone {
    exhaustive::<8>().into_iter().flat_map(|x| {
        exhaustive::<8>()
            .into_iter()
            .map(move |y| (x.as_signed(), y.as_signed()))
    })
}

#[test]
fn test_vm_unsigned_binop_function() {
    #[kernel]
    fn gt(a: b8, b: b8) -> bool {
        a > b
    }

    #[kernel]
    fn ge(a: b8, b: b8) -> bool {
        a >= b
    }

    #[kernel]
    fn eq(a: b8, b: b8) -> bool {
        a == b
    }

    #[kernel]
    fn ne(a: b8, b: b8) -> bool {
        a != b
    }

    #[kernel]
    fn le(a: b8, b: b8) -> bool {
        a <= b
    }

    #[kernel]
    fn lt(a: b8, b: b8) -> bool {
        a < b
    }

    test_kernel_vm_and_verilog::<gt, _, _, _>(gt, tuple_pair_b8()).unwrap();
    test_kernel_vm_and_verilog::<ge, _, _, _>(ge, tuple_pair_b8()).unwrap();
    test_kernel_vm_and_verilog::<eq, _, _, _>(eq, tuple_pair_b8()).unwrap();
    test_kernel_vm_and_verilog::<ne, _, _, _>(ne, tuple_pair_b8()).unwrap();
    test_kernel_vm_and_verilog::<le, _, _, _>(le, tuple_pair_b8()).unwrap();
    test_kernel_vm_and_verilog::<lt, _, _, _>(lt, tuple_pair_b8()).unwrap();
}

#[test]
fn test_vm_signed_binop_function() {
    #[kernel]
    fn gt(a: s8, b: s8) -> bool {
        a > b
    }

    #[kernel]
    fn ge(a: s8, b: s8) -> bool {
        a >= b
    }

    #[kernel]
    fn eq(a: s8, b: s8) -> bool {
        a == b
    }

    #[kernel]
    fn ne(a: s8, b: s8) -> bool {
        a != b
    }

    #[kernel]
    fn le(a: s8, b: s8) -> bool {
        a <= b
    }

    #[kernel]
    fn lt(a: s8, b: s8) -> bool {
        a < b
    }

    test_kernel_vm_and_verilog::<gt, _, _, _>(gt, tuple_pair_s8()).unwrap();
    test_kernel_vm_and_verilog::<ge, _, _, _>(ge, tuple_pair_s8()).unwrap();
    test_kernel_vm_and_verilog::<eq, _, _, _>(eq, tuple_pair_s8()).unwrap();
    test_kernel_vm_and_verilog::<ne, _, _, _>(ne, tuple_pair_s8()).unwrap();
    test_kernel_vm_and_verilog::<le, _, _, _>(le, tuple_pair_s8()).unwrap();
    test_kernel_vm_and_verilog::<lt, _, _, _>(lt, tuple_pair_s8()).unwrap();
}

#[test]
fn test_vm_shr_is_sign_preserving() {
    #[kernel]
    fn shr(a: s12, b: b4) -> s12 {
        a >> b
    }

    let test = [(signed(-42), bits(2))];
    test_kernel_vm_and_verilog::<shr, _, _, _>(shr, test.into_iter()).unwrap();
}

#[test]
fn test_simple_if_expression() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        if a > b {
            a + 1
        } else {
            b + 2
        }
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_plain_literals() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_plain_literals_signed_context() {
    #[kernel]
    fn foo(a: s8, b: s8) -> s8 {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8()).unwrap();
}

#[test]
fn test_assignment() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let mut c = a;
        c = b;
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_assignment_of_if_expression() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let mut c = a;
        c = if a > b { a + 1 } else { b + 2 };
        c
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_tuple_construct() {
    #[kernel]
    fn foo(a: b8, b: b8) -> (b8, b8) {
        (a, b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_tuple_indexing() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = (a, b);
        c.0 + c.1
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_tuple_construct_and_deconstruct() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = (a, b);
        let (d, e) = c;
        d + e
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_nested_tuple_indexing() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = (a, (b, a));
        c.1 .0 + c.1 .1
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_field_indexing() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = Foo { a, b };
        c.a + c.b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_field_indexing_is_order_independent() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> Foo {
        let c = Foo { b, a };
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_tuple_struct_construction() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo(b8, b8);

    #[kernel]
    fn foo(a: b8, b: b8) -> Foo {
        Foo(a, b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_tuple_struct_indexing() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo(b8, b8);

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = Foo(a, b);
        c.0 + c.1
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_struct_field_indexing() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: (b8, b8),
        b: b8,
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let mut c = Foo { a: (a, a), b };
        c.a.0 = c.b;
        c.a.0 + c.a.1 + c.b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_struct_rest_syntax() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: (b8, b8),
        b: b8,
    }

    const FOO: Foo = Foo {
        a: (bits(1), bits(2)),
        b: bits(3),
    };

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = Foo { a: (a, a), ..FOO };
        let Foo { a: (d, e), .. } = c;
        d + e + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_array_inference() {
    #[kernel]
    fn foo(a: b8, b: b8) -> [b8; 2] {
        let c = [a, b];
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_array_indexing() {
    #[kernel]
    fn foo(a: b8, b: b8) -> [b8; 2] {
        let mut c = [a, b];
        c[1] = a;
        c[0] = b;
        [c[0] + c[1], c[1]]
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_enum_basic() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    enum Foo {
        A,
        B(b8),
        C { red: b8, green: b8, blue: b8 },
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> Foo {
        if a == b {
            Foo::A
        } else if a > b {
            Foo::B(a + b)
        } else {
            Foo::C {
                red: a,
                green: b,
                blue: a,
            }
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_match_enum() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    enum Foo {
        A,
        B(b8),
        C { red: b8, green: b8, blue: b8 },
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = Foo::C {
            red: a,
            green: b,
            blue: a,
        };
        match c {
            Foo::A => b8(1),
            Foo::B(x) => x,
            Foo::C { red, green, blue } => red + green + blue,
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_match_value() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        match a {
            Bits::<8>(1) => b,
            Bits::<8>(2) => a,
            _ => b8(3),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_signed_match() {
    #[kernel]
    fn foo(a: s8, b: s8) -> s8 {
        match a {
            SignedBits::<8>(1) => b,
            SignedBits::<8>(2) => a,
            _ => s8(3),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8()).unwrap();
}

#[test]
fn test_exec_sub_kernel() {
    #[kernel]
    fn double(a: b8) -> b8 {
        a + a
    }

    #[kernel]
    fn add(a: b8, b: b8) -> b8 {
        double(a) + b
    }

    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = add(a, b);
        c + a + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_assign_with_computed_expression() {
    #[kernel]
    fn foo(mut a: [b8; 4]) -> [b8; 4] {
        a[1 + 1] = b8(42);
        a
    }
    let test_input = [([bits(1), bits(2), bits(3), bits(4)],)];

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_input.into_iter()).unwrap();
}

#[test]
fn test_repeat_with_generic() {
    #[kernel]
    fn foo<const N: usize>(a: [b8; N]) -> [b8; N] {
        let g = [a[1]; 3 + 2];
        let c = [a[0]; N];
        c
    }
    let test_input = [([bits(1), bits(2), bits(3), bits(4)],)];

    test_kernel_vm_and_verilog::<foo<4>, _, _, _>(foo, test_input.into_iter()).unwrap();
}

#[test]
fn test_repeat_op() {
    #[kernel]
    fn foo(a: b8, b: b8) -> ([b8; 3], [b8; 4]) {
        let c = [a; 3];
        let d = [b; 4];
        (c, d)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_early_return() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        return a;
        b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_phi_mut_no_init() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let mut c: b8;
        if a.any() {
            c = b8(1);
        } else {
            c = b8(2);
        }
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
fn test_flow_control_if_expression() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        let c = if a > b { a + 1 } else { b + 2 };
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

#[test]
#[allow(clippy::no_effect)]
fn test_early_return_in_branch() {
    #[kernel]
    fn foo(a: b8, b: b8) -> b8 {
        if a > b {
            let d = 5;
            d + 3;
            return a;
        }
        b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8()).unwrap();
}

fn rand_bits<const N: usize>() -> Bits<N> {
    let mut rng = rand::thread_rng();
    let val: u128 = rng.gen();
    Bits::<N>(val & Bits::<N>::MASK.0)
}

#[test]
fn test_3d_array_dynamic_indexing() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
    pub struct VolumeBits {
        data: [[[b1; 8]; 8]; 8],
    }

    fn rand_volume_bits() -> VolumeBits {
        let mut ret = VolumeBits::default();
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    ret.data[i][j][k] = rand_bits();
                }
            }
        }
        ret
    }

    #[kernel]
    fn foo(a: VolumeBits, b: b3, c: b3, d: b3) -> b1 {
        a.data[b][c][d]
    }

    let test_cases = (0..100)
        .map(|_| (rand_volume_bits(), rand_bits(), rand_bits(), rand_bits()))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_cases.into_iter()).unwrap();
}

#[test]
fn test_complex_array_dynamic_indexing() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: bool,
        b: [b4; 4],
        c: bool,
    }

    fn rand_foo() -> Foo {
        // make a random Foo
        let mut rng = rand::thread_rng();
        Foo {
            a: rng.gen(),
            b: [rand_bits(), rand_bits(), rand_bits(), rand_bits()],
            c: rng.gen(),
        }
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Bar {
        a: b9,
        b: [Foo; 8],
    }

    fn rand_bar() -> Bar {
        // make a random Bar
        let mut rng = rand::thread_rng();
        Bar {
            a: b9(rng.gen::<u16>() as u128 % 512),
            b: [
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
            ],
        }
    }

    #[kernel]
    fn foo(bar: Bar, n1: b3, n2: b2) -> b4 {
        bar.b[n1].b[n2]
    }

    let test_cases = (0..100)
        .map(|_| (rand_bar(), rand_bits(), rand_bits()))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_cases.into_iter()).unwrap();

    #[kernel]
    fn bar(bar: Bar, n1: b3, b2: b2, b3: b4) -> Bar {
        let mut ret = bar;
        ret.b[n1].b[b2] = b3;
        ret
    }

    let test_cases = (0..100)
        .map(|_| (rand_bar(), rand_bits(), rand_bits(), rand_bits()))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<bar, _, _, _>(bar, test_cases.into_iter()).unwrap();
}

#[test]
fn test_array_dynamic_indexing() {
    #[kernel]
    fn foo(a: [b8; 8], b: b3) -> b8 {
        a[b]
    }

    let a = [
        bits(101),
        bits(102),
        bits(103),
        bits(104),
        bits(105),
        bits(106),
        bits(107),
        bits(108),
    ];
    let b = exhaustive();
    let inputs = b.into_iter().map(|b| (a, b)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter()).unwrap();
}

#[test]
fn test_array_dynamic_indexing_on_write() {
    #[kernel]
    fn foo(a: [b8; 8], b: b3) -> [b8; 8] {
        let mut c = a;
        c[b] = b8(42);
        c[0] = b8(12);
        c
    }
    let a = [
        bits(101),
        bits(102),
        bits(103),
        bits(104),
        bits(105),
        bits(106),
        bits(107),
        bits(108),
    ];
    let b = exhaustive();
    let inputs = b.into_iter().map(|b| (a, b)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter()).unwrap();
}

#[test]
fn test_empty_kernel_args_accepted() {
    #[kernel]
    fn foo(a: (), b: b3, c: ()) -> b3 {
        b
    }

    let inputs = (0..8).map(|x| ((), bits(x), ())).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter()).unwrap();
}

#[test]
fn test_empty_kernel_return_accepted() {
    #[kernel]
    fn foo(d: (), a: b3) -> (bool, ()) {
        (true, d)
    }

    let inputs = (0..8).map(|x| ((), bits(x))).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter()).unwrap();
}
