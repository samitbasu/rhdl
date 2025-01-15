#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use itertools::iproduct;
use rhdl::prelude::*;

#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl_core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_adt_use() -> miette::Result<()> {
    #[derive(PartialEq, Digital)]
    pub enum Foo {
        Red(b8, bool),
        Green(b8, bool),
    }

    impl Default for Foo {
        fn default() -> Self {
            Foo::Red(bits(0), false)
        }
    }

    #[kernel]
    fn get_color(a: Signal<Foo, Red>, c: Signal<bool, Red>) -> Signal<bool, Red> {
        signal(
            c.val()
                && match a.val() {
                    Foo::Red(_x, z) => z,
                    Foo::Green(_x, _z) => true,
                },
        )
    }

    test_kernel_vm_and_verilog::<get_color, _, _, _>(
        get_color,
        [
            (Foo::Red(bits(3), true), false),
            (Foo::Green(bits(4), true), true),
        ]
        .iter()
        .cloned()
        .map(|(a, b)| (signal(a), signal(b))),
    )?;
    Ok(())
}

#[test]
fn test_struct_expr_adt() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum Foo {
        A,
        B(b8),
        C {
            a: b8,
            b: b16,
        },
        #[default]
        D,
    }

    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        signal(if a < 10 {
            Foo::A
        } else if a < 20 {
            Foo::B(a)
        } else {
            Foo::C { a, b: bits(0) }
        })
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_b8())?;
    Ok(())
}

#[test]
fn test_unit_enums_are_repr() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Default)]
    pub enum Rad {
        #[default]
        A,
        B(b4),
        C {
            x: b4,
            y: b6,
        },
        D,
    }

    let x = Rad::D;
    let x = x.bin();
    eprintln!("{}", bitx_string(&x));
    Ok(())
}

#[test]
fn test_adt_inference_subset() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Digital, Default)]
    pub enum Rad {
        #[default]
        A,
        B(b4),
        C {
            x: b4,
            y: b6,
        },
        D,
    }

    #[derive(PartialEq, Digital, Default)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Digital, Default)]
    pub enum NooState {
        #[default]
        Init,
        Run(b4, b5),
        Walk {
            foo: b5,
        },
        Boom,
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff<C: Domain>(
        a: Signal<Foo, C>,
        _s: Signal<NooState, C>,
    ) -> Signal<(NooState, b7), C> {
        let _z = (a.val().b, a.val().a + MY_SPECIAL_NUMBER);
        let foo = b12(6);
        let _foo2 = foo + foo;
        let _c = a;
        let q = s4(2);
        let _q = Foo {
            a: b8(1),
            b: q,
            c: Rad::A,
        };
        signal((NooState::Init, b7(3)))
    }

    let foos = [
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::A,
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::B(b4(1)),
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::C { x: b4(1), y: b6(2) },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(b4(1), b5(2)),
        NooState::Walk { foo: b5(3) },
    ];
    let inputs =
        iproduct!(foos.into_iter().map(red), noos.into_iter().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_adt_inference() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Digital, Default)]
    pub enum Rad {
        #[default]
        A,
        B(b4),
        C {
            x: b4,
            y: b6,
        },
        D,
    }

    #[derive(PartialEq, Digital, Default)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Digital, Default)]
    pub enum NooState {
        #[default]
        Init,
        Run(b4, b5),
        Walk {
            foo: b5,
        },
        Boom,
    }

    #[kernel]
    fn fifo<C: Domain>(b: Signal<b8, C>, _a: Signal<b4, C>) -> Signal<b8, C> {
        b
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<Foo, C>, s: Signal<NooState, C>) -> Signal<(NooState, b7), C> {
        let a = a.val();
        let _z = (a.b, a.a + MY_SPECIAL_NUMBER);
        let foo = b12(6);
        let _foo2 = foo + foo;
        let _c = a;
        let q = s4(2);
        let _q = Foo {
            a: b8(1),
            b: q,
            c: Rad::A,
        };
        let c = Rad::A;
        let _d = c;
        let _z = fifo::<C>(signal(b8(3)), signal(b4(5)));
        let mut q = b4(1);
        let _l = q.any();
        q |= bits(1 << 3);
        let _p = (q & bits(1 << 2)).any();
        let _p = q.as_signed();
        if a.a > b8(12) {
            return signal((NooState::Boom, b7(3)));
        }
        let _e = Rad::B(q);
        let x1 = b4(4);
        let y1 = b6(6);
        let mut ar = [b4(1), b4(1), b4(3)];
        ar[1] = b4(2);
        let _z: [Bits<U4>; 3] = ar;
        let _q = ar[1];
        let f: [b4; 5] = [b4(1); 5];
        let _h = f[2];
        let _k = NooState::Init;
        let _f = Rad::C { y: y1, x: x1 };
        let d = match s.val() {
            NooState::Init => NooState::Run(b4(1), b5(2)),
            NooState::Run(_x, y) => NooState::Walk {
                foo: (y + 3).resize(),
            },
            NooState::Walk { foo: x } => {
                let _q = b5(1) + x;
                NooState::Boom
            }
            NooState::Boom => NooState::Init,
        };
        let k = 42;
        signal((d, b7(k)))
    }

    let foos = [
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::A,
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::B(b4(1)),
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad::C { x: b4(1), y: b6(2) },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(b4(1), b5(2)),
        NooState::Walk { foo: b5(3) },
    ];
    let inputs =
        iproduct!(foos.into_iter().map(red), noos.into_iter().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_adt_shadow() {
    #[derive(PartialEq, Digital, Default)]
    pub enum NooState {
        #[default]
        Init,
        Run(b8, b8, b8),
        Walk {
            foo: b8,
        },
        Boom,
    }

    #[kernel]
    fn do_stuff<C: Domain>(mut s: Signal<NooState, C>) -> Signal<(b8, NooState), C> {
        let _y = b12(72);
        let _foo = b14(32);
        let mut a: b8 = bits(0);
        let d = match s.val() {
            NooState::Init => {
                a = bits(1);
                NooState::Run(bits(1), bits(2), bits(3))
            }
            NooState::Walk { foo: x } => {
                a = x;
                NooState::Boom
            }
            NooState::Run(x, _, y) => {
                a = (x + y).resize();
                NooState::Walk { foo: bits(7) }
            }
            NooState::Boom => {
                a = (a + 3).resize();
                NooState::Init
            }
        };
        signal((a, d))
    }
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits(1), bits(2), bits(3)),
        NooState::Walk { foo: bits(4) },
    ];
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(
        do_stuff::<Red>,
        noos.into_iter().map(|x| (signal(x),)),
    )
    .unwrap();
}

