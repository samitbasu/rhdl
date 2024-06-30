use crate::tests::{red, tuple_exhaustive_red, tuple_pair_b8_red};
use rhdl_bits::{alias::*, bits};
use rhdl_core::{
    test_kernel_vm_and_verilog,
    types::{
        domain::{self, Red},
        signal::signal,
    },
    Domain, Signal,
};
use rhdl_macro::{kernel, Digital};

#[test]
fn test_rebind_compile() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    pub enum SimpleEnum {
        Init,
        Run(u8),
        Point { x: b4, y: u8 },
        Boom,
    }

    const B6: b6 = bits(6);

    #[kernel]
    fn add(state: Signal<SimpleEnum, Red>) -> Signal<u8, Red> {
        let x = state;
        signal(match x.val() {
            SimpleEnum::Init => 1,
            SimpleEnum::Run(x) => x,
            SimpleEnum::Point { x, y } => y,
            SimpleEnum::Boom => 7,
        })
    }

    let inputs = [
        SimpleEnum::Init,
        SimpleEnum::Run(1),
        SimpleEnum::Run(5),
        SimpleEnum::Point { x: bits(7), y: 11 },
        SimpleEnum::Point { x: bits(7), y: 13 },
        SimpleEnum::Boom,
    ];
    test_kernel_vm_and_verilog::<add, _, _, _>(add, inputs.into_iter().map(red).map(|x| (x,)))?;
    Ok(())
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
        let mut q = a;
        q = q + a;
        q = a;
        q
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
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
