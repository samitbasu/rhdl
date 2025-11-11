#![allow(clippy::needless_range_loop)]
#![allow(clippy::let_and_return)]
#![allow(dead_code)]
#![allow(clippy::nonminimal_bool)]
#![allow(unused_parens)]
#![allow(clippy::needless_late_init)]
// Book kernel tests - automatically generated from book examples
// Each module contains a single test from the RHDL book

mod arrays_1 {
    use rhdl::prelude::*;
    use test_log::test;

    #[kernel]
    fn kernel(x: [b4; 4]) -> b6 {
        let mut accum = b6(0);
        for i in 0..4 {
            accum += x[i].resize::<6>();
        }
        accum
    }

    #[test]
    fn test_kernel_block() {
        let kernel = compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
        expect_test::expect_file!["expect/book_kernel_arrays_1.expect"].assert_debug_eq(&kernel);
        let netlist: rhdl_core::ntl::Object = rhdl_core::ntl::from_rtl::build_ntl_from_rtl(&kernel);
        let netlist = rhdl_core::compiler::optimize_ntl(netlist).unwrap();
        expect_test::expect_file!["expect/book_kernel_arrays_1_ntl.expect"]
            .assert_debug_eq(&netlist);
        let vlog = kernel.as_vlog().unwrap();
        let vlog_str = vlog.pretty();
        expect_test::expect_file!["expect/book_kernel_arrays_1_vlog.expect"].assert_eq(&vlog_str);
    }
}

