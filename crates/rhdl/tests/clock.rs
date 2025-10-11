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

use CompilationMode::Asynchronous;

#[test]
#[allow(clippy::let_and_return)]
fn test_struct_follows_clock_constraints() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Copy, Timed, Clone)]
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
            a: signal((a + 1).resize()),
            b: signal((b + 1).resize()),
        };
        c
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["struct_follows_clock_constraints.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_struct_with_splice_follows_clock_constraints() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Copy, Timed, Clone)]
    struct Foo<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(mut s: Foo<C, D>) -> Foo<C, D> {
        let _a = s.a.val();
        let b = s.b.val();
        s.a = signal(b);
        s
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["struct_with_splice_follows_clock_constraints.expect"]
        .assert_eq(&report);
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_struct_follows_clock_constraints_fails() -> miette::Result<()> {
    #[derive(PartialEq, Digital, Copy, Timed, Clone)]
    struct Foo<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(s: Foo<C, D>) -> Foo<C, D> {
        let a = s.a.val();
        let b = s.b.val();
        let c = Foo::<C, D> {
            a: signal((b + 1).resize()),
            b: signal((a + 1).resize()),
        };
        c
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["struct_follows_clock_constraints_fails.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_struct_cannot_cross_clock_domains() -> miette::Result<()> {
    #[derive(PartialEq, Clone, Copy, Digital)]
    struct Foo {
        a: b8,
        b: b16,
        c: [b8; 3],
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(_a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<Foo, C> {
        let a = b.val();
        let d = Foo {
            a,
            b: bits(2),
            c: [bits(1), bits(2), bits(3)],
        }; // Struct literal
        signal(d)
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["struct_cannot_cross_clock_domains.expect"].assert_eq(&report);
    Ok(())
}

#[test]
#[allow(clippy::let_and_return)]
fn test_signal_call_cross_clock_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>, y: Signal<b8, C>) -> Signal<b8, C> {
        signal(x.val() + y.val())
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<b8, C> {
        let c = add::<C>(signal(a.val()), signal(b.val()));
        c
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_call_cross_clock_fails.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_cross_clock_select_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        let x = x.val();
        signal(if y.val().any() { x } else { (x + 2).resize() })
    }
    assert!(compile_design::<add::<Red, Red>>(Asynchronous).is_ok());
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_cross_clock_select_fails.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_cross_clock_select_causes_type_check_error() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        let x = x.val();
        let z = if y.val().any() { x } else { (x + 2).resize() };
        signal(z)
    }
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_cross_clock_select_causes_type_check_error.expect"]
        .assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_coherence_in_splice_operation() -> miette::Result<()> {
    #[derive(PartialEq, Clone, Copy, Digital)]
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
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_coherence_in_splice_operation.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_coherence_in_dynamic_indexing() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<[b8; 8], C>, y: Signal<b3, D>) -> Signal<b8, C> {
        let x = x.val();
        let z = x[y.val()];
        signal(z)
    }
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_coherence_in_dynamic_indexing.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_coherence_in_binary_ops() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>, y: Signal<b8, D>) -> Signal<b8, C> {
        let x = x.val();
        let y = y.val();
        let z = x + y;
        signal(z.resize())
    }
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_coherence_in_binary_ops.expect"].assert_eq(&report);
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
    compile_design::<add<Green, Green>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_coherence_in_branches.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_coherence_with_timed() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
    struct Baz<C: Domain, D: Domain> {
        a: Signal<b8, C>,
        b: Signal<b8, D>,
    }

    #[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
    struct Container<C: Domain, D: Domain> {
        x: Baz<C, D>,
        y: Baz<C, D>,
    }

    #[kernel]
    fn add<C: Domain, D: Domain>(x: Container<C, D>) -> Signal<b8, C> {
        let val = x.y.b.val() + 1;
        signal(val)
    }
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_coherence_with_timed.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_carrying_struct() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
    struct Baz {
        a: b8,
        b: b8,
        c: b8,
    }

    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<Baz, C>, _y: Signal<b8, D>) -> Signal<b8, D> {
        let x = x.val();
        let y = x.b + 1;
        signal(y)
    }
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_carrying_struct.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_coherence_with_const_in_binary_op() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>) -> Signal<b8, C> {
        let x = x.val();
        let y = b8(3);
        let z = x + y;
        signal(z.resize())
    }
    compile_design::<add<Red>>(Asynchronous)?;
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
    compile_design::<add<Red>>(Asynchronous)?;
    Ok(())
}

#[test]
fn test_signal_cast_works() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain>(x: Signal<b8, C>, y: Signal<b8, C>) -> Signal<b8, C> {
        let x = x.val();
        let y = y.val();
        let z = x + y;
        signal::<b8, C>(z.resize())
    }
    let obj = compile_design::<add<Red>>(Asynchronous)?;
    Ok(())
}

#[test]
fn test_signal_cast_cross_clocks_fails() -> miette::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>) -> Signal<b8, D> {
        signal(x.val() + 3)
    }
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_cast_cross_clocks_fails.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_cross_clock_shifting_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<b8, C>) -> Signal<b8, D> {
        let p = 4;
        let y: b8 = bits(7);
        let z: b3 = (y << p).resize();
        signal(x.val() << z)
    }
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_cross_clock_shifting_fails.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_signal_cross_clock_indexing_fails() -> anyhow::Result<()> {
    #[kernel]
    fn add<C: Domain, D: Domain>(x: Signal<[b8; 8], C>, y: Signal<b3, D>) -> Signal<b8, C> {
        let x = x.val();
        let z = x[y.val()];
        signal(z)
    }
    assert!(compile_design::<add::<Red, Red>>(Asynchronous).is_ok());
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_cross_clock_indexing_fails.expect"].assert_eq(&report);
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
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_tuple_crossing_fails.expect"].assert_eq(&report);
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
    compile_design::<add<Red, Red>>(Asynchronous)?;
    let err = compile_design::<add<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["signal_tuple_crossing_fails_second_test.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_enum_basic_cross_clocks() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Default, Clone, Copy, Digital)]
    enum Foo {
        #[default]
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

    compile_design::<foo<Red, Red>>(Asynchronous)?;
    let err = compile_design::<foo<Red, Blue>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["enum_basic_cross_clocks.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_cross_clock_domains_fails_with_repeat() -> miette::Result<()> {
    #[kernel]
    fn foo<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<[b8; 3], C> {
        let _a = a.val();
        let b = b.val();
        signal([b; 3])
    }

    compile_design::<foo<Red, Red>>(Asynchronous)?;
    let err = compile_design::<foo<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["cross_clock_domains_fails_with_repeat.expect"].assert_eq(&report);
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
    compile_design::<foo<Red, Red>>(Asynchronous)?;
    let err = compile_design::<foo<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["cannot_mix_clocks_in_an_array.expect"].assert_eq(&report);
    Ok(())
}

#[test]
fn test_exec_sub_kernel_preserves_clocking() -> miette::Result<()> {
    #[kernel]
    fn double<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        signal(a.val() + a.val())
    }

    #[kernel]
    fn do_stuff<C: Domain, D: Domain>(a: Signal<b8, C>, b: Signal<b8, D>) -> Signal<b8, C> {
        signal(a.val() + double::<C>(signal(b.val())).val())
    }

    compile_design::<do_stuff<Red, Red>>(Asynchronous)?;
    let err = compile_design::<do_stuff<Red, Green>>(Asynchronous)
        .expect_err("Expected this to fail with a clock coherence error");
    let report = miette_report(err);
    expect_test::expect_file!["exec_sub_kernel_preserves_clocking.expect"].assert_eq(&report);
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

    compile_design::<do_stuff<Red>>(Asynchronous)?;
    Ok(())
}

#[test]
fn test_retime_of_comparison_with_structs() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Clone, Copy)]
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

    compile_design::<do_stuff<Red>>(Asynchronous)?;
    Ok(())
}

#[test]
fn test_unknown_clock_domain() -> miette::Result<()> {
    #[kernel]
    fn do_stuff<C: Domain>(a: Signal<b12, C>) -> Signal<b12, C> {
        let a = a.val();
        let k = a;
        let m = b14(7);
        let c: b12 = (k + 3).resize();
        let d = if c > k { c } else { k };
        let e = (c, m);
        let (f, g) = e;
        let h = g + 1;
        let k: b4 = b4(7);
        trace("hk", &(h, k));
        let q = (b2(1), (b5(0), s8(5)), b12(6));
        let b = q.1.1;
        trace("b", &b);
        let (q0, (q1, q1b), q2) = q; // Tuple destructuring
        trace("q", &(q0, q1, q1b, q2));
        let z = q1b + 4;
        trace("z", &z);
        let h = [d, c, f];
        let [_i, j, k] = h;
        let _o = j;
        let l = {
            let a = b12(3);
            let b = 4;
            a + b
        };
        signal(l + k)
    }
    compile_design::<do_stuff<Red>>(Asynchronous)?;
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

    compile_design::<do_stuff>(Asynchronous)?;
    Ok(())
}
