use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(a: b8, b: b8) -> bool {
        let c = a * b - b;
        let c = a & c;
        let mut d = c + b;
        d >>= 2;
        !((d >= a) || (a == b))
    }
    // ANCHOR_END: step_1
}
