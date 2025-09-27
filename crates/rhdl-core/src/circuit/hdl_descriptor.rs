use rhdl_vlog;
use std::collections::BTreeMap;

/// A hardware description of a circuit.
///
/// This struct captures the HDL representation of a circuit,
/// including its name, body, and any child circuits it may have.
#[derive(Clone, Hash)]
pub struct HDLDescriptor {
    /// The unique name of the circuit.
    pub name: String,
    /// The HDL body of the circuit, typically a Verilog module definition.
    pub body: rhdl_vlog::ModuleDef,
    /// The child circuits of this circuit, if any.
    pub children: BTreeMap<String, HDLDescriptor>,
}

impl HDLDescriptor {
    /// Convert the HDL descriptor into a flat list of Verilog modules.
    pub fn as_module(&self) -> rhdl_vlog::ModuleList {
        let mut modules = vec![self.body.clone()];
        for child in self.children.values() {
            modules.extend(child.as_module().into_iter());
        }
        rhdl_vlog::ModuleList { modules }
    }
}
