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
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;
use rhdl::prelude::*;

#[test]
fn test_func_with_structured_args() -> miette::Result<()> {
    #[kernel]
    fn do_stuff((a, b): (Signal<b8, Red>, Signal<b8, Red>)) -> Signal<b8, Red> {
        let c = (a, b);
        let _d = c.0;
        signal(a.val() + b.val())
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(
        do_stuff,
        [((signal(b8(0)), signal(b8(3))),)].into_iter(),
    )?;
    Ok(())
}

#[test]
fn test_basic_cast() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<DATA: BitWidth>(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let bytes_per_word: Bits<U8> = bits(({ DATA::BITS } >> 3) as u128);
        let b = a.val() + bytes_per_word;
        signal(b.resize())
    }
    test_kernel_vm_and_verilog::<do_stuff<U32>, _, _, _>(do_stuff::<U32>, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func() -> miette::Result<()> {
    use rhdl::bits::alias::*;
    #[derive(PartialEq, Digital, Default)]
    pub struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[derive(PartialEq, Digital, Default)]
    pub enum State {
        #[default]
        Init,
        Run(b8),
        Boom,
        Unknown,
    }

    #[derive(PartialEq, Digital, Default)]
    pub struct Bar(pub b8, pub b8);

    #[kernel]
    fn do_stuff(arg: Signal<b4, Red>) -> Signal<b8, Red> {
        let a = arg.val(); // Straight local assignment
        let b = !a; // Unary operator
        let c = a + b - 1; // Binary operator
        let q = (a, b, c); // Tuple valued expression
        let (_a, _b, _c) = q; // Tuple destructuring
        let h = Bar(bits(1), bits(2)); // Tuple struct literal
        let _i = h.0; // Tuple struct field access
        let Bar(_j, _k) = h; // Tuple struct destructuring
        let _d = [1, 2, 3]; // Array literal
        let d = Foo {
            a: bits(1),
            b: bits(2),
            c: [bits(1), bits(2), bits(3)],
        }; // Struct literal
        let _p = Foo { a: bits(4), ..d };
        let _h = {
            let e = 3;
            let f = 4;
            b8(e) + b8(f)
        }; // Statement expression
        let Foo { a, b, .. } = d; // Struct destructuring
        let g = d.c[1]; // Array indexing
        let e = d.a; // Struct field access
        trace("dump", &(a, b, e, g));
        let mut d: b8 = b8(7); // Mutable local
        if d > b8(0) {
            // if statement
            d -= 1;
            // early return
            return signal(d);
        }
        // if-else statement (and a statement expression)
        let _j = if d < bits(3) { 7 } else { 9 };
        // Enum literal
        let k = State::Boom;
        // Enum literal with a payload
        let l = State::Run(bits(3));
        // Match expression with enum variants
        let j = match l {
            State::Init => b3(1),
            State::Run(_a) => b3(2),
            State::Boom => b3(3),
            _ => b3(4),
        };
        trace("dump2", &(j, k));
        // For loops
        for ndx in 0..8 {
            d = (d + b8(ndx)).resize();
        }
        // block expression
        signal(b8(42))
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
    fn foo(_a: Signal<b8, Red>) {}
    let Err(RHDLError::RHDLSyntaxError(err)) = compile_design::<foo>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    Ok(())
}

#[test]
fn test_empty_kernel_args_accepted() -> miette::Result<()> {
    #[kernel]
    fn foo(_a: Signal<(), Red>, b: Signal<b3, Red>, _c: Signal<(), Red>) -> Signal<b3, Red> {
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
    fn foo(d: Signal<(), Red>, _a: Signal<b3, Red>) -> (Signal<bool, Red>, Signal<(), Red>) {
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
        let _g = [a[1]; 3 + 2];
        let c = [a[0]; N];
        signal(c)
    }
    let test_input = [(signal([bits(1), bits(2), bits(3), bits(4)]),)];

    test_kernel_vm_and_verilog::<foo<4>, _, _, _>(foo, test_input.into_iter())?;
    Ok(())
}

#[test]
fn test_if_let_syntax() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Default, Digital)]
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
        signal(a.val() + a.val())
    }

    #[kernel]
    fn add(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        signal((double(a).val() + double(b).val()).resize())
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = add(a, b).val() + double(b).val();
        signal((c + a.val() + b.val()).resize())
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
    const ONE: b8 = bits(1);
    const TWO: b8 = bits(2);
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        match a.val() {
            ONE => b,
            TWO => a,
            _ => signal(b8(3)),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_basic_compile() -> miette::Result<()> {
    use itertools::iproduct;

    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo {
        a: b4,
        b: b4,
    }

    #[derive(PartialEq, Debug, Digital)]
    pub struct TupStruct(b4, b4);

    #[derive(PartialEq, Debug, Default, Digital)]
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

    #[derive(PartialEq, Debug, Digital, Default)]
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

    #[kernel]
    fn nib_add<C: Domain>(a: Signal<b4, C>, b: Signal<b4, C>) -> Signal<b4, C> {
        signal((a.val() + b.val()).resize())
    }

    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);
    const THREE: b4 = bits(3);
    const MOMO: b8 = bits(15);

    #[kernel]
    fn add<C: Domain>(
        mut a: Signal<b4, C>,
        b: Signal<[b4; 3], C>,
        state: Signal<SimpleEnum, C>,
    ) -> Signal<b4, C> {
        let a = a.val();
        let (d, c) = (1, 3);
        let p = a + c;
        let _q = p;
        let b = b.val();
        let q = b[2];
        let _p = [q; 3];
        let k = (q, q, q, q);
        let mut p: b4 = (k.2 + d).resize();
        if p > 2 {
            return signal(p.resize());
        }
        p = a - 1;
        let mut q = Foo { a, b: b[2] };
        let Foo { a: x, b: y } = q;
        q.a = (q.a + p).resize();
        let mut bb = b;
        bb[2] = p.resize();
        let z: b4 = (p + nib_add::<C>(signal(x), signal(y)).val()).resize();
        let q = TupStruct(x, y);
        let TupStruct(x, _y) = q;
        let _h = Bar::A;
        let _h = Bar::B(p);
        let _h = Bar::C { x: p, y: p };
        let _k: Bar = Bar::A;
        match x {
            ONE => {}
            TWO => {}
            THREE => {}
            _ => {}
        }
        let _count = match state.val() {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
        };
        signal((a + c + z).resize())
    }

    let a_set = exhaustive();
    let b_set: Vec<[b4; 3]> = iproduct!(a_set.iter(), a_set.iter(), a_set.iter())
        .map(|x| [*x.0, *x.1, *x.2])
        .collect();
    let state_set = [
        SimpleEnum::Init,
        SimpleEnum::Run(bits(1)),
        SimpleEnum::Run(bits(5)),
        SimpleEnum::Point {
            x: bits(7),
            y: bits(11),
        },
        SimpleEnum::Point {
            x: bits(7),
            y: bits(13),
        },
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

    let a = [s4(1), s4(2), s4(3), s4(-1), s4(-3)];
    let inputs =
        iproduct!(a.iter().cloned().map(red), a.iter().cloned().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_nested_generics() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
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
        let _d = (x.a, y.b);
        let e = Foo::<T> { a: c, b: c };
        signal(e == x)
    }

    let a = [s4(1), s4(2), s4(3), s4(-1), s4(-3)];
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
    const ONE: s8 = s8(1);
    const TWO: s8 = s8(2);

    #[kernel]
    fn foo(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        match a.val() {
            ONE => b,
            TWO => a,
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
        let a = a.val();
        let b = b.val();
        let mut c = a;
        c = (if a > b { a + 1 } else { b + 2 }).resize();
        signal(c)
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
    fn sum_bits<N: BitWidth>(a: Signal<Bits<N>, Red>) -> Signal<bool, Red> {
        let mut ret = false;
        let a = a.val();
        for i in 0..N::BITS {
            if a & (1 << i) != 0 {
                ret ^= true;
            }
        }
        trace("a", &a);
        signal(ret)
    }
    let res = compile_design::<sum_bits<U8>>(CompilationMode::Asynchronous)?;
    let inputs = (0..256).map(|x| (signal(bits(x)),));
    test_kernel_vm_and_verilog::<sum_bits<U8>, _, _, _>(sum_bits::<U8>, inputs)?;
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
            a = (a + b4(ndx)).resize();
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
    let z = b4(0b1010);
    match z.raw() {
        0b0000 => {}
        0b0001 => {}
        _ => {}
    }
}

#[test]
fn test_maybe_init_does_not_allow_select() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
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
fn test_multiply() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = a * b;
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_maybe_init_escape_causes_error() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
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
    #[derive(PartialEq, Debug, Digital, Default)]
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
    fn do_stuff(_a: Signal<b4, Red>) -> Signal<Foo, Red> {
        let mut foo = Foo::D;
        signal(foo)
    }

    compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}
