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

#[test]
fn test_struct_expr_not_adt() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<u8, C>) -> Signal<Foo, C> {
        let a = a.val();
        let d = Foo {
            a,
            b: 2,
            c: [1, 2, 3],
        }; // Struct literal
        signal(d)
    }

    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_u8::<Red>())?;
    Ok(())
}

#[test]
fn test_tuplestruct_nested_init() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Wrap(u8, (u8, u8), u8);

    #[kernel]
    fn add(a: Signal<u8, Red>) -> Signal<u8, Red> {
        let b = Wrap(1, (2, 3), 4);
        let Wrap(c, (d, e), f) = b;
        signal(c + d + e + f) + a
    }
    test_kernel_vm_and_verilog::<add, _, _, _>(add, tuple_u8()).unwrap()
}

#[test]
fn test_tuple_struct_construction() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo(b8, b8);

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        let b = b.val();
        signal(Foo(a, b))
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
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
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo { a: (a, a), ..FOO };
        let Foo { a: (d, e), .. } = c;
        signal(d + e + b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}
