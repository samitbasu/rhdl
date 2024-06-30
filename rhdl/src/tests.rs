#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]
use itertools::iproduct;
use rand::Rng;
use rhdl_bits::{alias::*, bits, signed, Bits, SignedBits};
use rhdl_core::{
    compile_design,
    compiler::mir::error::Syntax,
    error::RHDLError,
    test_kernel_vm_and_verilog,
    types::{
        domain::{self, Red},
        signal::signal,
    },
    Digital, Domain, Kind, Signal,
};
use rhdl_macro::{kernel, Digital};

mod adt;
mod ast;
mod clock;
mod derive;
mod indexing;
mod inference;
mod phi;
mod strukt;
mod tuple;
mod type_rolling;
mod vcd;
mod vm;

#[test]
fn test_unknown_clock_domain() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b12, C>) -> Signal<b12, C> {
        let k = a;
        let m = bits::<14>(7);
        let c = k + 3;
        let d = if c > k { c } else { k };
        let e = (c, m);
        let (f, g) = e;
        let h = g + 1;
        let k: b4 = bits::<4>(7);
        let q = (bits::<2>(1), (bits::<5>(0), signed::<8>(5)), bits::<12>(6));
        let b = q.1 .1;
        let (q0, (q1, q1b), q2) = q; // Tuple destructuring
        let z = q1b + 4;
        let h = [d, c, f];
        let [i, j, k] = h;
        let o = j;
        let l = {
            let a = b12(3);
            let b = bits(4);
            a + b
        };
        l + k
    }
    assert!(compile_design::<do_stuff<Red>>().is_err());
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_rebinding() {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b16, Red> {
        let q = a;
        let q = bits::<12>(6);
        let q = bits::<16>(7);
        signal(q)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_missing_register() {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let mut c = bits::<8>(0);
        match a.val() {
            Bits::<1>(0) => c = bits::<8>(2),
            Bits::<1>(1) => c = bits::<8>(3),
            _ => {}
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ssa() {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let mut q = a;
        q = q + a;
        q = a;
        q
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_importing() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum Red {
        A,
        B(b4),
        C {
            x: b4,
            y: b6,
        },
        #[rhdl(unmatched)]
        D,
    }

    const MY_SPECIAL_NUMBER: b8 = bits(42);

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b4, C>) -> Signal<(Red, Red, Red, b8), C> {
        let k = Red::A;
        let l = Red::B(bits::<4>(1));
        let c = Red::C {
            x: bits::<4>(1),
            y: bits::<6>(2),
        };
        let d = MY_SPECIAL_NUMBER;
        signal((k, l, c, d))
    }
    test_kernel_vm_and_verilog::<do_stuff<domain::Red>, _, _, _>(do_stuff, tuple_exhaustive_red())
        .unwrap();
}

#[test]
fn test_compile() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(u8, u8, u8),
        Walk { foo: u8 },
        Boom,
    }

    const CONST_PATH: b4 = bits(4);
    #[kernel]
    fn do_stuff<C: Domain>(mut a: Signal<Foo, C>, mut s: Signal<NooState, C>) -> Signal<Foo, C> {
        let k = {
            bits::<12>(4) + 6;
            bits::<12>(6)
        };
        let mut a: Foo = a.val();
        let mut s: NooState = s.val();
        let q = if a.a > 0 {
            bits::<12>(3)
        } else {
            bits::<12>(0)
        };
        let y = bits::<12>(72);
        let t2 = (y, y);
        let q: u8 = 4;
        let z = a.c;
        let w = (a, a);
        a.c[1] = q + 3;
        a.c = [0; 3];
        a.c = [1, 2, 3];
        let q = (1, (0, 5), 6);
        let (q0, (q1, q1b), q2): (u8, (u8, u8), u16) = q; // Tuple destructuring
        a.a = 2 + 3 + q1 + q0 + q1b + if q2 != 0 { 1 } else { 0 };
        let z;
        if 1 > 3 {
            z = bits::<4>(2);
        } else {
            z = bits::<4>(5);
        }
        a.b = {
            7 + 9;
            5 + !8
        };
        a.a = if 1 > 3 {
            7
        } else {
            {
                a.b = 1;
                a.b = 4;
            }
            9
        };
        let g = 1 > 2;
        let h = 3 != 4;
        let mut i = g && h;
        if z == bits::<4>(3) {
            i = false;
        }
        match a {
            Foo {
                a: 1,
                b: 2,
                c: [1, 2, 3],
            } => {}
            Foo {
                a: 3,
                b: 4,
                c: [1, 2, 3],
            } => {}
            _ => {}
        }
        let c = bits::<4>(match z {
            CONST_PATH => 1,
            Bits::<4>(1) => 2,
            Bits::<4>(2) => 3,
            Bits::<4>(3) => {
                a.a = 4;
                4
            }
            _ => 6,
        });
        let d = match s {
            NooState::Init => {
                a.a = 1;
                NooState::Run(1, 2, 3)
            }
            NooState::Walk { foo: x } => {
                a.a = x;
                NooState::Boom
            }
            NooState::Run(x, t, y) => {
                a.a = x + y;
                NooState::Walk { foo: 7 }
            }
            NooState::Boom => {
                a.a += 3;
                NooState::Init
            }
        };
        signal(a)
    }
    let foos = [
        Foo {
            a: 1,
            b: 2,
            c: [1, 2, 3],
        },
        Foo {
            a: 4,
            b: 5,
            c: [4, 5, 6],
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    let inputs =
        iproduct!(foos.into_iter().map(red), noos.into_iter().map(red)).collect::<Vec<_>>();
    compile_design::<do_stuff<domain::Red>>()?;
    test_kernel_vm_and_verilog::<do_stuff<domain::Red>, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_custom_suffix() {
    #[kernel]
    fn do_stuff(mut a: Signal<b4, Red>) {
        let b = a + 1;
        let c = bits::<4>(3);
        a = b;
    }
}

#[test]
fn test_error_about_for_loop() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) {
        let mut a = a.val();
        let c = 5;
        for ndx in 0..c {
            a += bits::<4>(ndx);
        }
    }
    let Err(RHDLError::RHDLSyntaxError(err)) = compile_design::<do_stuff>() else {
        panic!("Expected syntax error");
    };
    assert!(matches!(err.cause, Syntax::ForLoopNonIntegerEndValue));
    Ok(())
}

#[test]
fn test_match_scrutinee_bits() {
    let z = bits::<4>(0b1010);
    match z {
        rhdl_bits::Bits::<4>(0b0000) => {}
        rhdl_bits::Bits::<4>(0b0001) => {}
        _ => {}
    }
}
#[test]
fn test_macro_output() {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(u8, u8, u8),
        Walk { foo: u8 },
        Boom,
    }

    #[kernel]
    fn do_stuff<C: Domain>(
        mut a: Signal<Foo, C>,
        mut s: Signal<NooState, C>,
    ) -> Signal<NooState, C> {
        let z = bits::<6>(3);
        let c = match z {
            Bits::<6>(4) => bits::<4>(7),
            Bits::<6>(3) => bits::<4>(3),
            _ => bits::<4>(8),
        };
        let z = 1;
        let h = NooState::Run(1, z, 3);
        signal(h)
    }

    let foos = [
        Foo {
            a: 1,
            b: 2,
            c: [1, 2, 3],
        },
        Foo {
            a: 4,
            b: 5,
            c: [4, 5, 6],
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(1, 2, 3),
        NooState::Walk { foo: 4 },
    ];
    let inputs =
        iproduct!(foos.into_iter().map(red), noos.into_iter().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<domain::Red>, _, _, _>(do_stuff, inputs.into_iter())
        .unwrap();
}

#[test]
fn test_generics() {
    #[kernel]
    fn do_stuff<T: Digital, C: Domain>(a: Signal<T, C>, b: Signal<T, C>) -> Signal<bool, C> {
        signal(a == b)
    }

    let a = [
        signed::<4>(1),
        signed::<4>(2),
        signed::<4>(3),
        signed::<4>(-1),
        signed::<4>(-3),
    ];
    let inputs =
        iproduct!(a.iter().cloned().map(red), a.iter().cloned().map(red)).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, domain::Red>, _, _, _>(do_stuff, inputs.into_iter())
        .unwrap();
}

#[test]
fn test_nested_generics() {
    #[derive(PartialEq, Copy, Clone, Digital)]
    struct Foo<T: Digital> {
        a: T,
        b: T,
    }

    #[kernel]
    fn do_stuff<T: Digital, S: Digital, C: Domain>(
        x: Signal<Foo<T>, C>,
        y: Signal<Foo<S>, C>,
    ) -> Signal<bool, C> {
        let x = x.val();
        let y = y.val();
        let c = x.a;
        let d = (x.a, y.b);
        let e = Foo::<T> { a: c, b: c };
        signal(e == x)
    }

    let a = [
        signed::<4>(1),
        signed::<4>(2),
        signed::<4>(3),
        signed::<4>(-1),
        signed::<4>(-3),
    ];
    let b: Vec<b3> = exhaustive();
    let inputs = iproduct!(
        a.into_iter().map(|x| Foo { a: x, b: x }).map(red),
        b.into_iter().map(|x| Foo { a: x, b: x }).map(red)
    )
    .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<do_stuff<s4, b3, domain::Red>, _, _, _>(
        do_stuff::<s4, b3, domain::Red>,
        inputs.into_iter(),
    )
    .unwrap();
}

#[test]
fn test_fn_name_stuff() {
    // There are 2 namespaces (type and value).
    // A function is a value.  So this is legal:

    struct add_stuff {}

    impl add_stuff {
        fn args() -> Vec<Kind> {
            vec![Kind::make_bits(8), Kind::make_bits(8)]
        }
        fn ret() -> Kind {
            Kind::make_bits(8)
        }
    }

    fn add_stuff(a: u8, b: u8) -> u8 {
        a + b
    }

    assert_eq!(add_stuff(3_u8, 4_u8), 7_u8);
    assert_eq!(
        add_stuff::args(),
        vec![Kind::make_bits(8), Kind::make_bits(8)]
    );
    assert_eq!(add_stuff::ret(), Kind::make_bits(8));
}

#[test]
fn test_fn_name_generic_stuff() {
    struct add_stuff<T: Digital> {
        _phantom: std::marker::PhantomData<T>,
    }

    impl<T: Digital> add_stuff<T> {
        fn args() -> Vec<Kind> {
            vec![T::static_kind(), T::static_kind()]
        }
        fn ret() -> Kind {
            T::static_kind()
        }
    }

    fn add_stuff<T: Digital>(a: T, b: T) -> T {
        b
    }

    assert_eq!(add_stuff::<b4>(3.into(), 4.into()), bits(4));
    assert_eq!(
        add_stuff::<b4>::args(),
        vec![Kind::make_bits(4), Kind::make_bits(4)]
    );
    assert_eq!(add_stuff::<b4>::ret(), Kind::make_bits(4));
}

#[test]
fn test_for_loop() -> miette::Result<()> {
    #[kernel]
    fn looper(a: Signal<[bool; 8], Red>) -> Signal<bool, Red> {
        let a = a.val();
        let mut ret: bool = false;
        for i in 0..8 {
            ret ^= a[i];
        }
        signal(ret)
    }
    let inputs = (0..256).map(|x| {
        let mut a = [false; 8];
        for i in 0..8 {
            a[i] = (x >> i) & 1 == 1;
        }
        (signal(a),)
    });
    test_kernel_vm_and_verilog::<looper, _, _, _>(looper, inputs)?;
    Ok(())
}

#[test]
fn test_const_literal_match() {
    #[kernel]
    fn add<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        signal(match a.val() {
            1 => 1,
            2 => 2,
            _ => 3,
        })
    }
    test_kernel_vm_and_verilog::<add<domain::Red>, _, _, _>(add::<Red>, tuple_u8()).unwrap();
}

#[test]
fn test_const_literal_captured_match() {
    const ZERO: b4 = bits(0);
    const ONE: b4 = bits(1);
    const TWO: b4 = bits(2);

    #[kernel]
    fn do_stuff(a: Signal<b4, Red>) -> Signal<b4, Red> {
        signal(match a.val() {
            ONE => TWO,
            TWO => ONE,
            _ => ZERO,
        })
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_struct_literal_match() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub struct Foo {
        a: u8,
        b: u8,
    }

    #[kernel]
    fn add(a: Signal<Foo, Red>) -> Signal<u8, Red> {
        signal(match a.val() {
            Foo { a: 1, b: 2 } => 1,
            Foo { a: 3, b: 4 } => 2,
            _ => 3,
        })
    }

    let test_vec = (0..4)
        .flat_map(|a| (0..4).map(move |b| (red(Foo { a, b }),)))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<add, _, _, _>(add, test_vec.into_iter())?;
    Ok(())
}

fn exhaustive<const N: usize>() -> Vec<Bits<N>> {
    (0..(1 << N)).map(bits).collect()
}

fn tuple_exhaustive_red<const N: usize>() -> impl Iterator<Item = (Signal<Bits<N>, Red>,)> + Clone {
    exhaustive::<N>().into_iter().map(|x| (signal(x),))
}

fn tuple_u8<C: Domain>() -> impl Iterator<Item = (Signal<u8, C>,)> + Clone {
    (0_u8..255_u8).map(|x| (signal(x),))
}

fn tuple_pair_b8_red() -> impl Iterator<Item = (Signal<b8, Red>, Signal<b8, Red>)> + Clone {
    exhaustive::<8>()
        .into_iter()
        .flat_map(|x| exhaustive::<8>().into_iter().map(move |y| (red(x), red(y))))
}

fn tuple_pair_s8_red() -> impl Iterator<Item = (Signal<s8, Red>, Signal<s8, Red>)> + Clone {
    exhaustive::<8>().into_iter().flat_map(|x| {
        exhaustive::<8>()
            .into_iter()
            .map(move |y| (red(x.as_signed()), red(y.as_signed())))
    })
}

fn red<T: Digital>(x: T) -> Signal<T, Red> {
    signal(x)
}

#[test]
fn test_plain_literals() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_plain_literals_signed_context() {
    #[kernel]
    fn foo(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        a + 2 + b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8_red()).unwrap();
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
fn test_assignment_of_if_expression() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let mut c = a;
        c = if a > b { a + 1 } else { b + 2 };
        c
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_signed_match() {
    #[kernel]
    fn foo(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        match a.val() {
            SignedBits::<8>(1) => b,
            SignedBits::<8>(2) => a,
            _ => signal(s8(3)),
        }
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_s8_red()).unwrap();
}

#[test]
fn test_early_return() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        return a;
        b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

#[test]
#[allow(clippy::no_effect)]
fn test_early_return_in_branch() {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        if a > b {
            let d = 5;
            d + 3;
            return a;
        }
        b
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red()).unwrap();
}

fn rand_bits<const N: usize>() -> Bits<N> {
    let mut rng = rand::thread_rng();
    let val: u128 = rng.gen();
    Bits::<N>(val & Bits::<N>::MASK.0)
}
