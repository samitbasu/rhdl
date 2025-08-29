use rhdl_vlog;
use std::collections::BTreeMap;

#[derive(Clone, Hash)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: rhdl_vlog::ModuleDef,
    pub children: BTreeMap<String, HDLDescriptor>,
}

impl HDLDescriptor {
    pub fn as_module(&self) -> rhdl_vlog::ModuleList {
        let mut module = self.body.clone();
        module
            .submodules
            .extend(self.children.values().map(HDLDescriptor::as_module));
        module
    }
}
