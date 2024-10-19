use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;

#[test]
fn test_option_is_digital() {
    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
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
    assert_eq!(foo_test.b.bin(), b9::from(0b000000000).bin());
}

#[test]
fn test_result_is_digital() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
    enum Eflag {
        BadOpCode,
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
        Ok(j)
    }
    test_kernel_vm_and_verilog_synchronous::<bar, _, _, _>(
        bar,
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
    test_kernel_vm_and_verilog_synchronous::<validify, _, _, _>(
        validify,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}
