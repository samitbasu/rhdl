use rhdl::prelude::*;

#[cfg(test)]
mod common;

#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

// A macro to deduplicate the test code for the bN x bN case for
// bits
macro_rules! test_op_b4xb4 {
    ($op: tt) => {
        {
        #[kernel]
        fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<(b4, b4, b4, b4, b4), Red> {
            let b1 = a1.val();
            let b2 = a2.val();
            let a1 = b1.dyn_bits();
            let a2 = b2.dyn_bits();
            let c = a1 $op a2;
            let d = c $op 1;
            let e = 1 $op d;
            let f = a1 $op b2;
            let g = b1 $op a2;
            signal((c.as_bits(), d.as_bits(), e.as_bits(), f, g))
        }
        let args = exhaustive::<U4>().into_iter().flat_map(|a1| {
            exhaustive::<U4>()
                .into_iter()
                .map(move |a2| (red(a1), red(a2)))
        });
        test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    }
    }
}

macro_rules! test_op_s4xs4 {
    ($op: tt) => {
        {
        #[kernel]
        fn do_stuff(a1: Signal<s4, Red>, a2: Signal<s4, Red>) -> Signal<(s4, s4, s4), Red> {
            let a1 = a1.val().dyn_bits();
            let a2 = a2.val().dyn_bits();
            let c = a1 $op a2;
            let d = c + 1;
            let e = 1 + d;
            signal((c.as_signed_bits(), d.as_signed_bits(), e.as_signed_bits()))
        }
        let args = exhaustive_signed::<U4>().into_iter().flat_map(|a1| {
            exhaustive_signed::<U4>()
                .into_iter()
                .map(move |a2| (red(a1), red(a2)))
        });
        test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    }
    }
}

#[test]
fn test_add_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(+);
    test_op_s4xs4!(+);
    Ok(())
}

#[test]
fn test_sub_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(-);
    test_op_s4xs4!(-);
    Ok(())
}

macro_rules! shift_test_bits {
    ($op: tt) => {
    {
        #[kernel]
        fn do_stuff(a1: Signal<b8, Red>, a2: Signal<b3, Red>) -> Signal<(b8, b8, b8), Red> {
            let b1 = a1.val();
            let b2 = a2.val();
            let a1 = b1.dyn_bits();
            let a2 = b2.dyn_bits();
            let c = a1 $op a2;
            let d = c $op 1;
            let e = a1 $op b2;
            signal((c.as_bits(), d.as_bits(), e.as_bits()))
        }
        let args = exhaustive::<U8>().into_iter().flat_map(|a1| {
            exhaustive::<U3>()
                .into_iter()
                .map(move |a2| (red(a1), red(a2)))
        });
        test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    }
    };
}

#[test]
fn test_shift_via_dyn_bits() -> miette::Result<()> {
    env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .init();
    shift_test_bits!(>>);
    shift_test_bits!(<<);
    Ok(())
}

macro_rules! shift_test_signed_bits {
    ($op: tt) => {
    {
        #[kernel]
        fn do_stuff(a1: Signal<s8, Red>, a2: Signal<b3, Red>) -> Signal<(s8, s8, s8), Red> {
            let b1 = a1.val();
            let b2 = a2.val();
            let a1 = b1.dyn_bits();
            let a2 = b2.dyn_bits();
            let c = a1 $op a2;
            let d = c $op 1;
            let e = a1 $op b2;
            signal((c.as_signed_bits(), d.as_signed_bits(), e.as_signed_bits()))
        }
        let args = exhaustive_signed::<U8>().into_iter().flat_map(|a1| {
            exhaustive::<U3>()
                .into_iter()
                .map(move |a2| (red(a1), red(a2)))
        });
        test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    }
    };
}

#[test]
fn test_shift_via_signed_dyn_bits() -> miette::Result<()> {
    shift_test_signed_bits!(>>);
    shift_test_signed_bits!(<<);
    Ok(())
}

#[test]
fn test_shl_signed_via_dyn_bits() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<s8, Red>, a2: Signal<b3, Red>) -> Signal<(s8, s8), Red> {
        let a1 = a1.val().dyn_bits();
        let a2 = a2.val().dyn_bits();
        let c = a1 << a2;
        let d = c << 1;
        signal((c.as_signed_bits(), d.as_signed_bits()))
    }
    let args = exhaustive_signed::<U8>().into_iter().flat_map(|a1| {
        exhaustive::<U3>()
            .into_iter()
            .map(move |a2| (red(a1), red(a2)))
    });
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    Ok(())
}

