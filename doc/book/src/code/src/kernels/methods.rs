use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    pub fn kernel(a: b8, b: s8) -> bool {
        let x = a.any();
        let y = b.any();
        let x = x && a.all();
        let y = y && b.all();
        let x = x && a.xor();
        let y = y && b.xor();
        let a_as_s8: s8 = a.as_signed();
        let b_as_b8: b8 = b.as_unsigned();
        (a_as_s8 == b) && (b_as_b8 == a) && x && y
    }

    // ANCHOR_END: step_1
}
