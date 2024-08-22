use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;

#[test]
fn test_dynamic_vs_static_indexing_on_assign() -> miette::Result<()> {
    #[kernel]
    fn foo(mut a: [b4; 8]) -> b4 {
        let c = 3;
        let p = bits(4);
        a[c] = p; // Should compile down to a static index.
        a[c]
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
                | rhdl_core::rtl::spec::OpCode::DynamicSplice(_)
        ))
    });
    Ok(())
}

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
                | rhdl_core::rtl::spec::OpCode::DynamicSplice(_)
        ))
    });
    Ok(())
}

#[test]
fn test_dynamic_indexing_lowers_with_multiple_dimensions() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [[b4; 8]; 8]) -> b4 {
        let c = 3;
        let d = 4;
        a[c][d] // Should compile down to a static index.
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
fn test_dynamic_splice_lowers_with_multiple_dimensions() -> miette::Result<()> {
    #[kernel]
    fn foo(mut a: [[b4; 8]; 8]) -> b4 {
        let c = 3;
        let d = 4;
        a[c][d] = bits(5); // Should compile down to a static splice.
        a[c][d]
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicSplice(_)
                | rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
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
