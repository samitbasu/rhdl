use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1

    #[kernel]
    pub fn kernel(a: b32) -> b9 {
        let mut count = b9(0);
        for i in 0..32 {
            if a & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    pub fn count_ones<const N: usize, const M: usize>(a: Bits<N>) -> Bits<M>
    where
        rhdl::bits::W<N>: BitWidth,
        rhdl::bits::W<M>: BitWidth,
    {
        let mut count = bits::<M>(0);
        for i in 0..N {
            if a & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    #[kernel]
    pub fn kernel(a: b8) -> b4 {
        count_ones::<8, 4>(a)
    }

    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    pub fn xnor<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let mut ret_value = bits::<N>(0);
        for i in 0..N {
            let a_bit = a & (1 << i) != 0;
            let b_bit = b & (1 << i) != 0;
            if !(a_bit ^ b_bit) {
                ret_value |= 1 << i;
            }
        }
        ret_value
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8) -> b8 {
        xnor::<8>(a, b)
    }

    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    #[kernel]
    pub fn generic<const N: usize>(a: [bool; N], b: [bool; N]) -> [bool; N] {
        let mut ret_value = [false; N];
        for i in 0..N {
            ret_value[i] = !(a[i] ^ b[i]);
        }
        ret_value
    }

    #[kernel]
    pub fn kernel(a: [bool; 4], b: [bool; 4]) -> [bool; 4] {
        generic::<4>(a, b)
    }

    // ANCHOR_END: step_4
}
