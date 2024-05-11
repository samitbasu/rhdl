#![allow(unused_variables)]
use rhdl_bits::Bits;
use rhdl_core::{
    crusty::downstream::follow_pin_downstream,
    path::Path,
    schematic::{self, dot::write_dot, schematic_impl::pin_path},
};
use rhdl_macro::kernel;

use crate::test_utils::{get_schematic, trace_reached_output, Bar, Foo, Nested};

#[test]
fn test_downstream_array() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> [Bits<4>; 2] {
        [a + b, b]
    }
    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default().index(0))));
    assert!(!trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default().index(1))));
}

#[test]
fn test_downstream_array_repeated() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> [Bits<4>; 2] {
        [a + b, a + b]
    }
    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default().index(0))));
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default().index(1))));
}

#[test]
fn test_downstream_binary() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> Bits<4> {
        a + b
    }
    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_case() {
    #[kernel]
    fn func(a: bool, b: Bits<4>) -> Bits<4> {
        match a {
            true => b,
            false => Bits::<4>::default(),
        }
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[1];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_enum() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Bar::A(a);
        match c {
            Bar::A(b) => b,
            Bar::B(b) => b,
            Bar::C(b) => Bits::<4>(1),
            Bar::D => Bits::<4>(0),
        }
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("downstream_enum.dot").unwrap(),
    )
    .unwrap();

    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_index() {
    #[kernel]
    fn func(a: Foo) -> Bits<4> {
        let c = a.b;
        c + Bits::<4>(1)
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("b")),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())));
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("c")),
    )
    .unwrap();
    assert!(!trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_index_nested() {
    #[kernel]
    fn func(a: Foo) -> Bits<4> {
        let c = a.e.a[1];
        c + Bits::<4>(1)
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("e").field("a").index(1)),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_index_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Bits<4>) -> Bits<4> {
        let c = [Bits::<4>(0), b];
        c[a]
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[1];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_index_nested_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Foo) -> Bits<4> {
        let c = [Bits::<4>(0), b.e.a[a]];
        c[a]
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[1];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("e").field("a").index(0)),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_index_double_nested_dynamic() {
    #[kernel]
    fn func(a: Bits<2>, b: Bits<2>, c: Foo) -> Bits<4> {
        c.e.c[a].a[b]
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[2];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(
            input_pin,
            Path::default()
                .field("e")
                .field("c")
                .index(1)
                .field("a")
                .index(0),
        ),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_repeat() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let b = [a; 4];
        b[3]
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_select() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>, c: bool) -> Bits<4> {
        if c {
            a
        } else {
            b
        }
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())));
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(schematic.inputs[2], Path::default()),
    )
    .unwrap();
    assert!(!trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())));
}

#[test]
fn test_downstream_splice() {
    #[kernel]
    fn func(a: Bits<4>) -> Foo {
        let mut c = Foo::default();
        c.e.a[1] = a;
        c
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    eprintln!("{:?}", trace);
    write_dot(
        &schematic,
        Some(&trace),
        std::fs::File::create("downstream_splice.dot").unwrap(),
    )
    .unwrap();
    assert!(trace.sinks.contains(&pin_path(
        schematic.output,
        Path::default().field("e").field("a").index(1)
    )));
}

#[test]
fn test_downstream_splice_orig_port() {
    #[kernel]
    fn func(mut a: Foo) -> Nested {
        a.e.a[0] = Bits::<4>(1);
        a.e
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("e").field("a").index(1)),
    )
    .unwrap();
    assert!(trace.sinks.contains(&pin_path(
        schematic.output,
        Path::default().field("a").index(1)
    )))
}

#[test]
fn test_downstream_struct() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Foo::default();
        let d = Foo { a, ..c };
        d.a
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_struct_rest() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Foo::default();
        let d = Foo { a, ..c };
        let e = Foo { b: d.a, ..d };
        e.a
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_rejects_illegal_query() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        let c = Foo::default();
        let d = Foo { a, ..c };
        let e = Foo { b: d.a, ..d };
        e.a
    }
    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    assert!(follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default().field("b")),
    )
    .is_err());
}

#[test]
fn test_downstream_tuple() {
    #[kernel]
    fn func(a: Bits<4>, b: Bits<4>) -> Bits<4> {
        let c = (a, b);
        c.1
    }

    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[1];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}

#[test]
fn test_downstream_unary() {
    #[kernel]
    fn func(a: Bits<4>) -> Bits<4> {
        !a
    }
    let schematic = get_schematic::<func>();
    let input_pin = schematic.inputs[0];
    let trace = follow_pin_downstream(
        &schematic.clone().into(),
        pin_path(input_pin, Path::default()),
    )
    .unwrap();
    assert!(trace
        .sinks
        .contains(&pin_path(schematic.output, Path::default())))
}
