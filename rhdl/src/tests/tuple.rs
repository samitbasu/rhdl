use crate::tests::{red, tuple_pair_b8_red, tuple_u8};
use rhdl_bits::alias::*;
use rhdl_core::{
    test_kernel_vm_and_verilog,
    types::{domain::Red, signal::signal},
    Domain, Signal,
};
use rhdl_macro::{kernel, Digital};

#[test]
fn test_tuple_destructure_in_args() {
    #[kernel]
    fn add((b, c): (Signal<u8, Red>, Signal<u8, Red>)) -> Signal<u8, Red> {
        b + c
    }

    let test_vec = (0..4)
        .flat_map(|a| (0..4).map(move |b| ((red(a), red(b)),)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter()).unwrap();
}

#[test]
fn test_tuple_struct_nested_init() {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: u8,
        b: u8,
    }

    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Bar {
        a: u8,
        b: Foo,
    }

    #[kernel]
    fn add<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        let b = Bar {
            a: 1,
            b: Foo { a: 2, b: 3 },
        };
        let Bar {
            a,
            b: Foo { a: c, b: d },
        } = b;
        signal(a + c + d)
    }

    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_u8()).unwrap()
}

#[test]
fn test_tuple_construct() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> (Signal<b8, Red>, Signal<b8, Red>) {
        (a, b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_tuple_indexing() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = (a, b);
        c.0 + c.1
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_tuple_construct_and_deconstruct() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = (a, b);
        let (d, e) = c;
        d + e
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_nested_tuple_indexing() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let c = (a, (b, a));
        c.1 .0 + c.1 .1
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_nested_tuple_init() {
    #[kernel]
    fn add<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        let b = (1, (2, 3), 4);
        let (c, (d, e), f) = b;
        signal(c + d + e + f) + a
    }

    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_u8()).unwrap();
}

#[test]
fn test_nested_tuple_array_init() {
    #[kernel]
    fn add<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        let b = [(1, (2, 3), 4); 3];
        let (c, (d, e), f) = b[1];
        let [g, (h0, (h1a, h1b), h2), i] = b;
        signal(c + d + e + f + g.0 + h0 + h1a + h1b + h2 + i.1 .0) + a
    }

    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_u8()).unwrap();
}
