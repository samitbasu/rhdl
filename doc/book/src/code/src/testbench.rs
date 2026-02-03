#[cfg(test)]
mod circuit_testbench {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone, CircuitDQ)]
    pub struct AndGate;

    impl CircuitIO for AndGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = and_gate;
    }

    #[kernel]
    pub fn and_gate(i: Signal<(bool, bool), Red>, _q: AndGateQ) -> (Signal<bool, Red>, AndGateD) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a & b; // AND operation
        (signal(c), AndGateD {})
    }

    // ANCHOR: AND_Testbench
    #[test]
    fn test_and_gate_testbench() -> miette::Result<()> {
        // Inputs to exercise the circuit
        let inputs = [(true, false), (false, true), (true, true), (false, false)]
            .into_iter()
            .map(|(a, b)| signal((a, b)))
            .uniform(100);
        // The circuit under test
        let uut = AndGate;
        // Create the testbench
        let testbench = uut.run(inputs).collect::<TestBench<_, _>>();
        // Generate the RTL testbench module
        let test_module = testbench.rtl(&uut, &TestBenchOptions::default())?;
        // Write the testbench to a Verilog file
        std::fs::write("and_test_tb.v", test_module.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: AND_Testbench
}

#[cfg(test)]
mod counter_testbench {
    use rhdl::prelude::*;
    use std::iter::repeat_n;

    // ANCHOR: counter_testbench
    #[test]
    fn test_counter_testbench() -> miette::Result<()> {
        let input = repeat_n(true, 2)
            .chain(repeat_n(false, 2).chain(repeat_n(true, 2)))
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        let testbench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let test_module = testbench.rtl(&uut, &TestBenchOptions::default())?;
        std::fs::write("counter_test_tb.v", test_module.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: counter_testbench
}
