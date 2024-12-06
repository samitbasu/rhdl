#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use itertools::iproduct;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::prelude::*;
use rhdl_core::{
    flow_graph::optimization::optimize_flow_graph,
    sim::testbench::kernel::test_kernel_vm_and_verilog,
};

#[test]
fn test_func_with_structured_args() -> miette::Result<()> {
    #[kernel]
    fn do_stuff((a, b): (Signal<b8, Red>, Signal<b8, Red>)) -> Signal<b8, Red> {
        let c = (a, b);
        let d = c.0;
        a + b
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(
        do_stuff,
        [((signal(b8(0)), signal(b8(3))),)].into_iter(),
    )?;
    Ok(())
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func() -> miette::Result<()> {
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
        Unknown,
    }

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
    pub struct Bar(pub u8, pub u8);

    #[kernel]
    fn do_stuff(arg: Signal<b4, Red>) -> Signal<b8, Red> {
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
            return signal(d);
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
            _ => b3(4),
        };
        // For loops
        for ndx in 0..8 {
            d = d + bits::<8>(ndx);
        }
        // block expression
        signal(bits::<8>(42))
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_method_call_syntax() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<(bool, bool, bool, s8), Red> {
        let a = a.val();
        let any = a.any();
        let all = a.all();
        let xor = a.xor();
        let s = a.as_signed();
        signal((any, all, xor, s))
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_empty_return_rejected() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>) {}
    let Err(RHDLError::RHDLSyntaxError(err)) = compile_design::<foo>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    Ok(())
}

#[test]
fn test_empty_kernel_args_accepted() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<(), Red>, b: Signal<b3, Red>, c: Signal<(), Red>) -> Signal<b3, Red> {
        b
    }

    let inputs = (0..8)
        .map(|x| (red(()), red(bits(x)), red(())))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_empty_kernel_return_accepted() -> miette::Result<()> {
    #[kernel]
    fn foo(d: Signal<(), Red>, a: Signal<b3, Red>) -> (Signal<bool, Red>, Signal<(), Red>) {
        (signal(true), d)
    }

    let inputs = (0..8).map(|x| (red(()), red(bits(x)))).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_repeat_with_generic() -> miette::Result<()> {
    #[kernel]
    fn foo<const N: usize>(a: Signal<[b8; N], Red>) -> Signal<[b8; N], Red> {
        let a = a.val();
        let g = [a[1]; 3 + 2];
        let c = [a[0]; N];
        signal(c)
    }
    let test_input = [(signal([bits(1), bits(2), bits(3), bits(4)]),)];

    test_kernel_vm_and_verilog::<foo<4>, _, _, _>(foo, test_input.into_iter())?;
    Ok(())
}

#[test]
fn test_if_let_syntax() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Default, Digital)]
    pub enum Foo {
        Bar(b8),
        #[default]
        Baz,
    }

    #[kernel]
    fn foo(a: Signal<Foo, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = if let Foo::Bar(x) = a { x } else { b8(0) };
        signal(b)
    }

    let test_input = [(signal(Foo::Bar(bits(3))),), (signal(Foo::Baz),)];
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_input.into_iter())?;
    Ok(())
}

#[test]
fn test_repeat_op() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<([b8; 3], [b8; 4]), Red> {
        let a = a.val();
        let b = b.val();
        let c = [a; 3];
        let d = [b; 4];
        signal((c, d))
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_exec_sub_kernel() -> miette::Result<()> {
    #[kernel]
    fn double(a: Signal<b8, Red>) -> Signal<b8, Red> {
        a + a
    }

    #[kernel]
    fn add(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        double(a) + double(b)
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = add(a, b) + double(b);
        c + a + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_assign_with_computed_expression() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<[b8; 4], Red>) -> Signal<[b8; 4], Red> {
        let mut a = a.val();
        a[1 + 1] = b8(42);
        signal(a)
    }
    let test_input = [(signal([bits(1), bits(2), bits(3), bits(4)]),)];

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_input.into_iter())?;
    Ok(())
}

