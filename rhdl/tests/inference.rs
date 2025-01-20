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
use rhdl::core::sim::testbench::kernel::test_kernel_vm_and_verilog;

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_ast_basic_func_inferred_bits() -> miette::Result<()> {
    use rhdl::bits::alias::*;
    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum State {
        Init,
        Run(b8),
        Boom,
        #[default]
        Unknown,
    }

    #[derive(PartialEq, Digital)]
    pub struct Bar(pub b8, pub b8);

    #[kernel]
    fn do_stuff(arg: Signal<b4, Red>) -> Signal<b8, Red> {
        let arg = arg.val();
        let a = arg; // Straight local assignment
        let b = !a; // Unary operator
        let c = a + b - 1; // Binary operator
        let q = (a, b, c); // Tuple valued expression
        let (a, b, c) = q; // Tuple destructuring
        trace("abc", &(a, b, c)); // Trace statement
        let h = Bar(bits(1), bits(2)); // Tuple struct literal
        let _i = h.0; // Tuple struct field access
        let Bar(_j, _k) = h; // Tuple struct destructuring
        let _d = [1, 2, 3]; // Array literal
        let d = Foo {
            a: bits(1),
            b: bits(2),
            c: [bits(1), bits(2), bits(3)],
        }; // Struct literal
        let p = Foo { a: bits(4), ..d };
        trace("p", &p);
        let h = {
            let e = 3;
            let f = 4;
            b8(e) + b8(f)
        }; // Statement expression
        trace("h", &h);
        let Foo { a, b, .. } = d; // Struct destructuring
        trace("ab", &(a, b));
        let g = d.c[1]; // Array indexing
        let e = d.a; // Struct field access
        let mut d: b8 = bits(7); // Mutable local
        if d > bits(0) {
            // if statement
            d -= 1;
            // early return
            return signal(d);
        }
        // if-else statement (and a statement expression)
        let j = if d < bits(3) { b4(7) } else { b4(9) };
        // Enum literal
        let k = State::Boom;
        trace("gejk", &(g, e, j, k));
        // Enum literal with a payload
        let l = State::Run(bits(3));
        // Match expression with enum variants
        let j = match l {
            State::Init => 1,
            State::Run(_a) => 2,
            State::Boom => 3,
            _ => 4,
        };
        // For loops
        for ndx in 0..8 {
            d += ndx;
        }
        // block expression
        signal(42 + b8(j))
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
        trace("z", &z);
        a
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_signal_const_binop_inference() -> anyhow::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        let a = a.val();
        signal(a + 4)
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
        let x = x.val();
        let y = y.val();
        let z = z.val();
        let w = w.val();
        let ndx = ndx.val();
        // c, x, y are C
        let c = x + y;
        // d is C
        let d = x > y;
        // bx is C
        let bx = x;
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
        let h = z;
        trace("vars", &(zz, e, z2, res));
        // qq is Illegal!
        let qq = h + y;
        signal(match (z + 1 + qq).raw() {
            0 => z,
            _ => w,
        })
    }
    compile_design::<add<Red, Red>>(CompilationMode::Asynchronous)?;
    assert!(compile_design::<add::<Red, Green>>(CompilationMode::Asynchronous).is_err());
    Ok(())
}

