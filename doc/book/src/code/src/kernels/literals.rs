use rhdl::prelude::*;
pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[kernel]
    fn kernel(a: b8) -> b8 {
        let c1 = b8(0xbe); // hexadecimal constant
        let c2 = b8(0b1101_0110); // binary constant
        let c3 = b8(0o03_42); // octal constant
        let c4 = b8(135); // decimal constant
        a + c1 - c2 + c3 + c4
    }
    // ANCHOR_END: step_1
}

pub mod step_2 {
    use super::*;
    // ANCHOR: step_2
    #[kernel]
    fn kernel(a: b8) -> b8 {
        a + 42 // 👈 inferred as a 42 bit constant
    }
    // ANCHOR_END: step_2
}

pub mod step_3 {
    use super::*;
    // ANCHOR: step_3
    #[kernel]
    fn kernel(a: b8) -> b8 {
        a + 270 // 👈 panics at runtime or fails at RHDL compile time
    }
    // ANCHOR_END: step_3

    #[ignore]
    // ANCHOR: step_3_runtime
    #[test]
    fn test_panic_at_runtime() {
        let _ = kernel(b8(10));
    }
    // ANCHOR_END: step_3_runtime

    #[ignore]
    // ANCHOR: step_3_compile
    #[test]
    fn test_fail_at_rhdl_compile() -> miette::Result<()> {
        let _ = compile_design::<kernel>(CompilationMode::Asynchronous)?;
        Ok(())
    }
    // ANCHOR_END: step_3_compile
}

#[allow(clippy::nonminimal_bool)]
pub mod step_4 {
    use super::*;
    // ANCHOR: step_4
    #[kernel]
    fn kernel(a: bool) -> bool {
        (a ^ true) || false
    }
    // ANCHOR_END: step_4
}
