use rhdl::prelude::*;

#[allow(clippy::let_and_return)]
pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let c = a == b; // type is inferred as bool
        let d = !c;
        d
    }
    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let mut c = a + 1;
        c += a; // mutates c
        c == b
    }
    // ANCHOR_END: step_2
}

#[allow(clippy::needless_late_init)]
pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    pub fn kernel(a: b8, b: b8) -> bool {
        let c;
        let d = a + b;
        c = d;
        c == a
    }
    // ANCHOR_END: step_3
}

pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    #[kernel]
    pub fn kernel(a: b8, b: b8) -> b8 {
        let c: b8 = a;
        c + b
    }
    // ANCHOR_END: step_4
}

pub mod step_5 {
    use super::*;
    // ANCHOR: step_5
    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    pub fn kernel(arg: Foo) -> b8 {
        let Foo { a, b } = arg;
        a + b
    }
    // ANCHOR_END: step_5
}

pub mod step_6 {
    use super::*;
    // ANCHOR: step_6
    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo(b8);

    #[kernel]
    pub fn kernel(arg: Foo) -> Foo {
        let Foo(a) = arg;
        Foo(a + 1)
    }
    // ANCHOR_END: step_6
}

pub mod step_7 {
    use super::*;
    // ANCHOR: step_7
    #[kernel]
    pub fn kernel(a: (b8, b8)) -> bool {
        let (c, d) = a;
        c == d
    }
    // ANCHOR_END: step_7
}

pub mod step_8 {
    use super::*;
    // ANCHOR: step_8
    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Bar(b8, b8);

    #[derive(PartialEq, Clone, Copy, Digital)]
    pub struct Foo {
        a: b8,
        b: Bar,
    }

    #[kernel]
    pub fn kernel(state: Signal<Foo, Red>) -> Signal<b8, Red> {
        let Foo { a, b: Bar(_x, y) } = state.val();
        signal((a + y).resize())
    }
    // ANCHOR_END: step_8
}