#[test]
fn test_enum_match() -> miette::Result<()> {
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

    #[kernel]
    fn add<C: Domain>(state: Signal<SimpleEnum, C>) -> Signal<b8, C> {
        let x = state.val();
        signal(match x {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
        })
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(bits(1)),
        SimpleEnum::Run(bits(2)),
        SimpleEnum::Run(bits(3)),
        SimpleEnum::Point {
            x: bits(1),
            y: bits(2),
        },
        SimpleEnum::Point {
            x: bits(1),
            y: bits(9),
        },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(
        add,
        samples.into_iter().map(red).map(|x| (x,)),
    )?;
    Ok(())
}

#[ignore]
#[test]
fn test_enum_unmatched_variant_not_usable() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    enum SimpleEnum {
        #[default]
        Init,
        Run(b8),
        Boom,
        Unmatched,
    }

    #[kernel]
    fn add(_a: Signal<SimpleEnum, Red>) -> Signal<SimpleEnum, Red> {
        signal(SimpleEnum::Unmatched)
    }

    let samples = vec![SimpleEnum::Unmatched];
    let Err(err) =
        test_kernel_vm_and_verilog::<add, _, _, _>(add, samples.into_iter().map(red).map(|x| (x,)))
    else {
        panic!("Expected error")
    };
    match err {
        RHDLError::RHDLSyntaxError(_) => Ok(()),
        _ => panic!("Unexpected err {err:?}"),
    }
}

#[test]
fn test_enum_match_signed_discriminant() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Default)]
    #[rhdl(discriminant_width = 4)]
    #[repr(i8)]
    pub enum SimpleEnum {
        Init = 1,
        Run(b8) = 2,
        Point {
            x: b4,
            y: b8,
        } = 3,
        Boom = -2,
        #[default]
        Unmatched,
    }

    #[kernel]
    fn add(state: Signal<SimpleEnum, Red>) -> Signal<b8, Red> {
        let x = state.val();
        signal(match x {
            SimpleEnum::Init => bits(1),
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x: _, y } => y,
            SimpleEnum::Boom => bits(7),
            _ => bits(8),
        })
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(bits(1)),
        SimpleEnum::Run(bits(2)),
        SimpleEnum::Run(bits(3)),
        SimpleEnum::Point {
            x: bits(1),
            y: bits(2),
        },
        SimpleEnum::Point {
            x: bits(1),
            y: bits(9),
        },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, samples.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
}

#[test]
#[allow(clippy::comparison_chain)]
fn test_enum_basic() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    enum Foo {
        #[default]
        A,
        B(b8),
        C {
            red: b8,
            green: b8,
            blue: b8,
        },
        D,
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        let b = b.val();
        signal(if a == b {
            Foo::A
        } else if a > b {
            Foo::B(a + b)
        } else {
            Foo::C {
                red: a,
                green: b,
                blue: a,
            }
        })
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_match_enum() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    enum Foo {
        #[default]
        A,
        B(b8),
        C {
            red: b8,
            green: b8,
            blue: b8,
        },
        D,
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo::C {
            red: a,
            green: b,
            blue: a,
        };
        signal(match c {
            Foo::A => b8(1),
            Foo::B(x) => x,
            Foo::C { red, green, blue } => (red + green + blue).resize(),
            _ => b8(4),
        })
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}
