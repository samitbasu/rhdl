use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl_core::sim::testbench::kernel::test_kernel_vm_and_verilog_synchronous;

#[test]
fn test_option_is_digital() {
    #[derive(Debug, Digital)]
    struct Test {
        a: Option<b8>,
        b: Option<b8>,
    }

    let foo_test = Test {
        a: Some(b8::from(0b10101011)),
        b: None,
    };

    println!("foo val: {:?}", foo_test);
    assert_eq!(foo_test.a.bin(), b9::from(0b110101011).bin());
    assert_eq!(foo_test.b.bin(), bitx_parse("000000000").unwrap());
}

#[test]
fn test_result_is_digital() -> miette::Result<()> {
    #[derive(Debug, Digital, Default)]
    enum Eflag {
        BadNumber,
        OutOfRange,
        #[default]
        Unknown,
    }

    type FWResult<T> = Result<T, Eflag>;
    #[kernel]
    fn foo(i: b8) -> FWResult<b8> {
        if i.any() {
            Ok(b8(0b01010101))
        } else {
            Err(Eflag::BadNumber)
        }
    }

    #[kernel]
    fn bar(i: b8) -> FWResult<b8> {
        let j = foo(i)?;
        match foo(j) {
            Ok(_k) => Err(Eflag::OutOfRange),
            Err(_e) => Ok(j),
        }
    }
    test_kernel_vm_and_verilog_synchronous::<bar, _, _, _>(
        bar,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}

#[test]
fn test_option_works() -> miette::Result<()> {
    #[kernel]
    fn opt(i: b8) -> Option<b8> {
        if i.any() {
            Some(i)
        } else {
            None
        }
    }

    test_kernel_vm_and_verilog_synchronous::<opt, _, _, _>(
        opt,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}

#[test]
fn test_option_is_kernel_ok() -> miette::Result<()> {
    #[kernel]
    fn validify(i: b8) -> Option<b8> {
        if i.any() {
            Some(i)
        } else {
            None
        }
    }

    #[kernel]
    fn opt(i: b8) -> Option<b8> {
        let j = validify(i)?;
        match validify(j) {
            Some(_k) => None,
            None => Some(j),
        }
    }
    test_kernel_vm_and_verilog_synchronous::<opt, _, _, _>(
        opt,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}
