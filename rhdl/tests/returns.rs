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
use rhdl_core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_early_return() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, _b: Signal<b8, Red>) -> Signal<b8, Red> {
        return a;
        _b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
#[allow(clippy::no_effect)]
fn test_early_return_in_branch() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        if a > b {
            let d = 5;
            d + 3;
            return a;
        }
        b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_early_return_with_empty_element() -> miette::Result<()> {
    #[kernel]
    fn foo(a: (bool, ())) -> (bool, ()) {
        a
    }

    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    Ok(())
}

#[test]
fn test_early_return_empty_element_constructed() -> miette::Result<()> {
    #[kernel]
    fn foo(a: bool, _q: ()) -> (bool, ()) {
        (!a, ())
    }

    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{module:?}");
    Ok(())
}

#[test]
fn test_empty_return_not_allowed() -> miette::Result<()> {
    #[kernel]
    fn foo(_a: bool) -> () {}

    assert!(compile_design::<foo>(CompilationMode::Asynchronous).is_err());
    Ok(())
}
