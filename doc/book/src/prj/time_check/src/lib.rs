use rhdl::prelude::*;

#[kernel]
fn do_stuff(a1: Signal<b4, Red>, a2: Signal<b4, Blue>) -> Signal<b4, Red> {
    let a1 = a1.val();
    let a2 = a2.val();
    let c = a1 + a2;
    signal(c)
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
    assert_eq!(y, b4(8));
}
