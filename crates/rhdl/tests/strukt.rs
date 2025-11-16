#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;

#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_struct_expr_not_adt() -> miette::Result<()> {
    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b8, C>) -> Signal<Foo, C> {
        let a = a.val();
        let d = Foo {
            a,
            b: bits(2),
            c: [bits(1), bits(2), bits(3)],
        }; // Struct literal
        signal(d)
    }

    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}

#[test]
fn test_tuplestruct_nested_init() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    pub struct Wrap(b8, (b8, b8), b8);

    #[kernel]
    fn add(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let b = Wrap(b8(1), (b8(2), b8(3)), b8(4));
        let Wrap(c, (d, e), f) = b;
        signal(c + d + e + f + a.val())
    }
    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_tuple_struct_construction() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    pub struct Foo(b4, b4);

    #[kernel]
    fn foo(a: Signal<b4, Red>, b: Signal<b4, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        let b = b.val();
        signal(Foo(a, b))
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_bn_red::<4>())?;
    Ok(())
}

#[test]
fn test_struct_rest_syntax() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    pub struct Foo {
        a: (b4, b4),
        b: b4,
    }

    const FOO: Foo = Foo {
        a: (bits(1), bits(2)),
        b: bits(3),
    };

    #[kernel]
    fn foo(a: Signal<b4, Red>, b: Signal<b4, Red>) -> Signal<b4, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo { a: (a, a), ..FOO };
        let Foo { a: (d, e), .. } = c;
        signal(d + e + b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_bn_red::<4>())?;
    Ok(())
}
