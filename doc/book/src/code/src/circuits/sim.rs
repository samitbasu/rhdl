use rhdl::prelude::*;

#[derive(Circuit, Clone, CircuitDQ, Default)]
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
