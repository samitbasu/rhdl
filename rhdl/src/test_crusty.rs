use rhdl_bits::Bits;
use rhdl_core::{
    compile_design,
    crusty::upstream::follow_pin_upstream,
    path::Path,
    schematic::{
        self, builder::build_schematic, components::ComponentKind, dot::write_dot,
        schematic_impl::pin_path, schematic_impl::Schematic,
    },
    DigitalFn, KernelFnKind,
};
use rhdl_macro::{kernel, Digital};

fn get_schematic<T: DigitalFn>() -> Schematic {
    let Some(KernelFnKind::Kernel(kernel)) = T::kernel_fn() else {
        panic!("Kernel function not found");
    };
    let module = compile_design(kernel).unwrap();
    build_schematic(&module, module.top).unwrap()
}

fn trace_reached_no_inputs(
    schematic: &Schematic,
    trace: &schematic::schematic_impl::Trace,
) -> bool {
    eprintln!("{:?}", trace);
    trace.sinks.iter().all(|sink| {
        matches!(
            schematic.component(schematic.pin(sink.pin).parent).kind,
            ComponentKind::Constant(_) | ComponentKind::Enum(_)
        )
    })
}

fn trace_reached_inputs_or_constant(
    schematic: &Schematic,
    trace: &schematic::schematic_impl::Trace,
) -> bool {
    eprintln!("{:?}", trace);
    trace.sinks.iter().all(|sink| {
        schematic.inputs.contains(&sink.pin)
            || matches!(
                schematic.component(schematic.pin(sink.pin).parent).kind,
                ComponentKind::Constant(_) | ComponentKind::Enum(_)
            )
    })
}

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

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
enum Bar {
    A(Bits<4>),
    B(Bits<4>),
    C(bool),
    #[default]
    D,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
struct Egg {
    a: [Bits<4>; 2],
    b: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
struct Nested {
    a: [Bits<4>; 2],
    b: bool,
    c: [Egg; 2],
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Default)]
struct Foo {
    a: Bits<4>,
    b: Bits<4>,
    c: bool,
    d: Bar,
    e: Nested,
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
