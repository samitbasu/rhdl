use rhdl::prelude::*;

// ANCHOR: AND-gate
#[derive(Clone, Circuit, CircuitDQ)]
pub struct ANDGate;

impl CircuitIO for ANDGate {
    type I = Signal<(bool, bool), Red>;
    type O = Signal<bool, Red>;
    type Kernel = and_kernel;
}
// ANCHOR_END: AND-gate

#[kernel]
pub fn and_kernel(i: Signal<(bool, bool), Red>, _q: ANDGateQ) -> (Signal<bool, Red>, ANDGateD) {
    let (a, b) = i.val(); // a and b are both bool
    let c = a & b; // AND operation
    (signal(c), ANDGateD {})
}

#[cfg(test)]
pub mod fixture {
    #![allow(dead_code)]
    use rhdl::prelude::*;

    // ANCHOR: fixture-def
    pub struct Fixture<T> {
        name: String,
        drivers: Vec<Driver<T>>,
        circuit: T,
    }
    // ANCHOR_END: fixture-def
    pub trait FixtureTrait<T> {
        // ANCHOR: impl-fixture
        fn new(name: &str, t: T) -> Self;
        fn add_driver(&mut self, driver: Driver<T>);
        fn pass_through_input(&mut self, name: &str, path: &Path) -> Result<(), RHDLError>;
        fn pass_through_output(&mut self, name: &str, path: &Path) -> Result<(), RHDLError>;
        fn constant_input<S: Digital>(&mut self, val: S, path: &Path) -> Result<(), RHDLError>;
        fn module(&self) -> Result<vlog::ModuleList, RHDLError>;
        fn constraints(&self) -> String;
        // ANCHOR_END: impl-fixture
    }
}

#[cfg(test)]
pub mod fixture_new {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    #[ignore]
    fn test_fixture_and() -> miette::Result<()> {
        // ANCHOR: empty-fixture
        let uut = ANDGate;
        let fixture = Fixture::new("and_fixture", uut);
        let module = fixture.module()?;
        // ANCHOR_END: empty-fixture
        std::fs::write("and_fixture_empty.v", format!("{}", module)).into_diagnostic()?;
        Ok(())
    }
}

pub mod step_2 {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn test_fixture_and() -> miette::Result<()> {
        let uut = ANDGate;
        let mut fixture = Fixture::new("and_fixture", uut);
        let input = <ANDGate as CircuitIO>::I::dont_care();
        fixture.pass_through_input("a_in", &path!(input.val().0))?;
        fixture.pass_through_input("b_in", &path!(input.val().1))?;
        let module = fixture.module()?;
        std::fs::write("and_fixture_step_2.v", format!("{}", module)).into_diagnostic()?;
        Ok(())
    }
}