#[test]
fn test_simple_type_inference() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b12, C>) -> Signal<b14, C> {
        let a = a.val();
        let k = a;
        let m = b14(7);
        let c: b12 = (k + 3).resize();
        let d = if c > k { c } else { k };
        let e = (c, m);
        let (f, g) = e;
        let h0 = g + 1;
        let _k: b4 = b4(7);
        let q = (b2(1), (b5(0), s8(5)), b12(6));
        let b = q.1 .1;
        let (q0, (q1, q1b), q2) = q; // Tuple destructuring
        let z = q1b + 4;
        let h = [d, c, f];
        let [i, j, k] = h;
        let o = j;
        let l = {
            let a = b12(3);
            let b = b12(4);
            a + b
        };
        trace("dump", &(k, b, z, i));
        trace("dump2", &(o, q0, q1, q2));
        signal((if h0.any() { l + k + 0 } else { l + k + 1 }).resize())
    }
    test_kernel_vm_and_verilog::<do_stuff<Red>, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_struct_inference_inferred_lengths() -> miette::Result<()> {
    use rhdl::bits::alias::*;
    use rhdl::bits::bits;

    #[derive(PartialEq, Digital)]
    pub struct Rad {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Digital)]
    pub struct Bar(pub b8, pub b8);

    #[derive(PartialEq, Default, Digital)]
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
    fn do_stuff(a: Signal<Foo, Red>) -> Signal<(b8, b8, NooState, Foo), Red> {
        let _z = (a.val().b, a.val().a);
        let _c = a;
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
        let Foo { a: ar, b: _, c: _ } = d;
        let _q = Bar(bits(1), bits(2));
        let x = NooState::Run(bits(1), bits(2));
        let e = ar;
        signal((e, ar, x, d))
    }
    let inputs = [
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad { x: b4(1), y: b6(2) },
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad { x: b4(1), y: b6(2) },
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
    use rhdl::bits::alias::*;
    use rhdl::bits::bits;

    #[derive(PartialEq, Digital)]
    pub struct Rad {
        x: b4,
        y: b6,
    }

    #[derive(PartialEq, Digital)]
    pub struct Foo {
        a: b8,
        b: s4,
        c: Rad,
    }

    #[derive(PartialEq, Digital)]
    pub struct Bar(pub b8, pub b8);

    #[derive(PartialEq, Default, Digital)]
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
    fn do_stuff(a: Signal<Foo, Red>) -> Signal<(b8, b8, NooState, Foo), Red> {
        let z = (a.val().b, a.val().a);
        let _c = a;
        let q = s4(-2);
        let c = Rad { x: b4(1), y: b6(2) };
        let d = Foo { a: b8(1), b: q, c };
        let Foo { a: ar, b, c: _ } = d;
        let q = Bar(bits(1), bits(2));
        trace("dump", &(z, c, b, q));
        let x = NooState::Run(b4(1), b5(2));
        let e = ar;
        signal((e, ar, x, d))
    }
    let inputs = [
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad { x: b4(1), y: b6(2) },
        },
        Foo {
            a: b8(1),
            b: s4(2),
            c: Rad { x: b4(1), y: b6(2) },
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
        match a.val().raw() {
            0 => c = bits(2),
            1 => c = bits(3),
            _ => {}
        }
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_bit_inference_with_explicit_length_works() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = a + 1;
        let c = b4(3);
        let d = b << c;
        signal(d)
    }
    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_resize_unsigned_inference() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<M: BitWidth>(a: Signal<b8, Red>) -> Signal<Bits<M>, Red> {
        let a = a.val();
        let b = a + 1;
        let c = b.resize();
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff<U12>, _, _, _>(do_stuff::<U12>, tuple_exhaustive_red())?;
    test_kernel_vm_and_verilog::<do_stuff<U8>, _, _, _>(do_stuff::<U8>, tuple_exhaustive_red())?;
    test_kernel_vm_and_verilog::<do_stuff<U4>, _, _, _>(do_stuff::<U4>, tuple_exhaustive_red())?;
    Ok(())
}

#[test]
fn test_resize_signed_inferred() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<M: BitWidth>(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<SignedBits<M>, Red> {
        let (a, b) = (a.val(), b.val());
        let c = a + b;
        let c = c.resize();
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff<U12>, _, _, _>(do_stuff::<U12>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<do_stuff<U4>, _, _, _>(do_stuff::<U4>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<do_stuff<U8>, _, _, _>(do_stuff::<U8>, tuple_pair_s8_red())?;
    Ok(())
}

#[test]
fn test_resize_signed_explicit() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<N: BitWidth>(a: Signal<s8, Red>, b: Signal<s8, Red>) -> Signal<s8, Red> {
        let (a, b) = (a.val(), b.val());
        let c = a + b;
        let c = c.resize::<N>();
        let c = c.resize();
        signal(c)
    }
    test_kernel_vm_and_verilog::<do_stuff<U12>, _, _, _>(do_stuff::<U12>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<do_stuff<U4>, _, _, _>(do_stuff::<U4>, tuple_pair_s8_red())?;
    Ok(())
}

#[test]
fn test_bit_inference_works() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b8, Red>) -> Signal<b8, Red> {
        let a = a.val();
        let b = a + 1;
        let c = 3;
        signal(b + c)
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
