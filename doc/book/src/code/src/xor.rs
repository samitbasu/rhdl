pub mod step_1 {
    #![allow(unused_imports)]

    // ANCHOR: xor-step-1
    use rhdl::prelude::*;

    pub struct XorGate;
    // ANCHOR_END: xor-step-1
}

pub mod step_2 {
    #![allow(unused_imports)]

    // ANCHOR: xor-step-2
    use rhdl::prelude::*;

    pub struct XorGate;

    impl CircuitDQ for XorGate {
        type D = ();
        type Q = ();
    }
    // ANCHOR_END: xor-step-2
}

pub mod step_3 {
    #![allow(non_camel_case_types)]

    // ANCHOR: xor-step-3
    use rhdl::prelude::*;

    pub struct XorGate;

    impl CircuitDQ for XorGate {
        type D = ();
        type Q = ();
    }

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate; // 👈 doesn't exist yet
    }
    // ANCHOR_END: xor-step-3

    type xor_gate = NoCircuitKernel<Signal<(bool, bool), Red>, (), (Signal<bool, Red>, ())>;
}

pub mod step_4 {
    #![allow(non_camel_case_types)]

    // ANCHOR: xor-step-4
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)] // 👈 new!
    pub struct XorGate;

    impl CircuitDQ for XorGate {
        type D = ();
        type Q = ();
    }

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate; // 👈 doesn't exist yet
    }
    // ANCHOR_END: xor-step-4

    type xor_gate = NoCircuitKernel<Signal<(bool, bool), Red>, (), (Signal<bool, Red>, ())>;
}

pub mod step_5 {
    #![allow(unused_variables)]
    use rhdl::prelude::*;

    // ANCHOR: xor-step-5
    /*
    👇 needed! */
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        todo!()
    }
    // ANCHOR_END: xor-step-5
}

pub mod step_6 {
    #![allow(unused_variables)]
    use rhdl::prelude::*;

    // ANCHOR: xor-step-6
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val(); // 👈 a and b are both bool
        let c = a ^ b; // 👈 Exclusive OR
        (signal(c), ()) // 👈 (Output, D)
    }
    // ANCHOR_END: xor-step-6
}

pub mod step_7 {
    #![allow(unused_variables)]
    use rhdl::prelude::*;

    // ANCHOR: xor-step-7
    #[kernel] // 👈 new!
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val();
        let c = a ^ b;
        (signal(c), ())
    }
    // ANCHOR_END: xor-step-7

    // ANCHOR: xor-step-8
    #[test]
    fn test_xor_gate() {
        let (out, _) = xor_gate(signal((true, false)), ());
        assert!(out.val());
        let (out, _) = xor_gate(signal((true, true)), ());
        assert!(!out.val());
        let (out, _) = xor_gate(signal((false, false)), ());
        assert!(!out.val());
        let (out, _) = xor_gate(signal((false, true)), ());
        assert!(out.val());
    }
    // ANCHOR_END: xor-step-8
}

pub mod step_8 {
    #[cfg(test)]
    use miette::IntoDiagnostic;

    // ANCHOR: xor-step-9
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    pub struct XorGate;

    impl CircuitDQ for XorGate {
        type D = ();
        type Q = ();
    }

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate;
    }

    #[kernel]
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val();
        let c = a ^ b;
        (signal(c), ())
    }
    // ANCHOR_END: xor-step-9

    // ANCHOR: xor-step-10
    #[test]
    fn test_all_inputs() {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let outputs = [false, true, true, false];
        inputs.iter().zip(outputs.iter()).for_each(|(inp, outp)| {
            let (y, _) = xor_gate(signal(*inp), ());
            assert_eq!(y.val(), *outp);
        });
    }
    // ANCHOR_END: xor-step-10

    // ANCHOR: xor-step-11
    #[test]
    fn show_verilog() -> miette::Result<()> {
        let gate = XorGate;
        let desc = gate.descriptor(ScopedName::top())?;
        let hdl = desc.hdl()?;
        let hdl = hdl.modules.pretty();
        std::fs::write("xor_gate.v", hdl).into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: xor-step-11

    // ANCHOR: xor-step-12
    #[test]
    fn test_iterators() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        //                       Separate samples by 100 units - 👇
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        for y in uut.run(it) {
            eprintln!("{}", y);
        }
        Ok(())
    }
    // ANCHOR_END: xor-step-12

    // ANCHOR: xor-step-13
    #[test]
    fn test_iterators_expected() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        //                   👇 TracedSample<Signal<(bool,bool), Red>, Signal<bool, Red>>
        uut.run(it).for_each(|s| {
            let input = s.input.val();
            let output = s.output.val();
            let expected = input.0 ^ input.1;
            assert_eq!(output, expected, "For input {input:?}, expected {expected}");
        });
        Ok(())
    }
    // ANCHOR_END: xor-step-13

    // ANCHOR: xor-step-14
    #[test]
    fn test_svg() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        let svg = uut.run(it).collect::<SvgFile>();
        svg.write_to_file("xor.svg", &SvgOptions::default().with_io_filter())
            .into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: xor-step-14

    // ANCHOR: xor-step-15
    #[test]
    fn test_vcd() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        let vcd = uut.run(it).collect::<VcdFile>();
        vcd.dump_to_file("xor.vcd").into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: xor-step-15

    // ANCHOR: xor-step-16
    #[test]
    fn test_testbench() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        let tb: TestBench<_, _> = uut.run(it).collect();
        let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
        std::fs::write("xor_tb.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: xor-step-16

    // ANCHOR: xor-step-17
    #[test]
    fn test_testbench_ntl() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().cycle().take(5).map(signal).uniform(100);
        let uut = XorGate;
        let tb: TestBench<_, _> = uut.run(it).collect();
        let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
        std::fs::write("xor_tb_ntl.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: xor-step-17

    // ANCHOR: xor-step-18
    #[test]
    fn test_make_fixture() -> miette::Result<()> {
        let mut fixture = Fixture::new("xor_top", XorGate);
        // bind! needs an input and output value to work with
        // This method provides a tuple of (input, output) with
        // dont_care values for all fields.
        let (input, output) = fixture.io_dont_care();
        // Bind an input port 'a' to input.val().0
        bind!(fixture, a -> input.val().0);
        // Bind an input port 'b' to input.val().1
        bind!(fixture, b -> input.val().1);
        // Bind an output port 'y' to output.val()
        // Note the direction of the arrow is reversed for outputs
        bind!(fixture, y <- output.val());
        let vlog = fixture.module()?;
        eprintln!("{vlog}");
        std::fs::write("xor_top.v", vlog.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: xor-step-18

    // ANCHOR: xor-step-19
    #[test]
    #[ignore]
    fn test_flash_icestorm() -> miette::Result<()> {
        const PCF: &str = "
set_io a H11
set_io b G11
set_io y E12    
    ";
        let uut = XorGate;
        let mut fixture = Fixture::new("xor_flash", uut);
        let (input, output) = fixture.io_dont_care();
        bind!(fixture, a -> input.val().0);
        bind!(fixture, b -> input.val().1);
        bind!(fixture, y <- output.val());
        rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "/tmp/ice-xor-step-19/build")
            .clean()?
            .build_and_flash(fixture, PCF)
    }
    // ANCHOR_END: xor-step-19
}
