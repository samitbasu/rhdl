use crate::rhdl_core::{
    ast::source::source_location::SourceLocation,
    compiler::mir::error::{RHDLCompileError, ICE},
    error::rhdl_error,
    rtl::Object,
    RHDLError,
};

pub trait Pass {
    fn raise_ice(obj: &Object, cause: ICE, loc: SourceLocation) -> RHDLError {
        let symbols = &obj.symbols;
        rhdl_error(RHDLCompileError {
            cause,
            src: symbols.source(),
            err_span: symbols.span(loc).into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
    fn description() -> &'static str;
}
