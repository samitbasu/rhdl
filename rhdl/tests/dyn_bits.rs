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
        #[kernel]
        fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<(b4, b4, b4), Red> {
            let a1 = a1.val().dyn_bits();
            let a2 = a2.val().dyn_bits();
            let c = a1 $op a2;
            let d = c + 1;
            let e = 1 + d;
            signal((c.as_bits(), d.as_bits(), e.as_bits()))
        }
        let args = exhaustive::<U4>().into_iter().flat_map(|a1| {
            exhaustive::<U4>()
                .into_iter()
                .map(move |a2| (red(a1), red(a2)))
        });
        test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, args)?;
    }
}

macro_rules! test_op_s4xs4 {
    ($op: tt) => {
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

#[test]
fn test_add_via_dyn_bits() -> miette::Result<()> {
    {
        test_op_b4xb4!(+);
    }
    {
        test_op_s4xs4!(+);
    }
    Ok(())
}

#[test]
fn test_sub_via_dyn_bits() -> miette::Result<()> {
    {
        test_op_b4xb4!(-);
    }
    {
        test_op_s4xs4!(-);
    }
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
    {
        test_op_b4xb4!(*);
    }
    {
        test_op_s4xs4!(*);
    }
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
