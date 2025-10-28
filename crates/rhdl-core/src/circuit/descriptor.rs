use crate::{HDLDescriptor, Kind, RHDLError, circuit::circuit_descriptor::CircuitType, ntl, rtl};

pub struct Descriptor {
    pub name: String,
    pub input_kind: Kind,
    pub output_kind: Kind,
    pub d_kind: Kind,
    pub q_kind: Kind,
    pub circuit_type: CircuitType,
    pub kernel: Option<rtl::Object>,
    pub netlist: Option<ntl::Object>,
    pub hdl: Option<HDLDescriptor>,
}

impl Descriptor {
    pub fn hdl(&self) -> Result<&HDLDescriptor, RHDLError> {
        self.hdl.as_ref().ok_or(RHDLError::HDLNotAvailable {
            name: self.name.clone(),
        })
    }
    pub fn netlist(&self) -> Result<&ntl::Object, RHDLError> {
        self.netlist.as_ref().ok_or(RHDLError::NetlistNotAvailable {
            name: self.name.clone(),
        })
    }
}
