use crate::{
    prelude::RHDLError,
    rhdl_core::{
        ast::source::source_location::SourceLocation,
        compiler::mir::error::{RHDLCompileError, ICE},
        error::rhdl_error,
        ntl::object::Object,
    },
};

pub trait Pass {
    fn raise_ice(obj: &Object, cause: ICE, loc: SourceLocation) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: obj.code.source(),
            err_span: obj.code.span(loc).into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
    fn description() -> &'static str;
}
