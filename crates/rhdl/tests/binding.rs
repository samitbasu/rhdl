#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use expect_test::expect_file;
use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_nested_enum_match_in_if_let_fails() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    pub struct Bar(b8, b8);

    #[derive(PartialEq, Digital, Default)]
    pub enum Foo {
        Red(Bar),
        Blue(b8),
        #[default]
        White,
    }

    #[kernel]
    fn add(state: Signal<Option<Foo>, Red>) -> Signal<b8, Red> {
        if let Some(Foo::Red(Bar(x, y))) = state.val() {
            signal((x + y).resize())
        } else {
            signal(bits(0))
        }
    }

    let inputs = [
        Some(Foo::Red(Bar(bits(3), bits(2)))),
        Some(Foo::Red(Bar(bits(3), bits(4)))),
        Some(Foo::Red(Bar(bits(3), bits(6)))),
        Some(Foo::Red(Bar(bits(3), bits(8)))),
        None,
    ];

    let expect_err = expect_file!["nested_enum_in_if_let.expect"];
    let res = compile_design::<add>(CompilationMode::Asynchronous);
    let err = res.err().unwrap();
    let report: miette::Report = err.into();
    expect_err.assert_eq(&report.to_string());
    Ok(())
}

#[test]
fn test_nested_rebind_in_if_let() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    pub struct Bar(b8, b8);

    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: Bar,
    }

    #[kernel]
    fn add(state: Signal<Option<Foo>, Red>) -> Signal<b8, Red> {
        if let Some(Foo { a, b: Bar(_x, y) }) = state.val() {
            signal((a + y).resize())
        } else {
            signal(bits(0))
        }
    }

    let inputs = [
        Some(Foo {
            a: bits(1),
            b: Bar(bits(3), bits(2)),
        }),
        Some(Foo {
            a: bits(3),
            b: Bar(bits(3), bits(4)),
        }),
        Some(Foo {
            a: bits(5),
            b: Bar(bits(3), bits(6)),
        }),
        Some(Foo {
            a: bits(7),
            b: Bar(bits(3), bits(8)),
        }),
        None,
    ];

    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(red).map(|x| (x,)))?;

    Ok(())
}

#[test]
fn test_nested_rebind_inlet() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    pub struct Bar(b8, b8);

    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: Bar,
    }

    #[kernel]
    fn add(state: Signal<Foo, Red>) -> Signal<b8, Red> {
        let Foo { a, b: Bar(_x, y) } = state.val();
        signal((a + y).resize())
    }

    let inputs = [
        Foo {
            a: bits(1),
            b: Bar(bits(3), bits(2)),
        },
        Foo {
            a: bits(3),
            b: Bar(bits(3), bits(4)),
        },
        Foo {
            a: bits(5),
            b: Bar(bits(3), bits(6)),
        },
        Foo {
            a: bits(7),
            b: Bar(bits(3), bits(8)),
        },
    ];

    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
}

#[test]
fn test_rebind_in_let() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    fn add(state: Signal<Foo, Red>) -> Signal<b8, Red> {
        let Foo { a, b } = state.val();
        signal((a + b).resize())
    }

    let inputs = [
        Foo {
            a: bits(1),
            b: bits(2),
        },
        Foo {
            a: bits(3),
            b: bits(4),
        },
        Foo {
            a: bits(5),
            b: bits(6),
        },
        Foo {
            a: bits(7),
            b: bits(8),
        },
    ];

    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
}

#[test]
fn test_rebind_compile() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    pub enum SimpleEnum {
        #[default]
        Init,
        Run(b8),
        Point {
            x: b4,
            y: b8,
        },
        Boom,
    }

    const B6: b6 = bits(6);

    #[kernel]
    fn add(state: Signal<SimpleEnum, Red>) -> Signal<b8, Red> {
        let x = state;
        signal(match x.val() {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
        })
    }

    let inputs = [
        SimpleEnum::Init,
        SimpleEnum::Run(bits(1)),
        SimpleEnum::Run(bits(5)),
        SimpleEnum::Point {
            x: bits(7),
            y: bits(11),
        },
        SimpleEnum::Point {
            x: bits(7),
            y: bits(13),
        },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
}

#[test]
fn test_importing() {
    use rhdl::bits::alias::*;
    #[derive(PartialEq, Default, Digital)]
    pub enum Rad {
        A,
        B(b4),
        C {
            x: b4,
            y: b6,
        },
        #[default]
        D,
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b4, C>) -> Signal<(Rad, Rad, Rad, b8), C> {
        let k = Rad::A;
        let l = Rad::B(b4(1));
        let c = Rad::C { x: b4(1), y: b6(2) };
        let d = MY_SPECIAL_NUMBER;
        signal((k, l, c, (d + a.val().resize())))
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_assignment() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let mut c = a;
        c = b;
        c
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ssa() {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let mut q = a;
        q = (q + a).resize();
        q = a;
        signal(q)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
#[allow(clippy::let_and_return)]
fn test_rebinding() {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b16, Red> {
        let _q = a;
        let _q = b12(6);
        let _q = b16(7);
        signal(_q)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}
