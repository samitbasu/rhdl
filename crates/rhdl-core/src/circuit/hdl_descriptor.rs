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
        let mut modules = vec![self.body.clone()];
        for child in self.children.values() {
            modules.extend(child.as_module().into_iter());
        }
        rhdl_vlog::ModuleList { modules }
    }
}
