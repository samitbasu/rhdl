use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::RHDLError,
    rhif::Object,
};

pub trait Pass {
    fn name() -> &'static str;
    fn description() -> &'static str;
    fn raise_ice(obj: &Object, cause: ICE, id: NodeId) -> RHDLError {
        RHDLError::RHDLInternalCompilerError(Box::new(RHDLCompileError {
            cause,
            src: obj.symbols.source.source.clone(),
            err_span: obj.symbols.node_span(id).into(),
        }))
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
}
