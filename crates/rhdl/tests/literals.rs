use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_const_match_finite_bits() -> miette::Result<()> {
    const ONE: b8 = bits(1);
    const TWO: b8 = bits(2);
    const THREE: b8 = bits(3);
    #[kernel]
    fn add<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        signal(match a.val() {
            ONE => TWO,
            TWO => THREE,
            _ => ONE,
        })
    }
    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_b8())?;
    Ok(())
}

#[test]
fn test_const_literal_match() {
    #[kernel]
    fn add<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        signal(b8(match a.val().raw() {
            1 => 1,
            2 => 2,
            _ => 3,
        }))
    }
    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_b8()).unwrap();
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

// This test is disabled until we either adopt custom suffixes or do some other thing
// to re-enable the ability to use literals in match arms.
#[test]
fn test_struct_literal_match() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    const FOO1: Foo = Foo {
        a: bits(1),
        b: bits(2),
    };

    const FOO2: Foo = Foo {
        a: bits(3),
        b: bits(4),
    };

    #[kernel]
    fn add(a: Signal<Foo, Red>) -> Signal<b8, Red> {
        let res = match a.val() {
            FOO1 => 1,
            FOO2 => 2,
            _ => 3,
        };
        signal(bits(res))
    }

    let test_vec = (0..4)
        .map(b8)
        .flat_map(|a| (0..4).map(b8).map(move |b| (red(Foo { a, b }),)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter())?;
    Ok(())
}

#[test]
fn test_plain_literals() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b6, Red>, b: Signal<b6, Red>) -> Signal<b6, Red> {
        signal((a.val() + 2 + b.val()).resize())
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_bn_red::<U6>())?;
    Ok(())
}

#[test]
fn test_plain_literals_signed_context() {
    #[kernel]
    fn foo(a: Signal<s6, Red>, b: Signal<s6, Red>) -> Signal<s6, Red> {
        signal(a.val() + 2 + b.val())
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_sn_red::<U6>()).unwrap();
}
