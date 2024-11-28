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
fn test_adt_use() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Foo {
        Red(u8, bool),
        Green(u8, bool),
    }

    impl Default for Foo {
        fn default() -> Self {
            Foo::Red(0, false)
        }
    }

    #[kernel]
    fn get_color(a: Signal<Foo, Red>, c: Signal<bool, Red>) -> Signal<bool, Red> {
        signal(
            c.val()
                && match a.val() {
                    Foo::Red(x, z) => z,
                    Foo::Green(x, z) => true,
                },
        )
    }

    test_kernel_vm_and_verilog::<get_color, _, _, _>(
        get_color,
        [(Foo::Red(3, true), false), (Foo::Green(4, true), true)]
            .iter()
            .cloned()
            .map(|(a, b)| (signal(a), signal(b))),
    )
    .unwrap();
}

#[test]
fn test_struct_expr_adt() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Default, Digital)]
    pub enum Foo {
        A,
        B(u8),
        C {
            a: u8,
            b: u16,
        },
        #[default]
        D,
    }

    #[kernel]
    fn do_stuff(a: Signal<u8, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        signal(if a < 10 {
            Foo::A
        } else if a < 20 {
            Foo::B(a)
        } else {
            Foo::C { a, b: 0 }
        })
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_u8())?;
    Ok(())
}

#[test]
fn test_unit_enums_are_repr() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital, Default)]
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

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
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

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
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
    fn do_stuff<C: Domain>(a: Signal<Foo, C>, s: Signal<NooState, C>) -> Signal<(NooState, b7), C> {
        let z = (a.val().b, a.val().a + MY_SPECIAL_NUMBER);
        let foo = bits::<12>(6);
        let foo2 = foo + foo;
        let c = a;
        let q = signed::<4>(2);
        let q = Foo {
            a: bits::<8>(1),
            b: q,
            c: Rad::A,
        };
        signal((NooState::Init, bits::<7>(3)))
    }

    let foos = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::A,
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::B(bits::<4>(1)),
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::C {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits::<4>(1), bits::<5>(2)),
        NooState::Walk { foo: bits::<5>(3) },
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

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
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

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Copy, Clone, Digital, Default)]
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
    fn fifo<C: Domain>(b: Signal<b8, C>, a: Signal<b4, C>) -> Signal<b8, C> {
        b
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<Foo, C>, s: Signal<NooState, C>) -> Signal<(NooState, b7), C> {
        let a = a.val();
        let z = (a.b, a.a + MY_SPECIAL_NUMBER);
        let foo = bits::<12>(6);
        let foo2 = foo + foo;
        let c = a;
        let q = signed::<4>(2);
        let q = Foo {
            a: bits::<8>(1),
            b: q,
            c: Rad::A,
        };
        let c = Rad::A;
        let d = c;
        let z = fifo::<C>(signal(bits::<8>(3)), signal(bits::<4>(5)));
        let mut q = bits::<4>(1);
        let l = q.any();
        q |= bits(1 << 3);
        let p = (q & bits(1 << 2)).any();
        let p = q.as_signed();
        if a.a > bits::<8>(12) {
            return signal((NooState::Boom, bits::<7>(3)));
        }
        let e = Rad::B(q);
        let x1 = bits::<4>(4);
        let y1 = bits::<6>(6);
        let mut ar = [bits::<4>(1), bits::<4>(1), bits::<4>(3)];
        ar[1] = bits::<4>(2);
        let z: [Bits<4>; 3] = ar;
        let q = ar[1];
        let f: [b4; 5] = [bits::<4>(1); 5];
        let h = f[2];
        let k = NooState::Init;
        let f = Rad::C { y: y1, x: x1 };
        let d = match s.val() {
            NooState::Init => NooState::Run(bits::<4>(1), bits::<5>(2)),
            NooState::Run(x, y) => NooState::Walk { foo: y + 3 },
            NooState::Walk { foo: x } => {
                let q = bits::<5>(1) + x;
                NooState::Boom
            }
            NooState::Boom => NooState::Init,
        };
        let k = 42;
        signal((d, bits::<7>(k)))
    }

    let foos = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::A,
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::B(bits::<4>(1)),
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad::C {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits::<4>(1), bits::<5>(2)),
        NooState::Walk { foo: bits::<5>(3) },
    ];
    let inputs =
        iproduct!(foos.into_iter().map(red), noos.into_iter().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_adt_shadow() {
    #[derive(PartialEq, Copy, Clone, Digital, Default)]
    pub enum NooState {
        #[default]
        Init,
        Run(u8, u8, u8),
        Walk {
            foo: u8,
        },
        Boom,
    }

    #[kernel]
    fn do_stuff<C: Domain>(mut s: Signal<NooState, C>) -> Signal<(u8, NooState), C> {
        let y = bits::<12>(72);
        let foo = bits::<14>(32);
        let mut a: u8 = 0;
        let d = match s.val() {
            NooState::Init => {
                a = 1;
                NooState::Run(1, 2, 3)
            }
            NooState::Walk { foo: x } => {
                a = x;
                NooState::Boom
            }
            NooState::Run(x, _, y) => {
                a = x + y;
                NooState::Walk { foo: 7 }
            }
            NooState::Boom => {
                a = a + 3;
                NooState::Init
            }
        };
        signal((a, d))
    }
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(
        do_stuff::<Red>,
        noos.into_iter().map(|x| (signal(x),)),
    )
    .unwrap();
}

#[test]
fn test_enum_match() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
    pub enum SimpleEnum {
        #[default]
        Init,
        Run(u8),
        Point {
            x: b4,
            y: u8,
        },
        Boom,
    }

    #[kernel]
    fn add<C: Domain>(state: Signal<SimpleEnum, C>) -> Signal<u8, C> {
        let x = state.val();
        signal(match x {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        })
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(2),
        SimpleEnum::Run(3),
        SimpleEnum::Point { x: bits(1), y: 2 },
        SimpleEnum::Point { x: bits(1), y: 9 },
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
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
    enum SimpleEnum {
        #[default]
        Init,
        Run(u8),
        Boom,
        Unmatched,
    }

    #[kernel]
    fn add(a: Signal<SimpleEnum, Red>) -> Signal<SimpleEnum, Red> {
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
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
    #[rhdl(discriminant_width = 4)]
    #[repr(i8)]
    pub enum SimpleEnum {
        Init = 1,
        Run(u8) = 2,
        Point {
            x: b4,
            y: u8,
        } = 3,
        Boom = -2,
        #[default]
        Unmatched,
    }

    #[kernel]
    fn add(state: Signal<SimpleEnum, Red>) -> Signal<u8, Red> {
        let x = state.val();
        signal(match x {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
            _ => 8,
        })
    }

    let samples = vec![
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(2),
        SimpleEnum::Run(3),
        SimpleEnum::Point { x: bits(1), y: 2 },
        SimpleEnum::Point { x: bits(1), y: 9 },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, samples.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
}

#[test]
#[allow(clippy::comparison_chain)]
fn test_enum_basic() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
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
    #[derive(PartialEq, Copy, Clone, Debug, Digital, Default)]
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
            Foo::C { red, green, blue } => red + green + blue,
            _ => b8(4),
        })
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}
