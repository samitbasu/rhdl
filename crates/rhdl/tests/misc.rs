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
    #[derive(PartialEq,Clone,Digital)]
    pub struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[derive(PartialEq, Default, Clone,Digital)]
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
    #[derive(PartialEq, Default, Clone,Digital)]
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
        match w {
            MyEnum::A => bits(1),
            MyEnum::B(a, _) => a,
            MyEnum::C { x, y: _, z: _ } => x,
        }
    }
    // Get the RHIF implementation of the kernel
    let obj = compile_design_stage1::<do_stuff>(CompilationMode::Synchronous)?;
    Ok(())
}

#[test]
fn test_latte_opcode() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(y: bool) -> b4 {
        let mut z = bits(0);
        if y {
            z += 1;
            z
        } else {
            z
        }
    }
    // Get the RHIF implementation of the kernel
    let _obj = compile_design_stage1::<do_stuff>(CompilationMode::Synchronous);
    Ok(())
}

#[test]
fn test_svg_diagram() {
    #[derive(PartialEq, Default, Clone,Digital)]
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
    #[derive(PartialEq, Default, Clone,Digital)]
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

#[test]
fn test_layout_example_latte25() -> miette::Result<()> {
    #[derive(PartialEq, Default, Clone,Digital)]
    #[rhdl(discriminant_width = "4")]
    pub enum Register {
        #[default]
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
    }

    #[derive(PartialEq, Default, Clone,Digital)]
    #[rhdl(discriminant_align = "lsb")]
    pub enum Target {
        #[default]
        Zero,
        Register(Register),
        Literal(b8),
    }

    #[derive(PartialEq, Default, Clone,Digital)]
    pub enum OpCode {
        #[default]
        Nop,
        Add(Target, Target),
        Mul(Register, b8),
    }

    fn decode(t: Target) -> b8 {
        todo!()
    }

    fn register_val(r: Register) -> b8 {
        todo!()
    }

    fn alu(t: OpCode) -> Option<b8> {
        match t {
            OpCode::Nop => None,
            OpCode::Add(t1, t2) => {
                let v1 = decode(t1);
                let v2 = decode(t2);
                Some(v1 + v2)
            }
            OpCode::Mul(r1, s) => {
                let v1 = register_val(r1);
                Some(v1 * s)
            }
        }
    }

    #[kernel]
    fn alu_stuff(a: b8, b: b8) -> b8 {
        let c = a + b;
        let d = a | b;
        let e = a * b;
        c + d + e
    }

    #[kernel]
    #[allow(clippy::let_and_return)]
    fn clock_cross_fails(a: Signal<b8, Red>, b: Signal<b8, Blue>) -> Signal<b8, Red> {
        let a = a.val(); // <--- Erases the clock domain of a
        let b = b.val(); // <--- Erases the clock domain of b
        let c = a | b; // <--- Combines the two signals with an OR gate
        let d = signal(c); // <--- rustc infers Red based on return type
        d // <--- All good, right??
    }

    // Uncomment to get an error report.
    //    compile_design::<clock_cross_fails>(CompilationMode::Asynchronous)?;

    let doc = OpCode::static_kind().svg("OpCode");
    svg::save("op_code.svg", &doc).unwrap();
    Ok(())
}
