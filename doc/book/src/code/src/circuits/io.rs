use rhdl::prelude::*;

// ANCHOR: circuit_io
pub trait CircuitIO: 'static + CircuitDQ {
    type I: Timed;
    type O: Timed;
    type Kernel: DigitalFn + DigitalFn2<A0 = Self::I, A1 = Self::Q, O = (Self::O, Self::D)>;
}

// ANCHOR_END: circuit_io

pub mod half_adder {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone, Copy)]
    pub struct HalfAdder;

    impl CircuitDQ for HalfAdder {
        type Q = ();
        type D = ();
    }

    #[kernel]
    pub fn half_adder(_i: Signal<(bool, bool), Red>, _q: ()) -> (Outputs, ()) {
        (
            Outputs {
                sum: signal(false),
                carry: signal(false),
            },
            (),
        )
    }

    // ANCHOR: half_adder_io

    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct Outputs {
        pub sum: Signal<bool, Red>,
        pub carry: Signal<bool, Red>,
    }

    impl CircuitIO for HalfAdder {
        type I = Signal<(bool, bool), Red>;
        type O = Outputs;
        type Kernel = half_adder; // 👈 function `half_adder` is decorated with #[kernel]
    }

    // ANCHOR_END: half_adder_io
}

pub mod xor_gate {
    use rhdl::prelude::*;

    // ANCHOR: xor_gate_kernel
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), ())
    }
    // ANCHOR_END: xor_gate_kernel
}
