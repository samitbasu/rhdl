use std::collections::HashMap;

use crate::{root_verilog, Circuit, HDLKind};

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
    ) -> anyhow::Result<()> {
        self.children.insert(name.into(), circuit.as_hdl(kind)?);
        Ok(())
    }
}

pub fn root_hdl<C: Circuit>(circuit: &C, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
    match kind {
        HDLKind::Verilog => root_verilog(circuit),
    }
}
