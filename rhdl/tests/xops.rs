use rhdl::prelude::*;

#[cfg(test)]
mod common;

#[cfg(test)]
use common::*;
use rhdl_core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_xadd_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b8, Red>) -> Signal<b9, Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xadd(a2);
        signal(c)
    }

    let args = exhaustive::<W4>().into_iter().flat_map(|a1| {
        exhaustive::<W8>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .init();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_xadd_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<s4, Red>, a2: Signal<s8, Red>) -> Signal<s9, Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xadd(a2);
        signal(c)
    }

    let args = exhaustive_signed::<W4>().into_iter().flat_map(|a1| {
        exhaustive_signed::<W8>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .init();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_xsub_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b8, Red>) -> Signal<(s9, s9), Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xsub(a2);
        let d = a2.xsub(a1);
        signal((c, d))
    }

    let args = exhaustive::<W4>().into_iter().flat_map(|a1| {
        exhaustive::<W8>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .init();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_xsub_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<s4, Red>, a2: Signal<s8, Red>) -> Signal<(s9, s9), Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xsub(a2);
        let d = a2.xsub(a1);
        signal((c, d))
    }

    let args = exhaustive_signed::<W4>().into_iter().flat_map(|a1| {
        exhaustive_signed::<W8>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .init();
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}
