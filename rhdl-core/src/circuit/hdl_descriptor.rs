use crate::hdl::ast::Module;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct HDLDescriptor {
    pub name: String,
    pub body: Module,
    pub children: BTreeMap<String, HDLDescriptor>,
}
