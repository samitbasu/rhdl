#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rand::prelude::*;
use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_tuple_struct_indexing() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo(b8, b8);

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo(a, b);
        signal(c.0 + c.1)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_struct_field_indexing() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo {
        a: (b8, b8),
        b: b8,
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let mut c = Foo { a: (a, a), b };
        c.a.0 = c.b;
        signal(c.a.0 + c.a.1 + c.b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_array_indexing_simple() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<[b8; 2], Red> {
        let a = a.val();
        let b = b.val();
        let mut c = [a, b];
        c[1] = a;
        c[0] = b;
        signal(c)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_array_indexing() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<[b8; 2], Red> {
        let a = a.val();
        let b = b.val();
        let mut c = [a, b];
        c[1] = a;
        c[0] = b;
        signal([(c[0] + c[1]).resize(), c[1]])
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_array_indexing_2() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<[b8; 2], Red> {
        let a = a.val();
        let b = b.val();
        let c = [a, b];
        signal([c[0], c[1]])
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[cfg(test)]
fn rand_bits<N: BitWidth>() -> Bits<N> {
    Bits::<N>::default()
}

#[test]
fn test_3d_array_dynamic_indexing() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    pub struct VolumeBits {
        data: [[[b1; 8]; 8]; 8],
    }

    fn rand_volume_bits() -> VolumeBits {
        let mut ret = VolumeBits::default();
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    ret.data[i][j][k] = rand_bits();
                }
            }
        }
        ret
    }

    #[kernel]
    fn foo(
        a: Signal<VolumeBits, Red>,
        b: Signal<b3, Red>,
        c: Signal<b3, Red>,
        d: Signal<b3, Red>,
    ) -> Signal<b1, Red> {
        let a = a.val();
        let b = b.val();
        let c = c.val();
        let d = d.val();
        signal(a.data[b][c][d])
    }

    let test_cases = (0..100)
        .map(|_| {
            (
                red(rand_volume_bits()),
                red(rand_bits()),
                red(rand_bits()),
                red(rand_bits()),
            )
        })
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_cases.into_iter())?;
    Ok(())
}

#[test]
fn test_complex_array_dynamic_indexing() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo {
        a: bool,
        b: [b4; 4],
        c: bool,
    }

    fn rand_foo() -> Foo {
        // make a random Foo
        let mut rng = rand::thread_rng();
        Foo {
            a: rng.gen(),
            b: [rand_bits(), rand_bits(), rand_bits(), rand_bits()],
            c: rng.gen(),
        }
    }

    #[derive(PartialEq, Debug, Digital)]
    pub struct Bar {
        a: b9,
        b: [Foo; 8],
    }

    fn rand_bar() -> Bar {
        // make a random Bar
        let mut rng = rand::thread_rng();
        Bar {
            a: b9(rng.gen::<u16>() as u128 % 512),
            b: [
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
                rand_foo(),
            ],
        }
    }

    #[kernel]
    fn foo(bar: Signal<Bar, Red>, n1: Signal<b3, Red>, n2: Signal<b2, Red>) -> Signal<b4, Red> {
        let bar = bar.val();
        let n1 = n1.val();
        let n2 = n2.val();
        signal(bar.b[n1].b[n2])
    }

    let test_cases = (0..100)
        .map(|_| (red(rand_bar()), red(rand_bits()), red(rand_bits())))
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, test_cases.into_iter())?;

    #[kernel]
    fn bar(
        bar: Signal<Bar, Red>,
        n1: Signal<b3, Red>,
        b2: Signal<b2, Red>,
        b3: Signal<b4, Red>,
    ) -> Signal<Bar, Red> {
        let bar = bar.val();
        let mut ret = bar;
        let n1 = n1.val();
        let b2 = b2.val();
        let b3 = b3.val();
        ret.b[n1].b[b2] = b3;
        signal(ret)
    }

    let test_cases = (0..100)
        .map(|_| {
            (
                red(rand_bar()),
                red(rand_bits()),
                red(rand_bits()),
                red(rand_bits()),
            )
        })
        .collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<bar, _, _, _>(bar, test_cases.into_iter())?;
    Ok(())
}

#[test]
fn test_array_dynamic_indexing() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<[b8; 8], Red>, b: Signal<b3, Red>) -> Signal<b8, Red> {
        let a = a.val();
        signal(a[b])
    }

    let a = [
        bits(101),
        bits(102),
        bits(103),
        bits(104),
        bits(105),
        bits(106),
        bits(107),
        bits(108),
    ];
    let b = exhaustive();
    let inputs = b.into_iter().map(|b| (red(a), red(b))).collect::<Vec<_>>();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_array_dynamic_indexing_on_write() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<[b8; 8], Red>, b: Signal<b3, Red>) -> Signal<[b8; 8], Red> {
        let b = b.val();
        let mut c = a.val();
        c[b] = b8(42);
        c[0] = b8(12);
        signal(c)
    }
    let a = [
        bits(101),
        bits(102),
        bits(103),
        bits(104),
        bits(105),
        bits(106),
        bits(107),
        bits(108),
    ];
    let b = exhaustive();
    let inputs = b.into_iter().map(|b| (red(a), red(b))).collect::<Vec<_>>();
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_field_indexing_is_order_independent() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<Foo, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo { b, a };
        signal(c)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_field_indexing() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    pub struct Foo {
        a: b8,
        b: b8,
    }

    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = b.val();
        let c = Foo { a, b };
        signal(c.a + c.b)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_simple_if_expression() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<b8, Red> {
        let (a, b) = (a.val(), b.val());
        signal(if a > b { a + 1 } else { b + 2 })
    }
    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}

#[test]
fn test_link_to_bits_fn() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital)]
    struct Tuplo(b4, s6);

    #[derive(PartialEq, Debug, Default, Digital)]
    enum NooState {
        #[default]
        Init,
        Run(b4, s6),
        Walk {
            foo: b5,
        },
        Boom,
    }

    #[kernel]
    fn add_two<C: Domain>(a: Signal<b4, C>) -> Signal<b4, C> {
        signal(a.val() + 2)
    }

    #[kernel]
    fn add_one<C: Domain>(a: Signal<b4, C>) -> Signal<b4, C> {
        add_two::<C>(a)
    }

    #[kernel]
    fn add<C: Domain>(a: Signal<b4, C>) -> Signal<b4, C> {
        let a = a.val();
        let b = b4(3);
        let d = signed(11);
        let c = b + a;
        let c = c.resize();
        let _k = c.any();
        let h = Tuplo(c, d);
        let p = h.0;
        let _q = NooState::Run(c, d);
        signal(c + add_one::<C>(signal(p)).val() + if h.1 > 0 { 1 } else { 2 })
    }

    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(add::<Red>, tuple_exhaustive_red())?;
    Ok(())
}
