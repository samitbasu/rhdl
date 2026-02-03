use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    pub fn kernel(a: (b8, b8)) -> b8 {
        let (x, y) = a;
        let z = (x, x, y);
        let a = z.0 + z.1 + z.2;
        a + 1
    }

    // ANCHOR_END: step_1
}
