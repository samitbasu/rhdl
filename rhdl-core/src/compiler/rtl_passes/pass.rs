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
        let symbols = &obj.symbols[&loc.func];
        rhdl_error(RHDLCompileError {
            cause,
            src: symbols.source.source.clone(),
            err_span: symbols.node_span(loc.node).into(),
        })
    }
    fn run(input: Object) -> Result<Object, RHDLError>;
}
