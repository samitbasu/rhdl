use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(a: b8, b: b8) -> b8 {
        let c;
        if a > b {
            c = b8(3);
        } else if a == b {
            c = b8(5);
        } else {
            c = b8(7);
        }
        c
    }
    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    fn kernel(a: b8, b: b8) -> b8 {
        bits(if a > b {
            3
        } else if a == b {
            5
        } else {
            7
        })
    }
    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    pub fn kernel(data: Option<b8>) -> Option<b8> {
        if let Some(data) = data {
            Some(data + 1)
        } else {
            None
        }
    }
    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum MyEnum {
        Red(b8),
        Green(b8, b8, b8),
        #[default]
        Blue,
    }

    #[kernel]
    pub fn kernel(data: MyEnum) -> b8 {
        if let MyEnum::Red(x) = data { x } else { b8(42) }
    }
    // ANCHOR_END: step_4
}
