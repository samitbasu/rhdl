use crate::hdl::ast::Module;
use std::{collections::BTreeMap, iter::once};

#[derive(Clone)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: Module,
    pub children: BTreeMap<String, HDLDescriptor>,
}

impl HDLDescriptor {
    #[deprecated]
    pub fn as_verilog(&self) -> String {
        once(crate::hdl::formatter::module(&self.body))
            .chain(self.children.values().map(HDLDescriptor::as_verilog))
            .collect()
    }
    pub fn as_modules(&self) -> Vec<Module> {
        once(self.body.clone())
            .chain(self.children.values().flat_map(HDLDescriptor::as_modules))
            .collect()
    }
}
