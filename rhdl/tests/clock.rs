#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;

#[test]
#[allow(clippy::let_and_return)]
fn test_struct_follows_clock_constraints() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital, Timed)]
    struct Foo<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[kernel]
    #[allow(clippy::let_and_return)]
    fn do_stuff<C: Domain, D: Domain>(s: Foo<C, D>) -> Foo<C, D> {
        let a = s.b.val();
        let b = s.a.val();
        let c = Foo::<C, D> {
            a: signal(a + 1),
            b: signal(b + 1),
        };
        c
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Green, Red>>().is_err());
    Ok(())
}

#[test]
fn test_struct_with_splice_follows_clock_constraints() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital, Timed)]
    struct Foo<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(mut s: Foo<C, D>) -> Foo<C, D> {
        let a = s.a.val();
        let b = s.b.val();
        s.a = signal(b);
        s
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Red, Green>>().is_err());
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_struct_follows_clock_constraints_fails() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital, Timed)]
    struct Foo<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(s: Foo<C, D>) -> Foo<C, D> {
        let a = s.a.val();
        let b = s.b.val();
        let c = Foo::<C, D> {
            a: signal(b + 1),
            b: signal(a + 1),
        };
        c
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_struct_cannot_cross_clock_domains() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Digital)]
    struct Foo {
        a: u8,
        b: u16,
        c: [u8; 3],
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(a: Signal<u8, C>, b: Signal<u8, D>) -> Signal<Foo, C> {
        let a = b.val();
        let d = Foo {
            a,
            b: 2,
            c: [1, 2, 3],
        }; // Struct literal
        signal(d)
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Red, Green>>().is_err());
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_signal_call_cross_clock_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>, y: Signal<b8, C>) -> Signal<b8, C> {
        x + y
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<b8, C> {
        let c = add::<C>(signal(a.val()), signal(b.val()));
        c
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_select_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        if y.val().any() {
            x
        } else {
            x + 2
        }
    }
    assert!(compile_design::<add::<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_select_causes_type_check_error() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        if y.val().any() {
            x
        } else {
            x + 2
        }
    }
    let Err(RHDLError::RHDLClockCoherenceViolation(_)) = compile_design::<add<Red, Green>>() else {
        panic!("Expected clock coherence violation");
    };
    Ok(())
}

#[test]
fn test_signal_coherence_in_splice_operation() -> miette::Result<()> {
    #[derive(Digital, Copy, Clone, PartialEq)]
    struct Baz {
        a: b8,
        b: b8,
        c: b8,
    }

    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<Baz, C>, y: Signal<b8, D>) -> Signal<Baz, C> {
        let x = x.val();
        let y = y.val();
        let z = Baz {
            b: y,
            c: bits(3),
            ..x
        };
        signal(z)
    }
    compile_design::<add<Red, Red>>()?;
    //    compile_design::<add<Red, Green>>()?;
    assert!(compile_design::<add<Green, Red>>().is_err());
    Ok(())
}

#[test]
fn test_signal_coherence_in_dynamic_indexing() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<[b8; 8], C>, y: Signal<b3, D>) -> Signal<b8, C> {
        let z = x[y.val()];
        signal(z)
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_coherence_in_binary_ops() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        let x = x.val();
        let y = y.val();
        let z = x + y;
        signal(z)
    }
    assert!(compile_design::<add<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_coherence_in_branches() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        let x = x.val();
        let y = y.val();
        let z = if y.any() { y } else { x };
        signal(z)
    }
    compile_design::<add<Green, Green>>()?;
    assert!(compile_design::<add<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_coherence_with_timed() -> miette::Result<()> {
    #[derive(Copy, Clone, Debug, PartialEq, Digital, Timed)]
    struct Baz<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Digital, Timed)]
    struct Container<C: Domain, D: Domain> {
        x: Baz<C, D>,
        y: Baz<C, D>,
    }

    #[kernel]
    fn add<C: Domain, D: Domain>(x: Container<C, D>) -> Signal<b8, C> {
        let val = x.y.b.val() + bits(1);
        signal(val)
    }
    assert!(compile_design::<add<Red, Green>>().is_err());
    compile_design::<add<Red, Red>>()?;
    Ok(())
}

#[test]
fn test_signal_carrying_struct() -> miette::Result<()> {
    #[derive(Copy, Clone, Debug, PartialEq, Digital)]
    struct Baz {
        a: b8,
        b: b8,
        c: b8,
    }

    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<Baz, C>, y: Signal<b8, D>) -> Signal<b8, D> {
        let x = x.val();
        let y = x.b + 1;
        signal(y)
    }
    assert!(compile_design::<add<Red, Green>>().is_err());
    compile_design::<add<Red, Red>>()?;
    Ok(())
}

