use std::marker::PhantomData;

use rhdl::{
    core::{AsyncKind, ScopedName},
    prelude::*,
};

// ANCHOR: circuit-trait
pub trait Circuit: 'static + CircuitIO + Sized {
    // snip
    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError>;
    // snip
}
// ANCHOR_END: circuit-trait

// ANCHOR: descriptor
pub struct Descriptor<T> {
    // snip
    pub hdl: Option<HDLDescriptor>,
    /// Phantom data for the marker type.
    pub _phantom: PhantomData<T>,
}
// ANCHOR_END: descriptor

// ANCHOR: hdl-descriptor
#[derive(Clone, Hash, Debug)]
pub struct HDLDescriptor {
    /// The unique name of the circuit.
    pub name: String,
    /// The list of modules that make up this circuit.
    pub modules: vlog::ModuleList,
}
// ANCHOR_END: hdl-descriptor

pub mod and_gate {
    use rhdl::{core::ScopedName, prelude::*};
    #[derive(Circuit, Clone, CircuitDQ, Default)]
    pub struct AndGate;

    impl CircuitIO for AndGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = and_gate;
    }

    #[kernel]
    pub fn and_gate(i: Signal<(bool, bool), Red>, _q: Q) -> (Signal<bool, Red>, D) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a & b; // AND operation
        (signal(c), D {})
    }

    #[test]
    fn test_and_gate() -> miette::Result<()> {
        type T = AndGate;
        // ANCHOR: verilog
        let uut = T::default(); // 👈 or whatever
        // Get the run time descriptor
        let desc = uut.descriptor(ScopedName::top())?;
        // Gets a reference to the checked HDL descriptor
        let hdl = desc.hdl()?;
        // Pretty print the Verilog
        println!("{}", hdl.modules.pretty());
        // ANCHOR_END: verilog
        std::fs::write("and_gate.v", hdl.modules.pretty()).unwrap();
        Ok(())
    }
}
