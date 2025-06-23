use crate::{
    prelude::RHDLError,
    rhdl_core::{
        compiler::mir::error::{RHDLCompileError, ICE},
        error::rhdl_error,
        ntl::object::{Object, SourceOpCode},
    },
};

pub trait Pass {
    fn raise_ice(obj: &Object, cause: ICE, loc: Option<SourceOpCode>) -> RHDLError {
        let err_span = if let Some(source_op) = loc {
            obj.code.span(source_op)
        } else {
            0..0
        };
        rhdl_error(RHDLCompileError {
            cause,
            src: obj.code.source(),
            err_span: err_span.into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
    fn description() -> &'static str;
}
