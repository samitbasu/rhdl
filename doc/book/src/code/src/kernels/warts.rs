use rhdl::prelude::*;

#[allow(dead_code)]
#[allow(unused_variables)]
pub mod step_1 {
    pub fn example() {
        // ANCHOR: step_1
        enum Color {
            Red(u8),
            Green(u8),
        }

        let x = Color::Red(4);
        // ANCHOR_END: step_1
    }
}

#[allow(dead_code)]
#[allow(clippy::let_and_return)]
pub mod step_2 {
    use super::*;

    #[derive(Digital, Clone, Copy, PartialEq)]
    enum Color {
        Red(b8),
        Green(b8),
    }

    impl Default for Color {
        fn default() -> Self {
            Color::Red(bits(0))
        }
    }

    // ANCHOR: step_2
    #[kernel]
    fn kernel(a: b8) -> Color {
        let x = Color::Red(a);
        x
    }
    // ANCHOR_END: step_2
}

#[cfg(feature = "lowercase_enum")]
pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[derive(Digital, Clone, Copy, PartialEq)]
    enum color {
        Red(b8),
        Green(b8),
    }

    impl Default for color {
        fn default() -> Self {
            color::Red(bits(0))
        }
    }

    #[kernel]
    fn kernel(a: b8) -> color {
        let x = color::Red(a);
        x
    }
    // ANCHOR_END: step_3
}

#[allow(clippy::needless_return)]
pub mod step_4 {
    pub fn example(x: i32, flag: bool, y: i32) -> Result<i32, i32> {
        // ANCHOR: step_4
        return if flag { Ok(x) } else { Err(y) };
        // ANCHOR_END: step_4
    }
}

#[cfg(feature = "const_generic_limits")]
pub mod step_5 {
    use super::*;
    use rhdl::bits::W;

    // ANCHOR: step_5
    fn extend<const N: usize>(a: Bits<N>) -> Bits<{ N + 1 }>
    where
        W<N>: BitWidth,
        W<{ N + 1 }>: BitWidth,
    {
        todo!()
    }

    // ANCHOR_END: step_5
}

pub mod step_6 {
    use super::*;
    // ANCHOR: step_6
    pub fn extend<const N: usize, const NP1: usize>(_a: Bits<N>) -> Bits<NP1>
    where
        rhdl::bits::W<N>: BitWidth,
        rhdl::bits::W<NP1>: BitWidth,
    {
        assert_eq!(N + 1, NP1);
        todo!()
    }

    // ANCHOR_END: step_6
}
