use rhdl::prelude::*;

pub mod step_0 {
    use super::*;

    // ANCHOR: step_0
    #[kernel]
    pub fn kernel(a: b8, b: b8) -> (b8, bool) {
        (a, b == a)
    }
    // ANCHOR_END: step_0
}

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1

    #[kernel]
    pub fn kernel(_a: b8, _b: b8) -> () {}

    // ANCHOR_END: step_1

    #[ignore]
    // ANCHOR: step_1_test
    #[test]
    fn test_empty_return_kernel() -> miette::Result<()> {
        let _ = compile_design::<kernel>(CompilationMode::Asynchronous)?;
        Ok(())
    }
    // ANCHOR_END: step_1_test
}

pub mod step_2 {
    use super::*;

    // ANCHOR: step_2
    #[kernel]
    pub fn kernel((a, b): (b8, b8)) -> b8 {
        a + b
    }
    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;

    // ANCHOR: step_3
    #[kernel]
    pub fn generic_kernel<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        a + b
    }

    #[kernel]
    pub fn kernel(a: b7, b: b7) -> b7 {
        generic_kernel::<7>(a, b)
    }
    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;

    // ANCHOR: step_4
    #[derive(Copy, Clone, PartialEq, Digital)]
    pub struct MyStruct {
        x: b8,
        y: b8,
    }

    #[kernel]
    pub fn kernel(s: MyStruct) -> MyStruct {
        MyStruct {
            x: s.x + s.y,
            y: s.y,
        }
    }
    // ANCHOR_END: step_4
}

#[allow(clippy::let_and_return)]
pub mod step_5 {
    use super::*;

    // ANCHOR: step_5
    #[kernel]
    pub fn my_add<const N: usize>(a: Bits<N>, b: Bits<N>) -> Bits<N>
    where
        rhdl::bits::W<N>: BitWidth,
    {
        a + b
    }

    #[kernel]
    pub fn kernel(a: b8, b: b8, c: b8) -> b8 {
        //               👇 Must be explicit here!
        let p1 = my_add::<8>(a, b);
        let p2 = my_add::<8>(p1, c);
        p2
    }
    // ANCHOR_END: step_5
}
