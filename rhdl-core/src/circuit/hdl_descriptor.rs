use crate::hdl::ast::Module;
use std::{collections::BTreeMap, iter::once};

#[derive(Clone)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: Module,
    pub children: BTreeMap<String, HDLDescriptor>,
}

impl HDLDescriptor {
    pub fn as_verilog(&self) -> String {
        once(crate::hdl::formatter::module(&self.body))
            .chain(self.children.values().map(HDLDescriptor::as_verilog))
            .collect()
    }
}
