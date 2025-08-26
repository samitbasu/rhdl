use crate::rhdl_core::ast::ast_impl::{FunctionId, NodeId};

#[derive(Debug, Clone, Copy, PartialEq, Hash, PartialOrd, Eq, Ord)]
pub struct SourceLocation {
    pub func: FunctionId,
    pub node: NodeId,
}

impl From<(FunctionId, NodeId)> for SourceLocation {
    fn from((func, node): (FunctionId, NodeId)) -> Self {
        Self { func, node }
    }
}
