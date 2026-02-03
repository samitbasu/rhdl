use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x {
            Bits::<8>(0) => b3(0),
            Bits::<8>(1) => b3(1),
            Bits::<8>(3) => b3(2),
            _ => b3(5),
        }
    }

    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x.raw() {
            0 => b3(0),
            1 => b3(1),
            3 => b3(2),
            _ => b3(5),
        }
    }

    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    pub const NO_DATA: b8 = b8(0);
    pub const SINGLE: b8 = b8(1);
    pub const MULTIPLE: b8 = b8(3);

    #[kernel]
    pub fn kernel(x: b8) -> b3 {
        match x {
            NO_DATA => b3(0),
            SINGLE => b3(1),
            MULTIPLE => b3(2),
            _ => b3(5),
        }
    }

    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    //       👇 namespace the raw constants in a module
    pub mod error_codes {
        use super::*;
        pub const ALL_OK: b2 = b2(0);
        pub const ENDPOINT_ERROR: b2 = b2(1);
        pub const ADDRESS_ERROR: b2 = b2(2);
        pub const RESERVED_ERROR: b2 = b2(3);
    }

    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum BusError {
        // 👈 Create a RHDL enum for the variants
        Endpoint,
        Address,
        #[default]
        Reserved,
    }

    #[kernel]
    pub fn kernel(x: b2, data: b8) -> Result<b8, BusError> {
        match x {
            error_codes::ALL_OK => Ok(data),
            error_codes::ENDPOINT_ERROR => Err(BusError::Endpoint),
            error_codes::ADDRESS_ERROR => Err(BusError::Address),
            error_codes::RESERVED_ERROR => Err(BusError::Reserved),
            _ => Err(BusError::Reserved), // 👈 unreachable but rustc doesn't know this
        }
    }

    // ANCHOR_END: step_4
}

pub mod step_5 {
    #![allow(unused)]
    use super::*;
    // ANCHOR: step_5
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    pub enum SimpleEnum {
        #[default]
        Init,
        Run(b8),
        Point {
            x: b4,
            y: b8,
        },
        Boom,
    }

    #[kernel]
    fn kernel(state: SimpleEnum) -> b8 {
        match state {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
        }
    }

    // ANCHOR_END: step_5
}

pub mod step_5b {
    use super::*;
    // ANCHOR: step_5b
    #[derive(PartialEq, Debug, Digital, Default, Clone, Copy)]
    pub enum MyEnum {
        #[default]
        Idle,
        Run(b8),
    }

    #[kernel]
    fn kernel(state: Option<MyEnum>) -> b8 {
        match state {
            Some(MyEnum::Idle) => bits(1),
            Some(MyEnum::Run(x)) => x,
            None => bits(0),
        }
    }
    // ANCHOR_END: step_5b

    #[ignore]
    // ANCHOR: step_5b_test
    #[test]
    fn test_nested_match_compile_error() -> miette::Result<()> {
        let _ = compile_design::<kernel>(CompilationMode::Asynchronous)?;
        Ok(())
    }
    // ANCHOR_END: step_5b_test
}

pub mod step_6 {
    use super::*;
    // ANCHOR: step_6
    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct Point {
        x: b8,
        y: b8,
    }

    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct Reflect(pub Point);

    #[kernel]
    pub fn kernel(x: Reflect) -> b8 {
        let Reflect(p) = x;
        let Point { x, y: _ } = p;
        x
    }

    // ANCHOR_END: step_6
}

pub mod step_7 {
    use super::*;
    // ANCHOR: step_7

    #[kernel]
    pub fn kernel(x: Option<b8>) -> Option<b8> {
        if let Some(v) = x { Some(v + 1) } else { None }
    }

    // ANCHOR_END: step_7
}