mod arrays_2 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(x: [b4; 8], ndx: b3) -> b4 {
        x[ndx]
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod arrays_3 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(x: b8, ndx: b3) -> bool {
        // 👇 - implies a barrel shifter
        (x & b8(1) << ndx) != 0
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod arrays_4 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel() -> [b4; 4] {
        [b4(3); 4]
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod binary_1 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8, b: b8) -> bool {
        let c = a * b - b;
        let c = a & c;
        let mut d = c + b;
        d >>= 2;
        !((d >= a) || (a == b))
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod blocks_1 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8, b: b8) -> b8 {
        let c = {
            let d = a;
            let e = a + d;
            e + 3 // 👈 block value computed from this expression
        };
        a + c - b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod early_1 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8, b: bool) -> b8 {
        let c = a + 1;
        if b {
            return c; // 👈 Early return
        }
        let c = c + a;
        c
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod early_2 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum InputError {
        TooBig,
        TooSmall,
        #[default]
        UnknownError,
    }

    #[kernel]
    pub fn validate_input(a: b8) -> Result<b8, InputError> {
        if a < 10 {
            Err(InputError::TooSmall)
        } else if a > 200 {
            Err(InputError::TooBig)
        } else {
            Ok(a)
        }
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> Result<b8, InputError> {
        let a = validate_input(a)?;
        let b = validate_input(b)?;
        Ok(a + b)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod early_3 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: Option<b8>) -> Option<b8> {
        let a = a?;
        Some(a + 1)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> (b8, bool) {
        (a, b == a)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_2 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel((a, b): (b8, b8)) -> b8 {
        a + b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_3 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn generic_kernel<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl_bits::W<N>: BitWidth,
    {
        a + b
    }

    #[kernel]
    pub fn kernel(a: b7, b: b7) -> b7 {
        generic_kernel::<7>(a, b)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_4 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyStruct {
        x: b8,
        y: b8,
    }

    #[kernel]
    pub fn kernel(s: MyStruct) -> MyStruct {
        MyStruct {
            x: s.x + s.y,
            y: s.y,
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_5 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn my_add(a: b8, b: b8) -> b8 {
        a + b
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8, c: b8) -> b8 {
        let p1 = my_add(a, b);
        let p2 = my_add(p1, c);
        p2
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod functions_6 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn my_add<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl_bits::W<N>: BitWidth,
    {
        a + b
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8, c: b8) -> b8 {
        //               👇 Must be explicit here!
        let p1 = my_add::<8>(a, b);
        let p2 = my_add::<8>(p1, c);
        p2
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod if_1 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8, b: b8) -> b8 {
        bits(if a > b {
            3
        } else if a == b {
            5
        } else {
            7
        })
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod if_2 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(data: Option<b8>) -> Option<b8> {
        if let Some(data) = data {
            Some(data + 1)
        } else {
            None
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod if_3 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum MyEnum {
        Red(b8),
        Green(b8, b8, b8),
        #[default]
        Blue,
    }

    #[kernel]
    pub fn kernel(data: MyEnum) -> b8 {
        if let MyEnum::Red(x) = data { x } else { b8(42) }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod literals_1 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8) -> b8 {
        let c1 = b8(0xbe); // hexadecimal constant
        let c2 = b8(0b1101_0110); // binary constant
        let c3 = b8(0o03_42); // octal constant
        let c4 = b8(135); // decimal constant
        a + c4 - c1 + c2 + c3
    }

    #[test]
    fn test_kernel_block() -> miette::Result<()> {
        compile_design::<kernel>(CompilationMode::Synchronous)?;
        Ok(())
    }
}

mod literals_2 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8) -> b8 {
        a + 42 // 👈 inferred as a 42 bit constant
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod literals_3 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: b8) -> b8 {
        a + 270 // 👈 panics at runtime or fails at RHDL compile time
    }

    #[test]
    #[should_panic]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod literals_4 {
    use rhdl::prelude::*;

    #[kernel]
    fn kernel(a: bool) -> bool {
        (a ^ true) || false
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod loops_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b32) -> b9 {
        let mut count = b9(0);
        for i in 0..32 {
            if a & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod loops_2 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn count_ones<const N: usize, const M: usize>(a: Bits<N>) -> Bits<M>
    where
        rhdl_bits::W<N>: BitWidth,
        rhdl_bits::W<M>: BitWidth,
    {
        let mut count = bits::<M>(0);
        for i in 0..N {
            if a & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    #[kernel]
    pub fn kernel(a: b8) -> b4 {
        count_ones::<8, 4>(a)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod loops_3 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn xnor<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl_bits::W<N>: BitWidth,
    {
        let mut ret_value = bits::<N>(0);
        for i in 0..N {
            let a_bit = a & (1 << i) != 0;
            let b_bit = b & (1 << i) != 0;
            if !(a_bit ^ b_bit) {
                ret_value |= (1 << i);
            }
        }
        ret_value
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> b8 {
        xnor::<8>(a, b)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod loops_4 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn generic<const N: usize>(a: [bool; N], b: [bool; N]) -> [bool; N] {
        let mut ret_value = [false; N];
        for i in 0..N {
            ret_value[i] = !(a[i] ^ b[i]);
        }
        ret_value
    }

    #[kernel]
    pub fn kernel(a: [bool; 4], b: [bool; 4]) -> [bool; 4] {
        generic::<4>(a, b)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x {
            Bits::<8>(0) => b3(0),
            Bits::<8>(1) => b3(1),
            Bits::<8>(3) => b3(2),
            _ => b3(5),
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_2 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x.raw() {
            0 => b3(0),
            1 => b3(1),
            3 => b3(2),
            _ => b3(5),
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_3 {
    use rhdl::prelude::*;

    pub const NO_DATA: b8 = b8(0);
    pub const SINGLE: b8 = b8(1);
    pub const MULTIPLE: b8 = b8(3);

    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x {
            NO_DATA => b3(0),
            SINGLE => b3(1),
            MULTIPLE => b3(2),
            _ => b3(5),
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_4 {
    use rhdl::prelude::*;

    //       👇 namespace the raw constants in a module
    pub mod error_codes {
        use super::*;
        pub const ALL_OK: b2 = b2(0);
        pub const ENDPOINT_ERROR: b2 = b2(1);
        pub const ADDRESS_ERROR: b2 = b2(2);
        pub const RESERVED_ERROR: b2 = b2(3);
    }

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum BusError {
        // 👈 Create a RHDL enum for the variants
        Endpoint,
        Address,
        #[default]
        Reserved,
    }

    #[kernel]
    pub fn kernel(x: b2, data: b8) -> Result<b8, BusError> {
        match x {
            error_codes::ALL_OK => Ok(data),
            error_codes::ENDPOINT_ERROR => Err(BusError::Endpoint),
            error_codes::ADDRESS_ERROR => Err(BusError::Address),
            error_codes::RESERVED_ERROR => Err(BusError::Reserved),
            _ => Err(BusError::Reserved), // 👈 unreachable but rustc doesn't know this
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

pub mod match_5 {
    use rhdl::prelude::*;

    #[derive(PartialEq, Digital, Debug, Default, Clone, Copy)]
    pub enum SimpleEnum {
        #[default]
        Init,
        Run(b8),
        Point {
            x: b4,
            y: b8,
        },
        Boom,
    }

    const B6: b6 = bits(6);

    #[kernel]
    fn kernel(state: SimpleEnum) -> b8 {
        match state {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_6 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct Point {
        x: b8,
        y: b8,
    }

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct Reflect(pub Point);

    #[kernel]
    pub fn kernel(x: Reflect) -> b8 {
        let Reflect(p) = x;
        let Point { x, y: _ } = p;
        x
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod match_7 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(x: Option<b8>) -> Option<b8> {
        if let Some(v) = x { Some(v + 1) } else { None }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod methods_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: s8) -> bool {
        let x = a.any();
        let y = b.any();
        let x = x && a.all();
        let y = y && b.all();
        let x = x && a.xor();
        let y = y && b.xor();
        let a_as_s8: s8 = a.as_signed();
        let b_as_b8: b8 = b.as_unsigned();
        (a_as_s8 == b) && (b_as_b8 == a) && x && y
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_1 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyStruct {
        a: b8,
        b: b4,
        c: bool,
    }

    #[kernel]
    pub fn kernel(a: b8) -> MyStruct {
        let mut t = MyStruct {
            a,
            b: b4(1),
            c: a == 0,
        };
        t.b = b4(2);
        t
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_2 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyOtherStruct(pub b8, pub b8, pub b8);

    #[kernel]
    pub fn kernel(t: MyOtherStruct) -> b8 {
        let mut ret = t.0;
        if t.1 > ret {
            ret = t.1;
        }
        if t.2 > ret {
            ret = t.2;
        }
        ret
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_3 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyStruct {
        a: b8,
        b: b4,
        c: bool,
    }

    #[kernel]
    pub fn kernel(t: MyStruct) -> b4 {
        let MyStruct { a, b, c } = t;
        if c { b } else { a.resize::<4>() }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_4 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyStruct {
        a: b8,
        b: b4,
        c: bool,
    }

    #[kernel]
    pub fn kernel(t: MyStruct) -> b4 {
        let MyStruct { b, .. } = t;
        b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_5 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub struct MyStruct {
        a: b8,
        b: b4,
        c: bool,
    }

    #[kernel]
    pub fn kernel(a: b8) -> MyStruct {
        MyStruct {
            a,
            ..MyStruct::default()
        }
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod structs_6 {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub struct Color(pub b6, pub b6, pub b6);

    #[kernel]
    pub fn kernel(c: Color) -> b7 {
        let Color(red, _green, _blue) = c;
        red.resize::<7>() << 1
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod tuples_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: (b8, b8)) -> b8 {
        let (x, y) = a;
        let z = (x, x, y);
        let a = z.0 + z.1 + z.2;
        a + 1
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_1 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let c = a == b; // type is inferred as bool
        let d = !c;
        d
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_2 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let mut c = a + 1;
        c += a; // mutates c
        c == b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_3 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let c;
        let d = a + b;
        c = d;
        c == a
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_4 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> b8 {
        let c: b8 = a;
        c + b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_5 {
    use rhdl::prelude::*;

    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    pub fn kernel(arg: Foo) -> b8 {
        let Foo { a, b } = arg;
        a + b
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_6 {
    use rhdl::prelude::*;

    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo(b8);

    #[kernel]
    pub fn kernel(arg: Foo) -> Foo {
        let Foo(a) = arg;
        Foo(a + 1)
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_7 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn kernel(a: (b8, b8)) -> bool {
        let (c, d) = a;
        c == d
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod variables_8 {
    use rhdl::prelude::*;

    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Bar(b8, b8);

    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo {
        a: b8,
        b: Bar,
    }

    #[kernel]
    pub fn kernel(state: Signal<Foo, Red>) -> Signal<b8, Red> {
        let Foo { a, b: Bar(_x, y) } = state.val();
        signal((a + y).resize())
    }

    #[test]
    fn test_kernel_block() {
        compile_design::<kernel>(CompilationMode::Synchronous).unwrap();
    }
}

mod simulation_exhaustive {
    use rhdl::prelude::*;
    fn counter(cr: ClockReset, i: bool, q: b8) -> (b8, b8) {
        let d = if i { q + 1 } else { q };
        let o = q;
        (o, d)
    }

    #[test]
    fn test_counter_exhaustively() {
        let cr = clock_reset(clock(false), reset(false));
        for i in [false, true] {
            for q in (0..256).map(b8) {
                let (o, d) = counter(cr, i, q);
                if i {
                    assert_eq!(d, q + 1);
                } else {
                    assert_eq!(d, q);
                }
                assert_eq!(o, q);
            }
        }
    }
}

#[test]
fn test_iterator_timed_from_map() {
    use rhdl::prelude::*;
    let i = (0..)
        .map(b8)
        .map(signal::<_, Red>)
        .enumerate()
        .map(|(ndx, s)| timed_sample(ndx as u64 * 50, s));
    i.take(10).for_each(|x| {
        println!("{x}");
    })
}
