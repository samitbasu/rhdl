#![allow(unused_variables)]
use rhdl_bits::Bits;
use rhdl_core::{
    compile_design,
    crusty::upstream::follow_pin_upstream,
    schematic::{
        self, builder::build_schematic, components::ComponentKind, dot::write_dot,
        schematic_impl::pin_path, schematic_impl::Schematic,
    },
    types::path::Path,
    DigitalFn, KernelFnKind,
};
use rhdl_macro::{kernel, Digital};

use crate::test_utils::{
    get_schematic, trace_reached_inputs_or_constant, trace_reached_no_inputs, Bar, Egg, Foo,
};

#[test]
fn test_upstream_binary() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> (Bits<4>, Bits<4>) {
        (a + b, a - b)
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default().index(1)),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_array() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Bits::<4>(0);
        let b = [c, a, c];
        b[1]
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_array_repeated() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Bits::<4>(0);
        let b = [c, a, c, a];
        b[3]
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_case() {
    #[kernel]
    fn func(a: bool, b: Bits<4>) -> Bits<4> {
        match a {
            true => b,
            false => b + 1,
        }
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_enum() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Bar::A(a);
        match c {
            Bar::A(b) => b,
            Bar::B(b) => b + 1,
            Bar::C(_b) => Bits::<4>(1),
            Bar::D => Bits::<4>(0),
        }
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_enum_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_index() {
    #[kernel]
    fn func(a: Foo) -> Bits<4> {
        a.b
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_index_nested() {
    #[kernel]
    fn func(a: Foo) -> Bits<4> {
        a.e.a[1]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_index_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Bits<4>) -> Bits<4> {
        let d = Bits::<4>(0);
        let c = [d, b];
        c[a]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_index_nested_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Foo) -> Bits<4> {
        b.e.a[a]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_index_double_nested_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Bits<2>, c: Foo) -> Bits<4> {
        c.e.c[a].a[b]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_index_double_nested_dynamic_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_repeat() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let b = [a; 4];
        b[3]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_select() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>, c: bool) -> Bits<4> {
        if c {
            a
        } else {
            b
        }
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_select_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_splice() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let mut c = Foo::default();
        c.e.a[1] = a;
        c.e.a[1]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_splice_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_rejects_illegal_query() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let mut c = Foo::default();
        c.e.a[0] = a;
        c.e.a[1]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    assert!(follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default().index(0)),
    )
    .is_err());
}

#[test]
fn test_upstream_splice_no_pass() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let mut c = Foo::default();
        c.e.a[0] = a;
        c.e.a[1]
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_splice_no_pass_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_no_inputs(&schematic, &trace));
}

#[test]
fn test_upstream_struct() {
    #[kernel]
    fn func(a: bool) -> bool {
        let rest = Egg::default();
        let c = Egg { b: a, ..rest };
        c.b
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(schematic.output, Path::default()),
    )
    .unwrap();

    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_struct_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_struct_via_rest() {
    #[kernel]
    fn func(a: bool) -> bool {
        let rest = Egg::default();
        let c = Egg { b: a, ..rest };
        let d = Egg { a: rest.a, ..c };
        d.b
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(schematic.output, Path::default()),
    )
    .unwrap();

    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_struct_via_rest_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_tuple() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> Bits<4> {
        let c = (a, b);
        c.1
    }

    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("upstream_tuple_works.dot").unwrap(),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_unary() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        a + 1
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_inputs_or_constant(&schematic, &trace));
}

#[test]
fn test_upstream_constant() {
    #[kernel]
    fn func() -> Bits<4> {
        Bits::<4>(0)
    }
    let schematic = get_schematic::<func>();
    let output_pin = schematic.output;
    let trace = follow_pin_upstream(
        &schematic.clone().into(),
        pin_path(output_pin, Path::default()),
    )
    .unwrap();
    assert!(trace_reached_no_inputs(&schematic, &trace));
}
