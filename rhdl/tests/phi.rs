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
fn test_phi_if_consts() {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b1, C>) -> Signal<b8, C> {
        let a = a.val();
        let j = if a.any() { 3 } else { 7 };
        signal(b8(j))
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_phi_if_consts_inferred_len() {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let j = if a.any() { 3 } else { 7 };
        signal(bits(j))
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_phi_missing_register_signed_inference() {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<s8, Red> {
        let mut c = signed(0);
        match a.val().raw() {
            0 => c = signed(2),
            1 => c = signed(3),
            _ => {}
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_phi_missing_register() {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let mut c = b8(0);
        if a.val().any() {
            c = b8(1);
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_phi_inferred_lengths() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let c: b6 = bits(0);
        let _d = c;
        let mut c = bits(0);
        if a.val().any() {
            c = bits(2);
        } else {
            c = bits(4);
        }
        let _y = c;
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_phi() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let mut c = b8(0);
        match a.raw() {
            0 => c = b8(2),
            1 => c = b8(3),
            _ => {}
        }
        let _d = c;
        if a.any() {
            c = b8(1);
            c = b8(2);
        } else {
            c = b8(3);
            c = b8(4);
            if a.all() {
                c = b8(5);
            }
        }
        let _y = c;
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_phi_mut_no_init() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, _b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let mut c: b8;
        if a.any() {
            c = b8(1);
        } else {
            c = b8(2);
        }
        signal(c)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_flow_control_if_expression() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = if a > b { a + 1 } else { b + 2 };
        signal(c + 1)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_if_expression() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<bool, Red> {
        signal(a.val() > b.val())
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}
