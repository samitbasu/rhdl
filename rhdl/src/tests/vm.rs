use crate::tests::{red, tuple_exhaustive_red, tuple_pair_b8_red, tuple_pair_s8_red};
use rhdl_bits::{alias::*, bits, signed};
use rhdl_core::{
    compile_design,
    rhif::vm::execute_function,
    test_kernel_vm_and_verilog,
    types::{domain::Red, signal::signal},
    Digital, Domain, Signal,
};
use rhdl_macro::kernel;

#[test]
fn test_vm_simple_function() {
    #[kernel]
    fn pass<C: Domain>(a: Signal<b8, C>) -> Signal<b8, C> {
        a
    }

    test_kernel_vm_and_verilog::<pass<Red>, _, _, _>(pass, tuple_exhaustive_red()).unwrap();
}

#[test]
fn test_vm_simple_function_with_invalid_args_causes_ice() {
    #[kernel]
    fn pass<C: Domain>(a: Signal<u8, C>) -> Signal<u8, C> {
        a
    }
    let design = compile_design::<pass<Red>>().unwrap();
    eprintln!("design: {:?}", design);
    let res = execute_function(&design, vec![(42_u16).typed_bits()]);
    assert!(res.is_err());
}

#[test]
fn test_vm_simple_binop_function() {
    #[kernel]
    fn add<C: Domain>(a: Signal<b12, C>, b: Signal<b12, C>) -> Signal<b12, C> {
        a + b + b
    }

    let tests = [
        (bits(3), bits(17)),
        (bits(1), bits(42)),
        (bits(1000), bits(32)),
    ];

    test_kernel_vm_and_verilog::<add<Red>, _, _, _>(
        add::<Red>,
        tests.into_iter().map(|x| (red(x.0), red(x.1))),
    )
    .unwrap();
}

#[test]
fn test_vm_unsigned_binop_function() {
    #[kernel]
    fn gt<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a > b)
    }

    #[kernel]
    fn ge<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a >= b)
    }

    #[kernel]
    fn eq<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a == b)
    }

    #[kernel]
    fn ne<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a != b)
    }

    #[kernel]
    fn le<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a <= b)
    }

    #[kernel]
    fn lt<C: Domain>(a: Signal<b8, C>, b: Signal<b8, C>) -> Signal<bool, C> {
        signal(a < b)
    }

    test_kernel_vm_and_verilog::<gt<Red>, _, _, _>(gt::<Red>, tuple_pair_b8_red()).unwrap();
    test_kernel_vm_and_verilog::<ge<Red>, _, _, _>(ge::<Red>, tuple_pair_b8_red()).unwrap();
    test_kernel_vm_and_verilog::<eq<Red>, _, _, _>(eq::<Red>, tuple_pair_b8_red()).unwrap();
    test_kernel_vm_and_verilog::<ne<Red>, _, _, _>(ne::<Red>, tuple_pair_b8_red()).unwrap();
    test_kernel_vm_and_verilog::<le<Red>, _, _, _>(le::<Red>, tuple_pair_b8_red()).unwrap();
    test_kernel_vm_and_verilog::<lt<Red>, _, _, _>(lt::<Red>, tuple_pair_b8_red()).unwrap();
}

#[test]
fn test_vm_signed_binop_function() -> miette::Result<()> {
    #[kernel]
    fn gt<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a > b)
    }

    #[kernel]
    fn ge<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a >= b)
    }

    #[kernel]
    fn eq<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a == b)
    }

    #[kernel]
    fn ne<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a != b)
    }

    #[kernel]
    fn le<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a <= b)
    }

    #[kernel]
    fn lt<C: Domain>(a: Signal<s8, C>, b: Signal<s8, C>) -> Signal<bool, C> {
        signal(a < b)
    }

    test_kernel_vm_and_verilog::<gt<Red>, _, _, _>(gt::<Red>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<ge<Red>, _, _, _>(ge::<Red>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<eq<Red>, _, _, _>(eq::<Red>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<ne<Red>, _, _, _>(ne::<Red>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<le<Red>, _, _, _>(le::<Red>, tuple_pair_s8_red())?;
    test_kernel_vm_and_verilog::<lt<Red>, _, _, _>(lt::<Red>, tuple_pair_s8_red())?;
    Ok(())
}

#[test]
fn test_vm_shr_is_sign_preserving() {
    #[kernel]
    fn shr<C: Domain>(a: Signal<s12, C>, b: Signal<b4, C>) -> Signal<s12, C> {
        let a = a.val();
        let b = b.val();
        signal(a >> b)
    }

    let test = [(red(signed(-42)), red(bits(2)))];
    test_kernel_vm_and_verilog::<shr<Red>, _, _, _>(shr, test.into_iter()).unwrap();
}
