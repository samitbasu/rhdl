pub mod circuit_dq {
    use rhdl::prelude::*;
    // ANCHOR: circuit-dq
    pub trait CircuitDQ: 'static {
        type D: Timed;
        type Q: Timed;
    }
    // ANCHOR_END: circuit-dq
}

pub mod circuit_x {
    use rhdl::prelude::*;

    type A = std::marker::PhantomData<()>;
    type B = std::marker::PhantomData<()>;
    type C = std::marker::PhantomData<()>;

    impl CircuitDQ for X {
        type D = D;
        type Q = Q;
    }

    impl CircuitIO for X {
        type I = ();
        type O = ();
        type Kernel = NoCircuitKernel<(), Q, ((), D)>;
    }

    // ANCHOR: circuit-x
    #[derive(Circuit)]
    pub struct X {
        child_1: A,
        child_2: B,
        child_3: C,
    }
    // ANCHOR_END: circuit-x

    // ANCHOR: circuit-x-d
    #[derive(Digital, Timed, Clone, Copy, PartialEq)]
    pub struct D {
        child_1: <A as CircuitIO>::I,
        child_2: <B as CircuitIO>::I,
        child_3: <C as CircuitIO>::I,
    }
    // ANCHOR_END: circuit-x-d

    // ANCHOR: circuit-x-q
    #[derive(Digital, Timed, Clone, Copy, PartialEq)]
    pub struct Q {
        child_1: <A as CircuitIO>::O,
        child_2: <B as CircuitIO>::O,
        child_3: <C as CircuitIO>::O,
    }
    // ANCHOR_END: circuit-x-q
}

pub mod circuit_x_2 {
    use rhdl::prelude::*;

    type A = std::marker::PhantomData<()>;
    type B = std::marker::PhantomData<()>;
    type C = std::marker::PhantomData<()>;

    impl CircuitIO for X {
        type I = ();
        type O = ();
        type Kernel = NoCircuitKernel<(), XQ, ((), XD)>;
    }

    // ANCHOR: circuit-x-derive
    //                  👇 new!
    #[derive(Circuit, CircuitDQ)]
    pub struct X {
        child_1: A,
        child_2: B,
        child_3: C,
    }
    // ANCHOR_END: circuit-x-derive
}
