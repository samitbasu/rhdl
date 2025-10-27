use crate::{HDLDescriptor, Kind, RHDLError, circuit::circuit_descriptor::CircuitType};

pub struct Descriptor {
    pub name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub circuit_type: CircuitType,
    pub hdl: Option<HDLDescriptor>,
    pub netlist: Option<HDLDescriptor>,
}
