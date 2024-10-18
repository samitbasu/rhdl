use rhdl::prelude::*;

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
fn test_result_is_digital() {
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
    fn foo(_cr: ClockReset, i: b8) -> FWResult<b8> {
        if i == b8(0b10101010) {
            FWResult::<b8>::Ok(b8(0b01010101))
        } else {
            FWResult::<b8>::Err(Eflag::BadNumber)
        }
    }

    #[kernel]
    fn bar(_cr: ClockReset, i: b8) -> FWResult<b8> {
        let j = match foo(_cr, i) {
            FWResult::<b8>::Ok(j) => j,
            FWResult::<b8>::Err(e) => return FWResult::<b8>::Err(e),
        };
        FWResult::<b8>::Ok(j)
    }
}
