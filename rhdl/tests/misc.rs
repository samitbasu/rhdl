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
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
fn test_missing_register() {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let mut c = b8(0);
        match a.val().raw() {
            0 => c = b8(2),
            1 => c = b8(3),
            _ => {}
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red()).unwrap();
}

#[test]
#[should_panic]
fn test_cast_to_b16_of_big_number_fails() {
    let x: b16 = bits(5 + !8);
}

#[test]
#[allow(clippy::needless_late_init)]
#[allow(clippy::no_effect)]
fn test_compile() -> miette::Result<()> {
    use rhdl::bits::alias::*;
    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[derive(PartialEq, Default, Digital)]
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
    fn do_stuff<C: Domain>(mut a: Signal<Foo, C>, mut s: Signal<NooState, C>) -> Signal<Foo, C> {
        let _k = {
            b12(4) + 6;
            b12(6)
        };
        let mut a: Foo = a.val();
        let mut s: NooState = s.val();
        let _q = if a.a > 0 { b12(3) } else { b12(0) };
        let y = b12(72);
        let _t2 = (y, y);
        let q: b8 = bits(4);
        let _z = a.c;
        let _w = (a, a);
        a.c[1] = (q + 3).resize();
        a.c = [bits(0); 3];
        a.c = [bits(1), bits(2), bits(3)];
        let q = (bits(1), (bits(0), bits(5)), bits(6));
        let (q0, (q1, q1b), q2): (b8, (b8, b8), b16) = q; // Tuple destructuring
        a.a = (2 + 3 + q1 + q0 + q1b + if q2 != 0 { 1 } else { 0 }).resize();
        let z;
        if 1 > 3 {
            z = b4(2);
        } else {
            z = b4(5);
        }
        a.b = {
            7 + 9;
            bits(5 + 8)
        };
        a.a = if 1 > 3 {
            bits(7)
        } else {
            {
                a.b = bits(1);
                a.b = bits(4);
            }
            bits(9)
        };
        let g = 1 > 2;
        let h = 3 != 4;
        let mut _i = g && h;
        if z == b4(3) {
            _i = false;
        }
        let _c = b4(match z.raw() {
            4 => 1,
            1 => 2,
            2 => 3,
            3 => {
                a.a = bits(4);
                4
            }
            _ => 6,
        });
        let _d = match s {
            NooState::Init => {
                a.a = bits(1);
                NooState::Run(bits(1), bits(2), bits(3))
            }
            NooState::Walk { foo: x } => {
                a.a = x;
                NooState::Boom
            }
            NooState::Run(x, _t, y) => {
                a.a = (x + y).resize();
                NooState::Walk { foo: bits(7) }
            }
            NooState::Boom => {
                a.a = (a.a + 3).resize();
                NooState::Init
            }
        };
        signal(a)
    }
    let foos = [
        Foo {
            a: bits(1),
            b: bits(2),
            c: [bits(1), bits(2), bits(3)],
        },
        Foo {
            a: bits(4),
            b: bits(5),
            c: [bits(4), bits(5), bits(6)],
        },
    ];
    let noos = [
        NooState::Init,
        NooState::Boom,
        NooState::Run(bits(1), bits(2), bits(3)),
        NooState::Walk { foo: bits(4) },
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
        let b = a.val() + 1;
        let _c = b4(3);
        a = signal(b.resize());
    }
}

#[test]
fn test_latte_match() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Default)]
    enum MyEnum {
        #[default]
        A, // No payload
        B(b4, b6), // 4-bit and 6-bit tuple payload
        C {
            x: b4,
            y: b6,
            z: [b3; 3],
        }, // Struct payload
    }

    #[kernel]
    fn do_stuff(w: MyEnum) -> b4 {
        let v = match w {
            MyEnum::A => bits(1),
            MyEnum::B(a, _) => a,
            MyEnum::C { x, y: _, z: _ } => x,
        };
        v
    }
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();
    // Get the RHIF implementation of the kernel
    let obj = compile_design_stage1::<do_stuff>(CompilationMode::Synchronous)?;
    eprintln!("{:?}", obj);
    Ok(())
}

#[test]
fn test_latte_opcode() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(y: bool) -> b4 {
        let mut z = bits(0);
        let x = if y {
            z += 1;
            z
        } else {
            z
        };
        x
    }
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();
    // Get the RHIF implementation of the kernel
    let obj = compile_design_stage1::<do_stuff>(CompilationMode::Synchronous)?;
    Ok(())
}

#[test]
fn test_svg_diagram() {
    #[derive(PartialEq, Digital, Default)]
    pub enum MyEnum {
        #[default]
        A,
        B(b4, b6),
        C {
            x: b4,
            y: b6,
            z: [b3; 3],
        },
    }

    // Generate an SVG for this
    let doc = MyEnum::static_kind().svg("MyEnum");
    svg::save("my_enum.svg", &doc).unwrap();
}

#[test]
fn test_layout_compressed() {
    #[derive(PartialEq, Digital, Default)]
    pub enum MyEnum {
        #[default]
        A,
        B(b4, b6),
        C {
            x: b4,
            y: b6,
            z: [b3; 3],
        },
        D(bool),
    }
    let doc = MyEnum::static_kind().svg("MyEnum");
    svg::save("my_enum.svg", &doc).unwrap();
}
