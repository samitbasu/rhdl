pub mod step_1 {
    // ANCHOR: adder-step-1
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
    // ANCHOR_END: adder-step-1
}

pub mod step_2 {
    // ANCHOR: adder-step-2
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    pub struct AndGate;

    impl CircuitDQ for AndGate {
        type D = ();
        type Q = ();
    }

    impl CircuitIO for AndGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = and_gate;
    }

    #[kernel]
    pub fn and_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val();
        let c = a & b;
        (signal(c), ())
    }
    // ANCHOR_END: adder-step-2
}

pub mod step_3 {
    use rhdl::prelude::*;

    // ANCHOR: adder-step-3
    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct Outputs {
        pub sum: Signal<bool, Red>,
        pub carry: Signal<bool, Red>,
    }
    // ANCHOR_END: adder-step-3

    use super::step_1::XorGate;
    use super::step_2::AndGate;

    // ANCHOR: adder-step-4
    #[derive(Circuit)]
    pub struct HalfAdder {
        pub xor: XorGate,
        pub and: AndGate,
    }
    // ANCHOR_END: adder-step-4

    // ANCHOR: adder-step-5
    impl Default for HalfAdder {
        fn default() -> Self {
            Self {
                xor: XorGate,
                and: AndGate,
            }
        }
    }
    // ANCHOR_END: adder-step-5

    // ANCHOR: adder-step-6
    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct HalfAdderD {
        xor: <XorGate as CircuitIO>::I,
        and: <AndGate as CircuitIO>::I,
    }
    // ANCHOR_END: adder-step-6

    // ANCHOR: adder-step-7
    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct HalfAdderQ {
        xor: <XorGate as CircuitIO>::O,
        and: <AndGate as CircuitIO>::O,
    }
    // ANCHOR_END: adder-step-7

    // ANCHOR: adder-step-8
    impl CircuitDQ for HalfAdder {
        type D = HalfAdderD;
        type Q = HalfAdderQ;
    }
    impl CircuitIO for HalfAdder {
        type I = Signal<(bool, bool), Red>;
        type O = Outputs;
        type Kernel = half_adder_kernel; // 👈 doesn't exist yet
    }
    // ANCHOR_END: adder-step-8

    mod incomplete {
        #![allow(unused_variables)]
        #![allow(dead_code)]
        use super::*;
        // ANCHOR: adder-step-9
        //                     👇 Input type
        pub fn half_adder(i: Signal<(bool, bool), Red>, q: HalfAdderQ) -> (Outputs, HalfAdderD) {
            todo!()
        }
        // ANCHOR_END: adder-step-9
    }

    // ANCHOR: adder-step-10
    #[kernel]
    pub fn half_adder_kernel(i: Signal<(bool, bool), Red>, q: HalfAdderQ) -> (Outputs, HalfAdderD) {
        // Each gate is fed a copy of the input signal
        let d = HalfAdderD {
            xor: i,
            and: i, // 👈 Digital : Copy, so no cloning needed
        };
        // The output of those internal components is forwarded to the output
        let o = Outputs {
            sum: q.xor,
            carry: q.and,
        };
        (o, d)
    }
    // ANCHOR_END: adder-step-10
}

pub mod step_4 {
    use super::step_1::XorGate;
    use super::step_2::AndGate;

    #[cfg(test)]
    use miette::IntoDiagnostic;

    // ANCHOR: adder-step-11
    use rhdl::prelude::*;

    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct Outputs {
        pub sum: Signal<bool, Red>,
        pub carry: Signal<bool, Red>,
    }

    #[derive(Circuit)]
    pub struct HalfAdder {
        pub xor: XorGate,
        pub and: AndGate,
    }

    impl Default for HalfAdder {
        fn default() -> Self {
            Self {
                xor: XorGate,
                and: AndGate,
            }
        }
    }

    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct HalfAdderD {
        xor: <XorGate as CircuitIO>::I,
        and: <AndGate as CircuitIO>::I,
    }

