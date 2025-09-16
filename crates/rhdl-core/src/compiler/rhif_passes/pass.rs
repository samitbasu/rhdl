use crate::{
    ast::SourceLocation,
    compiler::mir::error::{ICE, RHDLCompileError},
    error::{RHDLError, rhdl_error},
    rhif::Object,
};

pub trait Pass {
    fn raise_ice(obj: &Object, cause: ICE, loc: SourceLocation) -> RHDLError {
        rhdl_error(RHDLCompileError {
            cause,
            src: obj.symbols.source(),
            err_span: obj.symbols.span(loc).into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
    fn description() -> &'static str;
}
