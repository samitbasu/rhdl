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
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_xmul_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b8, Red>) -> Signal<(b12, b12), Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xmul(a2);
        let d = a2.xmul(a1);
        signal((c, d))
    }

    let args = exhaustive::<W4>().into_iter().flat_map(|a1| {
        exhaustive::<W8>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_xmul_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<s4, Red>, a2: Signal<s8, Red>) -> Signal<(s12, s12), Red> {
        let a1 = a1.val();
        let a2 = a2.val();
        let c = a1.xmul(a2);
        let d = a2.xmul(a1);
        signal((c, d))
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
fn test_xsgn() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<s9, Red> {
        let a = a.val();
        let c = a.xsgn();
        signal(c)
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}

#[test]
fn test_xneg_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<s9, Red> {
        let a = a.val();
        let c = a.xneg();
        signal(c)
    }

    // Check that it works
    let a = b8(255);
    let b = a.xneg();
    assert_eq!(b, s9(-255));
    let a = s8(-128);
    let b = a.xneg();
    assert_eq!(b, s9(128));
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_xneg_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<s8, Red>) -> Signal<s9, Red> {
        let a = a.val();
        let c = a.xneg();
        signal(c)
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, s8_red())?;
    Ok(())
}

#[test]
fn test_xshl_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b10, Red> {
        signal(a.val().xshl::<W2>())
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}

#[test]
fn test_xshl_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<s8, Red>) -> Signal<s10, Red> {
        signal(a.val().xshl::<W2>())
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, s8_red())?;
    Ok(())
}

#[test]
fn test_xshr_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b6, Red> {
        signal(a.val().xshr::<W2>())
    }

    // Test that xshr does the right thing.
    let x = b8(0b1010_1111);
    let y = x.xshr::<W2>();
    assert_eq!(y, b6(0b10_1011));

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}

#[test]
fn test_xshr_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<s8, Red>) -> Signal<s6, Red> {
        signal(a.val().xshr::<W2>())
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, s8_red())?;
    Ok(())
}

#[test]
fn test_xext_unsigned() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b9, Red> {
        signal(a.val().xext::<W1>())
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}

#[test]
fn test_xext_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<s8, Red>) -> Signal<s9, Red> {
        signal(a.val().xext::<W1>())
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, s8_red())?;
    Ok(())
}

#[test]
fn test_xext_unsigned_inferred() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b10, Red> {
        let a = a.val();
        let b = a.xext::<W2>();
        signal(b)
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8::<Red>())?;
    Ok(())
}
