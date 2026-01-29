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

#[cfg(test)]
pub mod step_2 {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn test_fixture_and() -> miette::Result<()> {
        // ANCHOR: fixture-with-inputs
        // Construct the circuit instance
        let uut = ANDGate;
        // Create a new fixture
        let mut fixture = Fixture::new("and_fixture", uut);
        // Get an instance of the input type
        let input = <ANDGate as CircuitIO>::I::dont_care();
        // Pass through the inputs                👇 Extract path
        fixture.pass_through_input("a_in", &path!(input.val().0))?;
        fixture.pass_through_input("b_in", &path!(input.val().1))?;
        // Compile fixture into module (Verilog)
        let module = fixture.module()?;
        // Write to file
        std::fs::write("and_fixture_step_2.v", format!("{}", module)).into_diagnostic()?;
        // ANCHOR_END: fixture-with-inputs
        Ok(())
    }
}

#[cfg(test)]
pub mod step_3 {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn test_fixture_and() -> miette::Result<()> {
        // ANCHOR: fixture-with-io
        let uut = ANDGate;
        let mut fixture = Fixture::new("and_fixture", uut);
        let input = <ANDGate as CircuitIO>::I::dont_care();
        // New! Get an instance of the output type 👇
        let output = <ANDGate as CircuitIO>::O::dont_care();
        fixture.pass_through_input("a_in", &path!(input.val().0))?;
        fixture.pass_through_input("b_in", &path!(input.val().1))?;
        // New! Pass through the outputs           👇 Extract path
        fixture.pass_through_output("out", &path!(output.val()))?;
        let module = fixture.module()?;
        std::fs::write("and_fixture_step_3.v", format!("{}", module)).into_diagnostic()?;
        // ANCHOR_END: fixture-with-io
        Ok(())
    }
}

#[cfg(test)]
pub mod step_4 {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn test_fixture_and() -> miette::Result<()> {
        // ANCHOR: fixture-with-bind
        let uut = ANDGate;
        let mut fixture = Fixture::new("and_fixture", uut);
        let input = <ANDGate as CircuitIO>::I::dont_care();
        let output = <ANDGate as CircuitIO>::O::dont_care();
        // New!            👇 snazzy arrow syntax
        bind!(fixture, a_in -> input.val().0);
        bind!(fixture, b_in -> input.val().1);
        bind!(fixture, out <- output.val());
        let module = fixture.module()?;
        std::fs::write("and_fixture_step_4.v", format!("{}", module)).into_diagnostic()?;
        // ANCHOR_END: fixture-with-bind
        Ok(())
    }
}

#[cfg(test)]
pub mod step_5 {
    use super::*;
    use miette::IntoDiagnostic;

    #[test]
    fn test_fixture_and() -> miette::Result<()> {
        // ANCHOR: fixture-with-constant
        let uut = ANDGate;
        let mut fixture = Fixture::new("and_fixture", uut);
        let input = <ANDGate as CircuitIO>::I::dont_care();
        let output = <ANDGate as CircuitIO>::O::dont_care();
        bind!(fixture, a_in -> input.val().0);
        // New! Constant input 👇
        fixture.constant_input(true, &path!(input.val().1))?;
        bind!(fixture, out <- output.val());
        let module = fixture.module()?;
        std::fs::write("and_fixture_step_5.v", format!("{}", module)).into_diagnostic()?;
        // ANCHOR_END: fixture-with-constant
        Ok(())
    }
}

#[cfg(test)]
pub mod blinky_xem7010 {
    // Simple LED blinker for an XEM7010....

    // The blinker itself is a simple synchronous counter
    // with a bit selecting output.
    use miette::IntoDiagnostic;
    use rhdl::prelude::*;

    mod blinker {
        use super::*;

        // ANCHOR: blinky-U
        #[derive(Clone, Synchronous, SynchronousDQ, Default)]
        #[rhdl(dq_no_prefix)]
        pub struct U {
            // We need a 32 bit counter.
            counter: rhdl_fpga::core::counter::Counter<32>,
        }
        // ANCHOR_END: blinky-U

        // ANCHOR: blinky-io
        impl SynchronousIO for U {
            type I = ();
            type O = b8; // Needed to drive all 8 LEDs
            type Kernel = blinker;
        }
        // ANCHOR_END: blinky-io

        // ANCHOR: blinky-kernel
        #[kernel]
        pub fn blinker(_cr: ClockReset, _i: (), q: Q) -> (b8, D) {
            let mut d = D::dont_care();
            // The counter is always enabled.
            d.counter = true;
            let output_bit = (q.counter >> 28) & 1 != 0;
            let o = if output_bit { bits(0xaa) } else { bits(0x55) };
            (o, d)
        }
        // ANCHOR_END: blinky-kernel
    }

    #[test]
    fn test_blinker_fixture() -> miette::Result<()> {
        // ANCHOR: blinker-adapter
        type T = Adapter<blinker::U, Red>;
        let blinker: T = Adapter::new(blinker::U::default());
        // ANCHOR_END: blinker-adapter
        // ANCHOR: blinker-fixture-start
        let mut fixture = Fixture::new("top", blinker);
        // ANCHOR_END: blinker-fixture-start
        // ANCHOR: blinker-fixture-io
        let (i, o) = fixture.io_dont_care();
        // ANCHOR_END: blinker-fixture-io
        // ANCHOR: blinker-fixture-drivers
        fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::sys_clock::sys_clock(
            &path!(i.clock_reset.val().clock),
        )?);
        fixture.constant_input(reset(false), &path!(i.clock_reset.val().reset))?;
        fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::leds::leds(&path!(o.val()))?);
        // ANCHOR_END: blinker-fixture-drivers
        let _module = fixture.module()?;
        Ok(())
    }

    #[test]
    fn test_blinker_fixture_total() -> miette::Result<()> {
        // ANCHOR: blinker-fixture
        type T = Adapter<blinker::U, Red>;
        let blinker: T = Adapter::new(blinker::U::default());
        let mut fixture = Fixture::new("top", blinker);
        let (i, o) = fixture.io_dont_care();
        fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::sys_clock::sys_clock(
            &path!(i.clock_reset.val().clock),
        )?);
        fixture.constant_input(reset(false), &path!(i.clock_reset.val().reset))?;
        fixture.add_driver(rhdl_bsp::ok::drivers::xem7010::leds::leds(&path!(o.val()))?);
        let module = fixture.module()?;
        let constraints = fixture.constraints();
        // ANCHOR_END: blinker-fixture
        std::fs::write("blinky_fixture.v", format!("{}", module)).into_diagnostic()?;
        std::fs::write("blinky_constraints.xdc", constraints).into_diagnostic()?;
        Ok(())
    }
}
