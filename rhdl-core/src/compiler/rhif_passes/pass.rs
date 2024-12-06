use crate::{
    ast::source::source_location::SourceLocation,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::{rhdl_error, RHDLError},
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
}
