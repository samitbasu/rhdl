use crate::ast;
use crate::kind::Kind;

#[derive(Debug, Clone)]
pub struct Kernel {
    pub code: Box<ast::Block>,
    pub args: Vec<(String, Kind)>,
    pub ret: Kind,
    pub name: String,
}
