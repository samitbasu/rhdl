use rhdl::prelude::*;

#[allow(clippy::needless_range_loop)]
pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(x: [b4; 4]) -> b6 {
        let mut accum = b6(0);
        for i in 0..4 {
            accum += x[i].resize::<6>();
        }
        accum
    }
    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    fn kernel(x: [b4; 8], ndx: b3) -> b4 {
        x[ndx]
    }
    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    fn kernel(x: b8, ndx: b3) -> bool {
        // 👇 - implies a barrel shifter
        (x & b8(1) << ndx) != 0
    }
    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    #[kernel]
    fn kernel() -> [b4; 4] {
        [b4(3); 4]
    }
    // ANCHOR_END: step_4
}
