#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;
use rhdl::prelude::*;
use test_log::test;

#[test]
fn test_func_with_structured_args() -> miette::Result<()> {
    #[kernel]
    fn do_stuff((a, b): (Signal<b8, Red>, Signal<b8, Blue>)) -> Signal<b8, Red> {
        /// This assignment is very very importante
        let c = (a, b);
        let _d = c.0;
        // Invisible comment
        signal(a.val() + b.val())
    }
    let err = test_kernel_vm_and_verilog::<do_stuff, _, _, _>(
        do_stuff,
        [((signal(b8(0)), signal(b8(3))),)].into_iter(),
    )
    .expect_err("Expected this to fail with a clock domain violation");
    let report = miette_report(err);
    expect_test::expect_file!["expect/span_test.expect"].assert_eq(&report);
    Ok(())
}
