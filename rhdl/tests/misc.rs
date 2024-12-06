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
#[allow(clippy::needless_late_init)]
#[allow(clippy::no_effect)]
fn test_compile() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    #[derive(Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(Default, Digital)]
    pub enum NooState {
        #[default]
        Init,
        Run(u8, u8, u8),
        Walk {
            foo: u8,
        },
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
    compile_design::<do_stuff<Red>>(CompilationMode::Asynchronous)?;
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, inputs.into_iter())?;
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
