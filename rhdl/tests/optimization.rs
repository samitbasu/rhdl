use rhdl::prelude::*;
use rhdl_core::{
    rhif::spec::AluBinary, rtl::spec::Binary, sim::testbench::kernel::test_kernel_vm_and_verilog,
};
#[cfg(test)]
mod common;

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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
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
fn test_left_signed_shift_with_constant_argument() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<s8, Red>) -> Signal<s8, Red> {
        let a = a.val();
        let a = a << 2; // Should compile down to an indexing operation.
        signal(a)
    }
    let rtl = compile_design::<foo>(CompilationMode::Asynchronous)?;
    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::Binary(Binary {
                op: AluBinary::Shl,
                ..
            })
        ))
    });
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, s8_red())?;
    Ok(())
}

#[test]
fn test_left_shift_with_constant_argument() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let a = a << 2; // Should compile down to an indexing operation.
        signal(a)
    }
    let rtl = compile_design::<foo>(CompilationMode::Asynchronous)?;
    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::Binary(Binary {
                op: AluBinary::Shl,
                ..
            })
        ))
    });
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_right_shift_with_constant_argument() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let a = a >> 2; // Should compile down to an indexing operation.
        signal(a)
    }
    let rtl = compile_design::<foo>(CompilationMode::Asynchronous)?;
    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::Binary(Binary {
                op: AluBinary::Shr,
                ..
            })
        ))
    });
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_right_signed_shift_with_constant_argument() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<s8, Red>) -> Signal<s8, Red> {
        let a = a.val();
        let a = a >> 2; // Should compile down to an indexing operation.
        signal(a)
    }
    let rtl = compile_design::<foo>(CompilationMode::Asynchronous)?;
    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::Binary(Binary {
                op: AluBinary::Shr,
                ..
            })
        ))
    });
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, s8_red())?;
    Ok(())
}

#[test]
fn test_constant_propagation_with_dynamic_indexing() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = 3;
        a[c + 1] // Should compile down to a static index.
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
        let d = if d > 5 { d } else { 4 };
        a[d] // Should compile down to a static splice.
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    #[derive(PartialEq, Digital)]
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    #[derive(PartialEq, Digital)]
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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    #[derive(PartialEq, Digital, Default)]
    enum FooEnum {
        #[default]
        A,
        B(b4),
    }

    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = FooEnum::A;
        let d = match c {
            FooEnum::A => 3,
            FooEnum::B(_x) => 5,
        };
        a[d] // Should compile down to a static splice.
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
#[allow(unused_assignments)]
fn test_constant_propagation_through_assigns() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let mut c = 3;
        c = 4;
        let d = c;
        a[d] // Should compile down to a static index.
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

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
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
        ))
    });
    Ok(())
}

#[test]
fn test_constant_propagation_through_sub_kernels() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [b4; 8]) -> b4 {
        let c = bits(3);
        let d = bar(c);
        a[d]
    }

    #[kernel]
    fn bar(a: b4) -> b4 {
        a + 1
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;

    eprintln!("{:?}", rtl);
    rtl.ops.iter().for_each(|op| {
        assert!(!matches!(
            op.op,
            rhdl_core::rtl::spec::OpCode::DynamicIndex(_)
        ))
    });
    Ok(())
}

/*
Object foo
  fn_id FnID(d86f2bbbdf22045f)
  arguments [Some(r1)]
  return_register r4
Reg r0 : b0
Reg r1 : b1
Reg r2 : b1
Reg r3 : b0
Reg r4 : b1
// let (c, d, ) = a;

r0 <- r1[0..0]
r2 <- r1[0..1]
// let q = (c, c, );

r3 <- { r0, r0 }
// (q, d, )

r4 <- { r3, r2 }


*/
#[test]
fn test_empty_expressions_dropped() -> miette::Result<()> {
    #[kernel]
    fn foo(a: ((), bool)) -> (((), ()), bool) {
        let (c, d) = a;
        let p = ();
        let q = (c, p);
        (q, d)
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}

#[test]
fn test_empty_splices_dropped() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    struct Foo {
        a: b1,
        b: (),
    }

    #[kernel]
    fn foo(mut a: Foo, c: ()) -> Foo {
        a.b = c;
        a
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}

#[test]
#[allow(clippy::let_unit_value)]
fn test_empty_index_dropped() -> miette::Result<()> {
    #[kernel]
    fn foo(a: ([(); 8], b1), b: b1) -> ((), b1) {
        let c = a.0[3];
        (c, b)
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}

#[test]
fn test_empty_dynamic_splices_dropped() -> miette::Result<()> {
    #[kernel]
    fn foo(mut a: ([(); 8], b1), b: b3) -> ([(); 8], b1) {
        a.0[b] = ();
        a
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}

#[test]
fn test_empty_dynamic_index_dropped() -> miette::Result<()> {
    #[kernel]
    fn foo(a: ([(); 8], b1), b: b3) -> ((), b1) {
        (a.0[b], a.1)
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    assert!(rtl
        .ops
        .iter()
        .all(|op| matches!(op.op, rhdl_core::rtl::spec::OpCode::Comment(_))));
    Ok(())
}

#[test]
fn test_lower_multiplies_to_shifts() -> miette::Result<()> {
    #[kernel]
    fn foo(a: [[bool; 8]; 8], b: b3, c: b3) -> bool {
        a[b][c]
    }

    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.ops.iter().all(|op| !matches!(
        op.op,
        rhdl_core::rtl::spec::OpCode::Binary(Binary {
            op: AluBinary::Mul,
            ..
        })
    )));
    Ok(())
}

#[test]
#[allow(unused_variables)]
fn test_empty_indices_dropped() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    struct Foo {
        a: b1,
        b: (),
    }

    #[kernel]
    fn foo(a: Foo) -> b1 {
        let () = a.b;
        let _c = (a.b, ());
        a.a
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}

#[test]
#[allow(clippy::let_unit_value)]
#[allow(unused_variables)]
fn test_empty_case_dropped() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Default)]
    enum Color {
        #[default]
        Red,
        Green,
        Blue,
        Black,
    }

    #[kernel]
    fn foo(a: Color) -> b4 {
        let ret;
        let _mt = match a {
            Color::Red => {
                ret = bits(1);
            }
            Color::Green => {
                ret = bits(2);
            }
            Color::Blue => {
                ret = bits(3);
            }
            Color::Black => {
                ret = bits(4);
            }
        };
        ret
    }
    let rtl = compile_design::<foo>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", rtl);
    assert!(rtl.register_kind.values().all(|v| !v.is_empty()));
    assert!(rtl.literals.values().all(|v| !v.is_empty()));
    Ok(())
}
