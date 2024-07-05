use std::collections::HashMap;

use crate::{error::RHDLError, root_verilog, Circuit, HDLKind, Synchronous};

use super::synchronous_verilog::root_synchronous_verilog;

#[derive(Clone)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: String,
    pub children: HashMap<String, HDLDescriptor>,
}

impl std::fmt::Debug for HDLDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.body)?;
        for hdl in self.children.values() {
            writeln!(f, "{:?}", hdl)?;
        }
        Ok(())
    }
}

impl HDLDescriptor {
    pub fn add_child<C: Circuit>(
        &mut self,
        name: &str,
        circuit: &C,
        kind: HDLKind,
    ) -> Result<(), RHDLError> {
        self.children.insert(name.into(), circuit.as_hdl(kind)?);
        Ok(())
    }
    pub fn add_synchronous<S: Synchronous>(
        &mut self,
        name: &str,
        synchronous: &S,
        kind: HDLKind,
    ) -> Result<(), RHDLError> {
        self.children.insert(name.into(), synchronous.as_hdl(kind)?);
        Ok(())
    }
}

pub fn root_hdl<C: Circuit>(circuit: &C, kind: HDLKind) -> Result<HDLDescriptor, RHDLError> {
    match kind {
        HDLKind::Verilog => root_verilog(circuit),
    }
}

pub fn root_synchronous_hdl<S: Synchronous>(
    synchronous: &S,
    kind: HDLKind,
) -> Result<HDLDescriptor, RHDLError> {
    match kind {
        HDLKind::Verilog => root_synchronous_verilog(synchronous),
    }
}