#[test]
fn test_or_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(|);
    Ok(())
}

#[test]
fn test_and_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(&);
    Ok(())
}

#[test]
fn test_xor_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(^);
    Ok(())
}

#[test]
fn test_mul_via_dyn_bits() -> miette::Result<()> {
    test_op_b4xb4!(*);
    test_op_s4xs4!(*);
    Ok(())
}

#[test]
fn test_add_via_dyn_bits_fails_compile_with_mismatched() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<b5, Red> {
        let a1 = a1.val().dyn_bits();
        let a2 = a2.val().dyn_bits();
        let c = a1 + a2;
        let d = c + 1;
        let e: b5 = d.as_bits();
        signal(e)
    }
    assert!(test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, [].into_iter()).is_err());
    Ok(())
}

macro_rules! check_for_op_that_causes_overflow {
    ($op: ident) => {{
        #[kernel]
        fn do_stuff(a1: Signal<b128, Red>, a2: Signal<b128, Red>) -> Signal<b128, Red> {
            let a1 = a1.val().dyn_bits();
            let a2 = a2.val().dyn_bits();
            let c = a1.$op(a2);
            let c: b128 = c.as_bits();
            signal(c)
        }
        // Should cause a TypeError with bit overflow
        match compile_design::<do_stuff>(CompilationMode::Asynchronous) {
            Ok(_) => panic!("Should have failed to compile"),
            Err(RHDLError::RHDLTypeError(..)) => (),
            Err(_) => panic!("Should have failed to compile with a type error"),
        }
    }};
}

macro_rules! check_for_signed_op_that_causes_overflow {
    ($op: ident) => {{
        #[kernel]
        fn do_stuff(a1: Signal<s128, Red>, a2: Signal<s128, Red>) -> Signal<s128, Red> {
            let a1 = a1.val().dyn_bits();
            let a2 = a2.val().dyn_bits();
            let c = a1.$op(a2);
            let c: s128 = c.as_signed_bits();
            signal(c)
        }
        // Should cause a TypeError with bit overflow
        match compile_design::<do_stuff>(CompilationMode::Asynchronous) {
            Ok(_) => panic!("Should have failed to compile"),
            Err(RHDLError::RHDLTypeError(..)) => (),
            Err(_) => panic!("Should have failed to compile with a type error"),
        }
    }};
}

#[test]
fn test_xops_overflow() -> miette::Result<()> {
    check_for_op_that_causes_overflow!(xadd);
    check_for_op_that_causes_overflow!(xmul);
    check_for_signed_op_that_causes_overflow!(xadd);
    check_for_signed_op_that_causes_overflow!(xmul);
    Ok(())
}

#[test]
fn test_xadd_causes_overflow_warning_at_rhdl_compile_time() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a1: Signal<b128, Red>, a2: Signal<b128, Red>) -> Signal<b128, Red> {
        let a1 = a1.val().dyn_bits();
        let a2 = a2.val().dyn_bits();
        let c = a1.xadd(a2);
        let c: b128 = c.as_bits();
        signal(c)
    }
    // Should cause a TypeError with bit overflow
    match compile_design::<do_stuff>(CompilationMode::Asynchronous) {
        Ok(_) => panic!("Should have failed to compile"),
        Err(RHDLError::RHDLTypeError(..)) => (),
        Err(_) => panic!("Should have failed to compile with a type error"),
    }
    Ok(())
}

#[test]
fn test_xsgn_is_trapped_as_signed() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b4, Red>, b: Signal<b4, Red>) -> Signal<s4, Red> {
        let a = a.val().dyn_bits();
        let b = b.val().dyn_bits();
        let c = a.xsub(b);
        let c: s4 = c.as_signed_bits();
        signal(c)
    }
    // This should cause a type check error because the output xsub is 5 bits, not 4.
    match compile_design::<do_stuff>(CompilationMode::Asynchronous) {
        Ok(_) => panic!("Should have failed to compile"),
        Err(RHDLError::RHDLTypeCheckError(..)) => (),
        Err(x) => panic!("Should have failed to compile with a type error, instead of {x:?}"),
    }
    Ok(())
}
