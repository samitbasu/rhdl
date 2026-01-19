use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(a: b8, b: b8) -> b8 {
        let c = {
            let d = a;
            let e = a + d;
            e + 3 // 👈 block value computed from this expression
        };
        a + c - b
    }
    // ANCHOR_END: step_1
}
