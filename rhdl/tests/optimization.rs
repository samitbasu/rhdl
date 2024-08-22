use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;

#[test]
fn test_dynamic_vs_static_indexing() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = 3;
        a[c] // Should compile down to a static index.
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
        ))
    });
    Ok(())
}

#[test]
fn test_shift_with_constant_argument() -> miette::Result<()> {
    #[kernel]
    fn foo(a: b4) -> b4 {
        a << 2 // Should compile down to a static shift.
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    eprintln!("{:?}", rtl);
    Ok(())
}

#[test]
fn test_constant_propagation_with_dynamic_indexing() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = 3;
        a[c + 1] // Should compile down to a static index.
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
        ))
    });
    Ok(())
}
