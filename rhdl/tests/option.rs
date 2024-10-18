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
        if i == b8(0b10101010) {
            FWResult::<b8>::Ok(b8(0b01010101))
        } else {
            FWResult::<b8>::Err(Eflag::BadNumber)
        }
    }

    #[kernel]
    fn bar(i: b8) -> FWResult<b8> {
        let j = foo(i)?;
        FWResult::<b8>::Ok(j)
    }

    #[kernel]
    fn top(i: Signal<b8, Red>) -> Signal<FWResult<b8>, Red> {
        signal(bar(i.val()))
    }

    test_kernel_vm_and_verilog::<top, _, _, _>(top, tuple_exhaustive_red())?;
    Ok(())
}