#[test]
fn test_match_value() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        match a.val() {
            Bits::<8>(1) => b,
            Bits::<8>(2) => a,
            _ => signal(b8(3)),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_basic_compile() -> miette::Result<()> {
    use itertools::iproduct;

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: b4,
        b: b4,
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct TupStruct(b4, b4);

    #[derive(PartialEq, Copy, Clone, Debug, Default, Digital)]
    pub enum Bar {
        A,
        B(b4),
        C {
            x: b4,
            y: b4,
        },
        #[default]
        D,
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
    pub enum SimpleEnum {
        #[default]
        Init,
        Run(u8),
        Point {
            x: b4,
            y: u8,
        },
        Boom,
    }

    #[kernel]
    fn nib_add<C: Domain>(a: Signal<b4, C>, b: Signal<b4, C>) -> Signal<b4, C> {
        a + b
    }

    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);
    const MOMO: u8 = 15;

    #[kernel]
    fn add<C: Domain>(
        mut a: Signal<b4, C>,
        b: Signal<[b4; 3], C>,
        state: Signal<SimpleEnum, C>,
    ) -> Signal<b4, C> {
        let a = a.val();
        let (d, c) = (1, 3);
        let p = a + c;
        let q = p;
        let b = b.val();
        let q = b[2];
        let p = [q; 3];
        let k = (q, q, q, q);
        let mut p = k.2 + d;
        if p > 2 {
            return signal(p);
        }
        p = a - 1;
        let mut q = Foo { a, b: b[2] };
        let Foo { a: x, b: y } = q;
        q.a += p;
        let mut bb = b;
        bb[2] = p;
        let z: b4 = p + nib_add::<C>(signal(x), signal(y)).val();
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
        let count = match state.val() {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        };
        signal(a + c + z)
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
    let inputs = iproduct!(
        a_set.into_iter().map(red),
        b_set.into_iter().map(red),
        state_set.into_iter().map(red)
    )
    .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_generics() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<T: Digital, C: Domain>(a: Signal<T, C>, b: Signal<T, C>) -> Signal<bool, C> {
        signal(a == b)
    }

    let a = [
        signed::<4>(1),
        signed::<4>(2),
        signed::<4>(3),
        signed::<4>(-1),
        signed::<4>(-3),
    ];
    let inputs =
        iproduct!(a.iter().cloned().map(red), a.iter().cloned().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_nested_generics() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital)]
    struct Foo<T: Digital> {
        a: T,
        b: T,
    }

    #[kernel]
    fn do_stuff<T: Digital, S: Digital, C: Domain>(
        x: Signal<Foo<T>, C>,
        y: Signal<Foo<S>, C>,
    ) -> Signal<bool, C> {
        let x = x.val();
        let y = y.val();
        let c = x.a;
        let d = (x.a, y.b);
        let e = Foo::<T> { a: c, b: c };
        signal(e == x)
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
        a.into_iter().map(|x| Foo { a: x, b: x }).map(red),
        b.into_iter().map(|x| Foo { a: x, b: x }).map(red)
    )
    .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, b3, Red>, _, _, _>(
        do_stuff::<s4, b3, Red>,
        inputs.into_iter(),
    )?;
    Ok(())
}

#[test]
fn test_signed_match() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        match a.val() {
            SignedBits::<8>(1) => b,
            SignedBits::<8>(2) => a,
            _ => signal(s8(3)),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8_red())?;
    Ok(())
}

#[test]
fn test_assignment_of_if_expression() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let mut c = a;
        c = if a > b { a + 1 } else { b + 2 };
        c
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_precomputation() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = a.val();
        let c = c + 5 + 3 - 1;
        signal(c)
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_for_loop_const_generics() -> miette::Result<()> {
    #[kernel]
    fn sum_bits<const N: usize>(a: Signal<Bits<N>, Red>) -> Signal<bool, Red> {
        let mut ret = false;
        let a = a.val();
        for i in 0..N {
            if a & (1 << i) != 0 {
                ret ^= true;
            }
        }
        trace("a", &a);
        signal(ret)
    }
    let res = compile_design::<sum_bits<8>>(CompilationMode::Asynchronous)?;
    let inputs = (0..256).map(|x| (signal(bits(x)),));
    test_kernel_vm_and_verilog::<sum_bits<8>, _, _, _>(sum_bits::<8>, inputs)?;
    Ok(())
}

#[test]
#[allow(clippy::needless_range_loop)]
fn test_for_loop() -> miette::Result<()> {
    #[kernel]
    fn looper(a: Signal<[bool; 8], Red>) -> Signal<bool, Red> {
        let a = a.val();
        let mut ret: bool = false;
        for i in 0..8 {
            ret ^= a[i];
        }
        signal(ret)
    }
    let inputs = (0..256).map(|x| {
        let mut a = [false; 8];
        for i in 0..8 {
            a[i] = (x >> i) & 1 == 1;
        }
        (signal(a),)
    });
    test_kernel_vm_and_verilog::<looper, _, _, _>(looper, inputs)?;
    Ok(())
}

#[test]
fn test_error_about_for_loop() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) {
        let mut a = a.val();
        let c = 5;
        for ndx in 0..c {
            a += bits::<4>(ndx);
        }
    }
    let Err(RHDLError::RHDLSyntaxError(err)) =
        compile_design::<do_stuff>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    assert!(matches!(
        err.cause,
        rhdl::core::compiler::mir::error::Syntax::ForLoopNonIntegerEndValue
    ));
    Ok(())
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
fn test_maybe_init_does_not_allow_select() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    struct Foo {
        a: b4,
        b: b4,
        c: bool,
    }

    #[kernel]
    fn do_stuff(a: Signal<b4, Red>, b: Signal<b4, Red>) -> Signal<b4, Red> {
        let mut foo = Foo::dont_care();
        foo.a = a.val();
        foo.b = b.val();
        signal(if foo.c { foo.a } else { foo.b })
    }
    assert!(compile_design::<do_stuff>(CompilationMode::Asynchronous).is_err());
    Ok(())
}

#[test]
fn test_maybe_init_escape_causes_error() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    struct Foo {
        a: b4,
        b: b4,
    }

    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) -> Signal<Foo, Red> {
        let mut foo = Foo::dont_care();
        foo.a = a.val();
        signal(foo)
    }
    assert!(compile_design::<do_stuff>(CompilationMode::Asynchronous).is_err());
    Ok(())
}

#[test]
fn test_maybe_init_with_enum() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
    enum Foo {
        A,
        B(b4),
        C {
            a: b4,
            b: b4,
        },
        #[default]
        D,
    }

    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) -> Signal<Foo, Red> {
        let mut foo = Foo::D;
        signal(foo)
    }

    compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}
