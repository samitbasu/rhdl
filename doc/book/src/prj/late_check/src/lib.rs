use rhdl::prelude::*;

#[kernel]
fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Red>) -> Signal<b5, Red> {
    let a1 = a1.val().dyn_bits();
    let a2 = a2.val().dyn_bits();
    let c = a1.xadd(a2);
    let d = c.xadd(b1(1));
    let e: b5 = d.as_bits();
    signal(e)
}

#[test]
fn test_compile_do_stuff() -> miette::Result<()> {
    env_logger::init();
    let _ = compile_design::<do_stuff>(CompilationMode::Asynchronous)?;
    Ok(())
}

#[test]
fn test_run_do_stuff() {
    let y = do_stuff(signal(b4(3)), signal(b4(5))).val();
    assert_eq!(y, b5(9));
}
