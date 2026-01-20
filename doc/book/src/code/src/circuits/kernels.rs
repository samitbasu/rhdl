use rhdl::prelude::*;

pub trait Foo {
    type I;
    type O;
    type Q;
    type D;
    // ANCHOR: kernel-def
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
    // ANCHOR_END: kernel-def
}

pub mod xor {
    use rhdl::prelude::*;

    #[derive(Digital, Clone, Copy, PartialEq, Timed)]
    pub struct EmptyThing;

    #[derive(Circuit, Clone)]
    pub struct XorGate;

    #[derive(PartialEq, Digital, Clone, Copy, Timed)]
    #[doc(hidden)]
    pub struct Q;

    #[derive(PartialEq, Digital, Clone, Copy, Timed)]
    #[doc(hidden)]
    pub struct D;

    #[derive(PartialEq, Clone, Copy)]
    pub struct EmptyStruct;

    impl rhdl::core::Digital for EmptyStruct {
        const BITS: usize = 0;
        fn static_kind() -> rhdl::core::Kind {
            rhdl::core::Kind::make_struct(stringify!(EmptyStruct), [].into())
        }
        fn bin(self) -> Box<[rhdl::core::BitX]> {
            [].into()
        }
        fn dont_care() -> Self {
            Self {}
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    pub struct EmptyStruct2 {}

    impl rhdl::core::Digital for EmptyStruct2 {
        const BITS: usize = 0;
        fn static_kind() -> rhdl::core::Kind {
            rhdl::core::Kind::make_struct(stringify!(EmptyStruct2), [].into())
        }
        fn bin(self) -> Box<[rhdl::core::BitX]> {
            [].into()
        }
        fn dont_care() -> Self {
            Self {}
        }
    }

    impl rhdl::core::CircuitDQ for XorGate {
        type Q = Q;
        type D = D;
    }

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate;
    }

    #[kernel]
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: Q) -> (Signal<bool, Red>, D) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), D {})
    }
}

pub mod and {
    use rhdl::prelude::*;
    #[derive(Circuit, Clone, CircuitDQ)]
    pub struct AndGate;

    impl CircuitIO for AndGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = and_gate;
    }

    #[kernel]
    pub fn and_gate(i: Signal<(bool, bool), Red>, _q: AndGateQ) -> (Signal<bool, Red>, AndGateD) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a & b; // AND operation
        (signal(c), AndGateD {})
    }
}

// ANCHOR: half-adder-outputs
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Outputs {
    pub sum: Signal<bool, Red>,
    pub carry: Signal<bool, Red>,
}
// ANCHOR_END: half-adder-outputs

// ANCHOR: half-adder
#[derive(Circuit, Clone)]
pub struct HalfAdder {
    xor: xor::XorGate,
    and: and::AndGate,
}
// ANCHOR_END: half-adder

// ANCHOR: half-adder-io
impl CircuitIO for HalfAdder {
    type I = Signal<(bool, bool), Red>;
    type O = Outputs;
    type Kernel = half_adder;
}
// ANCHOR_END: half-adder-io

// ANCHOR: half-adder-d
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct D {
    xor: <xor::XorGate as CircuitIO>::I,
    and: <and::AndGate as CircuitIO>::I,
}
// ANCHOR_END: half-adder-d

// ANCHOR: half-adder-q
#[derive(Digital, Copy, Clone, Timed, PartialEq)]
pub struct Q {
    xor: <xor::XorGate as CircuitIO>::O,
    and: <and::AndGate as CircuitIO>::O,
}
// ANCHOR_END: half-adder-q

impl CircuitDQ for HalfAdder {
    type Q = Q;
    type D = D;
}

// ANCHOR: half-adder-kernel
#[kernel]
pub fn half_adder(i: Signal<(bool, bool), Red>, q: Q) -> (Outputs, D) {
    // D is the set of inputs for the internal components
    let d = D {
        xor: i,
        and: i, // 👈 Digital : Copy, so no cloning needed
    };
    // Q is the output of those internal components
    let o = Outputs {
        sum: q.xor,
        carry: q.and,
    };
    (o, d)
}
// ANCHOR_END: half-adder-kernel
