use crate::{
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rhif::object::SourceLocation,
    rtl::Object,
    RHDLError,
};

pub trait Pass {
    fn name() -> &'static str;
    fn raise_ice(obj: &Object, cause: ICE, loc: SourceLocation) -> RHDLError {
        let symbols = &obj.symbols;
        rhdl_error(RHDLCompileError {
            cause,
            src: symbols.source(),
            err_span: symbols.span(loc).into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
}
