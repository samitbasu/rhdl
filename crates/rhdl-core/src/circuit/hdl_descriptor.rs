use crate::rhdl_core::hdl::ast::Module;
use std::collections::BTreeMap;

#[derive(Clone, Hash)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: Module,
    pub children: BTreeMap<String, HDLDescriptor>,
}

impl HDLDescriptor {
    pub fn as_module(&self) -> Module {
        let mut module = self.body.clone();
        module
            .submodules
            .extend(self.children.values().map(HDLDescriptor::as_module));
        module
    }
}