#[test]
fn test_signal_coherence_with_const_in_binary_op() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>) -> Signal<b8, C> {
        let x = x.val();
        let y = bits(3);
        let z = x + y;
        signal(z)
    }
    compile_design::<add<Red>>()?;
    Ok(())
}

#[test]
fn test_signal_coherence_with_consts_ok() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>) -> Signal<b8, C> {
        let x = x.val();
        let y = bits(3);
        let z = if x.any() { x } else { y };
        signal(z)
    }
    compile_design::<add<Red>>()?;
    Ok(())
}

#[test]
fn test_signal_cast_works() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>, y: Signal<b8, C>) -> Signal<b8, C> {
        let z = x + y;
        signal::<b8, C>(z.val())
    }
    let obj = compile_design::<add<Red>>()?;
    eprintln!("{:?}", obj);
    Ok(())
}

#[test]
fn test_signal_cast_cross_clocks_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>) -> Signal<b8, D> {
        signal(x.val() + 3)
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_shifting_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>) -> Signal<b8, D> {
        let p = 4;
        let y: b8 = bits(7);
        let z = y << p;
        signal(x.val() << z)
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_cross_clock_indexing_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<[b8; 8], C>, y: Signal<b3, D>) -> Signal<b8, C> {
        let z = x[y.val()];
        signal(z)
    }
    assert!(compile_design::<add::<Red, Red>>().is_ok());
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_signal_tuple_crossing_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<(b8, b8), C> {
        let x = x.val();
        let y = y.val();
        let z = (x, y);
        signal(z)
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_signal_tuple_crossing_fails_second_test() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(
        x: Signal<b8, C>,
        y: Signal<b8, D>,
    ) -> (Signal<b8, C>, Signal<b8, D>) {
        let x = x.val();
        let y = y.val();
        let x = signal(x);
        let y = signal(y);
        let a = (y, x);
        a
    }
    compile_design::<add<Red, Red>>()?;
    assert!(compile_design::<add::<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_enum_basic_cross_clocks() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    enum Foo {
        A,
        B(b8),
    }

    #[kernel]
    fn foo<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<bool, D>) -> Signal<Foo, C> {
        let a = a.val();
        let b = b.val();
        let c = if b { Foo::A } else { Foo::B(a) };
        signal(c)
    }

    compile_design::<foo<Red, Red>>()?;
    assert!(compile_design::<foo<Red, Blue>>().is_err());
    Ok(())
}

#[test]
fn test_cross_clock_domains_fails_with_repeat() -> miette::Result<()> {
    #[kernel]
    fn foo<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<[b8; 3], C> {
        let a = a.val();
        let b = b.val();
        signal([b; 3])
    }

    compile_design::<foo<Red, Red>>()?;
    assert!(compile_design::<foo<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn cannot_mix_clocks_in_an_array() -> miette::Result<()> {
    #[kernel]
    fn foo<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<b8, C> {
        let a = a.val();
        let b = b.val();
        let c = [a, b, a];
        let d = c[0];
        signal(d)
    }
    compile_design::<foo<Red, Red>>()?;
    assert!(compile_design::<foo<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_exec_sub_kernel_preserves_clocking() -> miette::Result<()> {
    #[kernel]
    fn double<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        a + a
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<b8, C> {
        a + double::<C>(signal(b.val()))
    }

    compile_design::<do_stuff<Red, Red>>()?;
    assert!(compile_design::<do_stuff<Red, Green>>().is_err());
    Ok(())
}

#[test]
fn test_retime_of_comparison() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        let a = a.val();
        let b = b.val();
        let c = a > b;
        signal(c)
    }

    compile_design::<do_stuff<Red>>()?;
    Ok(())
}

#[test]
fn test_retime_of_comparison_with_structs() -> miette::Result<()> {
    #[derive(PartialEq, Copy, Clone, Debug, Digital)]
    struct Foo {
        a: b8,
    }

    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<Foo, C>, b: Signal<Foo, C>) -> Signal<bool, C> {
        let a = a.val();
        let b = b.val();
        let c = a == b;
        signal(c)
    }

    compile_design::<do_stuff<Red>>()?;
    Ok(())
}

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
#[ignore]
fn test_tuple_unused_variable() -> miette::Result<()> {
    #[kernel]
    fn do_stuff(a: Signal<b1, Red>) -> Signal<b1, Red> {
        let c = (3, a.val()); // c is domain (?, Red)
        signal(c.1)
    }

    compile_design::<do_stuff>()?;
    Ok(())
}
