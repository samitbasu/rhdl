#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;

#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func_inferred_bits() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum State {
        Init,
        Run(u8),
        Boom,
        #[rhdl(unmatched)]
        Unknown,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[kernel]
    fn do_stuff(arg: Signal<b4, Red>) -> Signal<b8, Red> {
        let a = arg; // Straight local assignment
        let b = !a; // Unary operator
        let c = a + (b - 1); // Binary operator
        let q = (a, b, c); // Tuple valued expression
        let (a, b, c) = q; // Tuple destructuring
        let h = Bar(1, 2); // Tuple struct literal
        let i = h.0; // Tuple struct field access
        let Bar(j, k) = h; // Tuple struct destructuring
        let d = [1, 2, 3]; // Array literal
        let d = Foo {
            a: 1,
            b: 2,
            c: [1, 2, 3],
        }; // Struct literal
        let p = Foo { a: 4, ..d };
        let h = {
            let e = 3;
            let f = 4;
            b8(e) + b8(f)
        }; // Statement expression
        let Foo { a, b, .. } = d; // Struct destructuring
        let g = d.c[1]; // Array indexing
        let e = d.a; // Struct field access
        let mut d: b8 = bits(7); // Mutable local
        if d > bits(0) {
            // if statement
            d = d - bits(1);
            // early return
            return signal(d);
        }
        // if-else statement (and a statement expression)
        let j = if d < bits(3) { 7 } else { 9 };
        // Enum literal
        let k = State::Boom;
        // Enum literal with a payload
        let l = State::Run(3);
        // Match expression with enum variants
        let j = match l {
            State::Init => b3(1),
            State::Run(a) => b3(2),
            State::Boom => b3(3),
            _ => b3(4),
        };
        // For loops
        for ndx in 0..8 {
            d = d + bits(ndx);
        }
        // block expression
        signal(bits(42))
    }

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_bits_inference_with_type() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        let y: b8 = bits(3);
        let r = 3;
        let z = y << r;
        a
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_signal_const_binop_inference() -> anyhow::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        a + b8(4)
    }
    compile_design::<do_stuff<Red>>(CompilationMode::Asynchronous)?;
    Ok(())
}

#[test]
fn test_signal_ops_inference() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(
        x: Signal<b8, C>,
        y: Signal<b8, C>,
        z: Signal<b8, D>,
        w: Signal<b8, D>,
        ndx: Signal<b8, C>,
    ) -> Signal<b8, D> {
        // c, x, y are C
        let c = x + y;
        // d is C
        let d = x > y;
        // bx is C
        let bx = x.val();
        // zz is C
        let zz = 2 < bx;
        // e is C
        let e = d && (!d ^ d);
        // q is D
        let q = z > w;
        // x is C
        let x = [c, c, c];
        // z2 is C
        let z2 = x[ndx];
        // res is D
        let res = if q { w } else { z };
        // h is D
        let h = z.val();
        // qq is Illegal!
        let qq = h + y.val();
        match (z + 1 + qq).val() {
            Bits::<8>(0) => z,
            _ => w,
        }
    }
    compile_design::<add<Red, Red>>(CompilationMode::Asynchronous)?;
    assert!(compile_design::<add::<Red, Green>>(CompilationMode::Asynchronous).is_err());
    Ok(())
}

#[test]
fn test_simple_type_inference() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b12, C>) -> Signal<b12, C> {
        let k = a;
        let m = bits::<14>(7);
        let c = k + 3;
        let d = if c > k { c } else { k };
        let e = (c, m);
        let (f, g) = e;
        let h0 = g + 1;
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
        if h0.any() {
            l + k
        } else {
            l + k + 1
        }
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_struct_inference_inferred_lengths() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Rad {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn do_stuff(a: Signal<Foo, Red>) -> Signal<(b8, b8, NooState, Foo), Red> {
        let z = (a.val().b, a.val().a);
        let c = a;
        let q = signed(-2);
        let c = Rad {
            x: bits(1),
            y: bits(2),
        };
        let d = Foo {
            a: bits(1),
            b: q,
            c,
        };
        let Foo { a: ar, b, c: _ } = d;
        let q = Bar(1, 2);
        let x = NooState::Run(bits(1), bits(2));
        let e = ar;
        signal((e, ar, x, d))
    }
    let inputs = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(
        do_stuff,
        inputs.into_iter().map(|x| (signal(x),)),
    )?;
    Ok(())
}

#[test]
fn test_struct_inference() -> miette::Result<()> {
    use rhdl_bits::alias::*;
    use rhdl_bits::bits;

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Rad {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub struct Bar(pub u8, pub u8);

    #[derive(PartialEq, Copy, Clone, Digital)]
    pub enum NooState {
        Init,
        Run(b4, b5),
        Walk { foo: b5 },
        Boom,
    }

    #[kernel]
    fn do_stuff(a: Signal<Foo, Red>) -> Signal<(b8, b8, NooState, Foo), Red> {
        let z = (a.val().b, a.val().a);
        let c = a;
        let q = signed::<4>(-2);
        let c = Rad {
            x: bits::<4>(1),
            y: bits::<6>(2),
        };
        let d = Foo {
            a: bits::<8>(1),
            b: q,
            c,
        };
        let Foo { a: ar, b, c: _ } = d;
        let q = Bar(1, 2);
        let x = NooState::Run(bits::<4>(1), bits::<5>(2));
        let e = ar;
        signal((e, ar, x, d))
    }
    let inputs = [
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
        Foo {
            a: bits::<8>(1),
            b: signed::<4>(2),
            c: Rad {
                x: bits::<4>(1),
                y: bits::<6>(2),
            },
        },
    ];
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(
        do_stuff,
        inputs.into_iter().map(|x| (signal(x),)),
    )?;
    Ok(())
}

#[test]
fn test_missing_register_inferred_types() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b8, Red> {
        let mut c = bits(0);
        match a.val() {
            Bits::<1>(0) => c = bits(2),
            Bits::<1>(1) => c = bits(3),
            _ => {}
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_bit_inference_works() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let b = a + 1;
        let c = bits(3);
        b + c
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_array_inference() -> miette::Result<()> {
    #[kernel]
    fn foo(a: Signal<b8, Red>, b: Signal<b8, Red>) -> Signal<[b8; 2], Red> {
        let a = a.val();
        let b = b.val();
        let c = [a, b];
        signal(c)
    }

    test_kernel_vm_and_verilog::<foo, _, _, _>(foo, tuple_pair_b8_red())?;
    Ok(())
}
