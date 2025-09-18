#![allow(clippy::upper_case_acronyms)]
use expect_test::expect_file;
use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::{
    test_kernel_vm_and_verilog, test_kernel_vm_and_verilog_synchronous,
};

#[test]
fn test_option_is_digital() {
    #[derive(PartialEq, Debug, Digital)]
    struct Test {
        a: Option<b8>,
        b: Option<b8>,
    }

    let foo_test = Test {
        a: Some(b8::from(0b10101011)),
        b: None,
    };

    println!("foo val: {foo_test:?}");
    assert_eq!(foo_test.a.bin(), b9::from(0b110101011).bin());
    assert_eq!(foo_test.b.bin(), bitx_parse("000000000").unwrap());
}

#[test]
fn test_result_is_digital() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
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
        if i.any() { Some(i) } else { None }
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
        if i.any() { Some(i) } else { None }
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

#[test]
fn test_option_result_no_ice() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn err_map(e: AXI4Error) -> ResponseCode {
        match e {
            AXI4Error::SLVERR => ResponseCode::SLVERR,
            AXI4Error::DECERR => ResponseCode::DECERR,
        }
    }

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = if let Some(resp) = i.val() {
            match resp {
                Result::Ok(()) => Some(ResponseCode::OKAY),
                Result::Err(e) => Some(err_map(e)),
            }
        } else {
            None
        };
        signal(d)
    }

    let inputs = [
        (signal(Some(Ok(()))),),
        (signal(Some(Err(AXI4Error::SLVERR))),),
        (signal(Some(Err(AXI4Error::DECERR))),),
        (signal(None),),
    ];

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_option_result_match_func() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn err_map(e: AXI4Error) -> ResponseCode {
        match e {
            AXI4Error::SLVERR => ResponseCode::SLVERR,
            AXI4Error::DECERR => ResponseCode::DECERR,
        }
    }

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = match i.val() {
            Some(Result::Ok(())) => Some(ResponseCode::OKAY),
            Some(Result::Err(e)) => Some(err_map(e)),
            None => None,
        };
        signal(d)
    }

    let expect = expect_file!["option_result_match.expect"];
    let res = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    let res = res.err().unwrap();
    let report = miette_report(res);
    expect.assert_eq(&report);
    Ok(())
}

#[test]
fn test_option_result_if_let() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = if let Some(resp) = i.val() {
            match resp {
                Ok(()) => Some(ResponseCode::OKAY),
                Err(e) => Some(match e {
                    AXI4Error::SLVERR => ResponseCode::SLVERR,
                    AXI4Error::DECERR => ResponseCode::DECERR,
                }),
            }
        } else {
            None
        };
        signal(d)
    }

    let inputs = [
        (signal(Some(Ok(()))),),
        (signal(Some(Err(AXI4Error::SLVERR))),),
        (signal(Some(Err(AXI4Error::DECERR))),),
        (signal(None),),
    ];

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_nested_matches() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    #[derive(PartialEq, Default, Digital)]
    pub struct ReadResponse<N: BitWidth> {
        data: Bits<N>,
        resp: ResponseCode,
    }

    #[kernel]
    fn do_stuff<DATA: BitWidth>(
        a: Signal<Result<Bits<DATA>, AXI4Error>, Red>,
    ) -> Signal<Option<ReadResponse<DATA>>, Red> {
        let b = match a.val() {
            Ok(data) => ReadResponse::<DATA> {
                data,
                resp: ResponseCode::OKAY,
            },
            Err(e) => ReadResponse::<DATA> {
                data: bits(0),
                resp: match e {
                    AXI4Error::SLVERR => ResponseCode::SLVERR,
                    AXI4Error::DECERR => ResponseCode::DECERR,
                },
            },
        };
        signal(Some(b))
    }
    let inputs = [
        (signal(Ok(bits(12))),),
        (signal(Err(AXI4Error::SLVERR)),),
        (signal(Err(AXI4Error::DECERR)),),
    ];
    test_kernel_vm_and_verilog::<do_stuff<U4>, _, _, _>(do_stuff::<U4>, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_option_result_nested_option_result_destructure() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let resp = i.val();
        let d = match resp {
            Some(Result::Ok(())) => Some(ResponseCode::OKAY),
            Some(Result::Err(e)) => Some(match e {
                AXI4Error::SLVERR => ResponseCode::SLVERR,
                AXI4Error::DECERR => ResponseCode::DECERR,
            }),
            None => None,
        };
        signal(d)
    }

    let expect = expect_file!["option_result_nested_option_result_destructure.expect"];
    let res = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    let res = res.err().unwrap();
    let report = miette_report(res);
    expect.assert_eq(&report);
    Ok(())
}

#[test]
fn test_option_result_nested_option_result_destructure_simple() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        _UNUSED,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<AXI4Error>, Red> {
        let resp = i.val();
        let d = match resp {
            Some(Result::Ok(())) => Some(AXI4Error::SLVERR),
            Some(Err(e)) => Some(e),
            None => None,
        };
        signal(d)
    }

    let expect_err = expect_file!["option_result_more.expect"];
    let res = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    let err = res.err().unwrap();
    let report = miette_report(err);
    expect_err.assert_eq(&report);
    Ok(())
}

#[test]
fn test_ok_err_variants_allowed_in_non_result() -> miette::Result<()> {
    // Check that we can use Ok and Err without the
    // compiler erroneously assuming its a standard Result type.
    #[derive(PartialEq, Debug, Digital)]
    pub enum MyResult {
        Ok(b8),
        AlsoOk(b8),
        Err(b8),
    }

    impl Default for MyResult {
        fn default() -> Self {
            Self::Err(bits(0))
        }
    }

    const OK_VAL: b8 = bits(10);
    const ALSO_OK_VAL: b8 = bits(20);

    #[kernel]
    fn kernel(x: b8) -> MyResult {
        match x {
            OK_VAL => MyResult::Ok(OK_VAL),
            ALSO_OK_VAL => MyResult::AlsoOk(ALSO_OK_VAL),
            _ => MyResult::Err(x),
        }
    }
    compile_design::<kernel>(CompilationMode::Synchronous)?;
    Ok(())
}
