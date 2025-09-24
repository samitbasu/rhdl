use rhdl::prelude::*;

#[derive(Digital, PartialEq, Default)]
pub enum OpCode {
    #[default]
    Add,
    And,
    Or,
    Xor,
}

#[derive(Circuit, Clone)]
pub struct Alu;

impl CircuitDQ for Alu {
    type D = ();
    type Q = ();
}

impl CircuitIO for Alu {
    type I = Signal<(OpCode, b4, b4), Green>;
    type O = Signal<b4, Green>;
    type Kernel = alu; // 👈 doesn't exist yet
}

#[kernel]
pub fn alu(i: Signal<(OpCode, b4, b4), Green>, _q: ()) -> (Signal<b4, Green>, ()) {
    let (opcode, a, b) = i.val();
    let c = match opcode {
        OpCode::Add => a + b,
        OpCode::And => a & b,
        OpCode::Or => a | b,
        OpCode::Xor => a ^ b,
    };
    (signal(c), ())
}
