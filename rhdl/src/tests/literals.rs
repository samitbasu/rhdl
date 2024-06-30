use crate::tests::{red, tuple_exhaustive_red, tuple_pair_b8_red, tuple_pair_s8_red, tuple_u8};
use rhdl_bits::{alias::*, bits};
use rhdl_core::{
    test_kernel_vm_and_verilog,
    types::{
        domain::{self, Red},
        signal::signal,
    },
    Domain, Signal,
};
use rhdl_macro::{kernel, Digital};

#[test]
fn test_const_literal_match() {
    #[kernel]
    fn add<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        signal(match a.val() {
            1 => 1,
            2 => 2,
            _ => 3,
        })
    }
    test_kernel_vm_and_verilog::<add<domain::Red>, _, _, _>(add::<Red>, tuple_u8()).unwrap();
}

#[test]
fn test_const_literal_captured_match() {
    const ZERO: b4 = bits(0);
    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);

    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) -> Signal<b4, Red> {
        signal(match a.val() {
            ONE => TWO,
            TWO => ONE,
            _ => ZERO,
        })
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_struct_literal_match() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: u8,
        b: u8,
    }

    #[kernel]
    fn add(a: Signal<Foo, Red>) -> Signal<u8, Red> {
        signal(match a.val() {
            Foo { a: 1, b: 2 } => 1,
            Foo { a: 3, b: 4 } => 2,
            _ => 3,
        })
    }

    let test_vec = (0..4)
        .flat_map(|a| (0..4).map(move |b| (red(Foo { a, b }),)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter())?;
    Ok(())
}

#[test]
fn test_plain_literals() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_plain_literals_signed_context() {
    #[kernel]
    fn foo(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8_red()).unwrap();
}
