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

#[test]
fn test_constant_propagation_with_array() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = [3, 4];
        a[c[0] + c[1]] // Should compile down to a static index.
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
fn test_constant_propagation_with_tuple() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = (3, 4);
        let d = c.0 + c.1;
        a[d] // Should compile down to a static index.
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
fn test_constant_propogation_with_select() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let mut c = [3, 4];
        c[1] = 2;
        let d = c[0] + c[1];
        let d = if (d > 5) { d } else { 4 };
        a[d] // Should compile down to a static splice.
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
fn test_constant_propogation_with_splice() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let mut c = [3, 4];
        c[1] = 2;
        let d = c[0] + c[1];
        a[d] // Should compile down to a static splice.
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
fn test_constant_propogation_with_struct() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Digital)]
    struct FooStruct {
        a: b4,
        b: b4,
    }

    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = FooStruct {
            a: bits(3),
            b: bits(4),
        };
        let d = c.a + c.b;
        a[d] // Should compile down to a static splice.
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
fn test_constant_propogation_with_struct_with_rest() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Digital)]
    struct FooStruct {
        a: b4,
        b: b4,
    }

    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let q = FooStruct {
            a: bits(3),
            b: bits(4),
        };
        let c = FooStruct { a: bits(3), ..q };
        let d = c.a + c.b;
        a[d] // Should compile down to a static splice.
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
fn test_constant_propogation_with_enum() -> miette::Result<()> {
    #[derive(Copy, Clone, PartialEq, Digital)]
    enum FooEnum {
        A,
        B(b4),
    }

    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = FooEnum::A;
        let d = match c {
            FooEnum::A => 3,
            FooEnum::B(x) => 5,
        };
        a[d] // Should compile down to a static splice.
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
fn test_constant_propagation_through_assigns() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let mut c = 3;
        c = 4;
        let d = c;
        a[d] // Should compile down to a static index.
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
fn test_constant_propagation_through_loops() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let mut c = 3;
        for _k in 0..4 {
            c += 1;
        }
        a[c] // Should compile down to a static index.
    }
    let module = compile_design::<foo>(CompilationMode::Synchronous)?;
    let rtl = compile_top(&module)?;
    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
        ))
    });
    Ok(())
}
