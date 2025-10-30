use crate::{
    HDLDescriptor, Kind, RHDLError,
    circuit::{circuit_descriptor::CircuitType, scoped_name::ScopedName},
    ntl, rtl,
};

#[derive(Debug)]
pub struct Descriptor {
    pub name: ScopedName,
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
        let hdl = self.hdl.as_ref().ok_or(RHDLError::HDLNotAvailable {
            name: self.name.to_string(),
        })?;
        hdl.modules.checked()?;
        Ok(hdl)
    }
    pub fn netlist(&self) -> Result<&ntl::Object, RHDLError> {
        self.netlist.as_ref().ok_or(RHDLError::NetlistNotAvailable {
            name: self.name.to_string(),
        })
    }
    pub fn with_netlist_black_box(mut self) -> Result<Descriptor, RHDLError> {
        match self.circuit_type {
            CircuitType::Asynchronous => {
                self.netlist = Some(ntl::builder::circuit_black_box(&self)?);
            }
            CircuitType::Synchronous => {
                self.netlist = Some(ntl::builder::synchronous_black_box(&self)?);
            }
        }
        Ok(self)
    }
}
