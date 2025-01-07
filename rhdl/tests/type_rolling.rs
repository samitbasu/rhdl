#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unreachable_code)]
#![allow(unused_must_use)]
#![allow(dead_code)]

use rhdl::prelude::*;
use rhdl_core::compiler::mir::error::Syntax;

#[test]
fn test_roll_your_own_binop_fails() -> miette::Result<()> {
    #[derive(Digital)]
    struct Baz {
        a: b8,
    }
    macro_rules! impl_binop {
        ($trait:ident, $method:ident, $op:tt, $kernel:ident) => {
            impl std::ops::$trait for Baz {
                type Output = Baz;
                fn $method(self, _rhs: Baz) -> Baz {
                    self
                }
            }
            #[kernel]
            fn $kernel(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
                let h = h.val();
                let j = h $op h;
                signal(j)
            }

            let Err(RHDLError::RHDLSyntaxError(err)) = compile_design::<$kernel>(CompilationMode::Asynchronous) else {
                panic!("Expected syntax error");
            };

        };
    }

    impl_binop!(Add, add, +, do_stuff_add);
    impl_binop!(Sub, sub, - , do_stuff_sub);
    impl_binop!(BitAnd, bitand, &, do_stuff_and);
    impl_binop!(BitOr, bitor, | , do_stuff_or);
    impl_binop!(BitXor, bitxor, ^, do_stuff_xor);
    impl_binop!(Mul, mul, *, do_stuff_mul);
    Ok(())
}

#[test]
fn test_roll_your_own_not_fails() -> miette::Result<()> {
    #[derive(Digital)]
    struct Baz {
        a: b8,
    }

    impl std::ops::Not for Baz {
        type Output = Baz;
        fn not(self) -> Baz {
            self
        }
    }

    impl std::ops::Neg for Baz {
        type Output = Baz;
        fn neg(self) -> Baz {
            self
        }
    }

    impl Baz {
        fn any(&self) -> bool {
            false
        }
    }

    #[kernel]
    fn do_stuff(h: Signal<Baz, Red>) -> Signal<bool, Red> {
        let h = h.val();
        let j = !h;
        signal(j.any())
    }

    // Assert that the compilation fails with a RHDL syntax error
    let Err(RHDLError::RHDLSyntaxError(err)) =
        compile_design::<do_stuff>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    assert!(matches!(err.cause, Syntax::RollYourOwnUnary { op: _ }));

    #[kernel]
    fn do_stuff_neg(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
        let h = h.val();
        let j = -h;
        signal(j)
    }
    // Assert that the compilation fails with a RHDL syntax error
    let Err(RHDLError::RHDLSyntaxError(err)) =
        compile_design::<do_stuff_neg>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    assert!(matches!(
        err.cause,
        rhdl_core::compiler::mir::error::Syntax::RollYourOwnUnary { op: _ }
    ));

    Ok(())
}

#[test]
fn test_roll_your_own_val_fails() -> miette::Result<()> {
    #[derive(Digital)]
    struct Baz {
        a: b8,
    }

    impl Baz {
        fn val(self) -> Self {
            self
        }
    }

    #[kernel]
    fn do_stuff(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
        let h = h.val();
        let j = h.val();
        signal(j)
    }

    // Assert that the compilation fails with a RHDL syntax error
    let Err(RHDLError::RHDLSyntaxError(err)) =
        compile_design::<do_stuff>(CompilationMode::Asynchronous)
    else {
        panic!("Expected syntax error");
    };
    Ok(())
}

#[test]
fn test_method_call_fails_with_roll_your_own() -> miette::Result<()> {
    #[derive(Digital)]
    struct Baz {
        a: b8,
    }

    impl Baz {
        fn any(&self) -> bool {
            false
        }
        fn all(&self) -> bool {
            false
        }
        fn xor(&self) -> bool {
            false
        }
        fn as_signed(self) -> Self {
            self
        }
        fn as_unsigned(self) -> Self {
            self
        }
        fn val(self) -> Self {
            self
        }
    }

    #[kernel]
    fn do_val(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
        let h = h.val();
        let h = h.val();
        signal(h)
    }

    #[kernel]
    fn do_signed(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
        let h = h.val();
        let h = h.as_signed();
        signal(h)
    }

    #[kernel]
    fn do_unsigned(h: Signal<Baz, Red>) -> Signal<Baz, Red> {
        let h = h.val();
        let h = h.as_unsigned();
        signal(h)
    }

    #[kernel]
    fn do_xor(h: Signal<Baz, Red>) -> Signal<bool, Red> {
        let h = h.val();
        let j = h.xor();
        signal(j)
    }

    #[kernel]
    fn do_any(h: Signal<Baz, Red>) -> Signal<bool, Red> {
        let h = h.val();
        let j = h.any();
        signal(j)
    }

    #[kernel]
    fn do_all(h: Signal<Baz, Red>) -> Signal<bool, Red> {
        let h = h.val();
        let j = h.all();
        signal(j)
    }

    assert!(matches!(
        compile_design::<do_val>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    assert!(matches!(
        compile_design::<do_signed>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    assert!(matches!(
        compile_design::<do_unsigned>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    assert!(matches!(
        compile_design::<do_xor>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    assert!(matches!(
        compile_design::<do_any>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    assert!(matches!(
        compile_design::<do_all>(CompilationMode::Asynchronous),
        Err(RHDLError::RHDLSyntaxError(err))
    ));
    Ok(())
}