    #[derive(Digital, Copy, Clone, Timed, PartialEq)]
    pub struct HalfAdderQ {
        xor: <XorGate as CircuitIO>::O,
        and: <AndGate as CircuitIO>::O,
    }

    impl CircuitDQ for HalfAdder {
        type D = HalfAdderD;
        type Q = HalfAdderQ;
    }

    impl CircuitIO for HalfAdder {
        type I = Signal<(bool, bool), Red>;
        type O = Outputs;
        type Kernel = half_adder_kernel; // 👈 doesn't exist yet
    }

    #[kernel]
    pub fn half_adder_kernel(i: Signal<(bool, bool), Red>, q: HalfAdderQ) -> (Outputs, HalfAdderD) {
        // Each gate is fed a copy of the input signal
        let d = HalfAdderD {
            xor: i,
            and: i, // 👈 Digital : Copy, so no cloning needed
        };
        // The output of those internal components is forwarded to the output
        let o = Outputs {
            sum: q.xor,
            carry: q.and,
        };
        (o, d)
    }
    // ANCHOR_END: adder-step-11

    // ANCHOR: adder-step-12
    #[test]
    fn test_half_adder() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let uut = HalfAdder::default();
        uut.run(it).for_each(|s| {
            let input = s.input.val();
            let output_sum = s.output.sum.val();
            let output_carry = s.output.carry.val();
            let sum_expected = input.0 ^ input.1;
            let carry_expected = input.0 & input.1;
            assert_eq!(output_sum, sum_expected);
            assert_eq!(output_carry, carry_expected);
        });
        Ok(())
    }
    // ANCHOR_END: adder-step-12

    // ANCHOR: adder-step-13
    #[test]
    fn test_testbench() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let uut = HalfAdder::default();
        let tb: TestBench<_, _> = uut.run(it).collect();
        let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
        std::fs::write("half_rtl_tb.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: adder-step-13

    // ANCHOR: adder-step-14
    #[test]
    fn test_testbench_ntl() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let uut = HalfAdder::default();
        let tb: TestBench<_, _> = uut.run(it).collect();
        let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
        std::fs::write("half_ntl_tb.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: adder-step-14

    // ANCHOR: adder-step-15
    #[test]
    fn test_make_svg() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let uut = HalfAdder::default();
        let svg = uut.run(it).collect::<SvgFile>();
        svg.write_to_file(
            "half_adder.svg",
            &SvgOptions::default()
                .with_io_filter()
                .with_tail_flush_time(100),
        )
        .into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: adder-step-15

    // ANCHOR: adder-step-16
    #[test]
    fn test_make_vcd() -> miette::Result<()> {
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let uut = HalfAdder::default();
        let vcd = uut.run(it).collect::<VcdFile>();
        vcd.dump_to_file("half_adder.vcd").into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: adder-step-16

    // ANCHOR: adder-step-17
    #[test]
    fn test_make_fixture() -> miette::Result<()> {
        let mut fixture = Fixture::new("half_top", HalfAdder::default());
        let (input, output) = fixture.io_dont_care();
        bind!(fixture, a -> input.val().0);
        bind!(fixture, b -> input.val().1);
        bind!(fixture, sum <- output.sum);
        bind!(fixture, carry <- output.carry);
        let vlog = fixture.module()?;
        std::fs::write("half_adder_fixture.v", vlog.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: adder-step-17

    // ANCHOR: adder-step-18
    #[test]
    #[ignore]
    fn test_build_flash() -> miette::Result<()> {
        const PCF: &str = "
set_io a H11
set_io b G11
set_io sum E12
set_io carry D14
    ";
        let mut fixture = Fixture::new("half_top", HalfAdder::default());
        let (input, output) = fixture.io_dont_care();
        bind!(fixture, a -> input.val().0);
        bind!(fixture, b -> input.val().1);
        bind!(fixture, sum <- output.sum.val());
        bind!(fixture, carry <- output.carry.val());
        rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "/tmp/ice-adder-step-18/build")
            .clean()?
            .build_and_flash(fixture, PCF)
    }
    // ANCHOR_END: adder-step-18
}
