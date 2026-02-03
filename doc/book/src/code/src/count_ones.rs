pub mod step_1 {
    #![allow(unused_variables)]
    #![allow(non_camel_case_types)]
    // ANCHOR: ones-step-1
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    pub struct OneCounter {} // 👈 No internal components

    impl CircuitIO for OneCounter {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter;
    }

    // 👇 manual definition of D and Q, both empty
    impl CircuitDQ for OneCounter {
        type D = ();
        type Q = ();
    }
    // ANCHOR_END: ones-step-1

    // Off-camera kernel to silence the error
    #[kernel]
    pub fn one_counter(_i: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        (signal(b4::ZERO), ())
    }
}

pub mod step_2 {
    #![allow(unused_variables)]
    use rhdl::prelude::*;
    // ANCHOR: ones-step-2
    pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        todo!()
    }
    // ANCHOR_END: ones-step-2
}

pub mod step_3 {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    pub struct OneCounter {}

    impl CircuitIO for OneCounter {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter;
    }

    impl CircuitDQ for OneCounter {
        type D = ();
        type Q = ();
    }

    // ANCHOR: ones-step-3
    #[kernel]
    pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        let mut count = b4(0);
        let input = input.val();
        for i in 0..8 {
            //      👇 Test that i-th bit is set
            if input & (1 << i) != 0 {
                count += 1;
            }
        }
        (signal(count), ())
    }
    // ANCHOR_END: ones-step-3
}

pub mod step_4 {
    #[cfg(test)]
    use miette::IntoDiagnostic;
    // ANCHOR: ones-step-4
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    pub struct OneCounter {}

    impl CircuitIO for OneCounter {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter;
    }

    impl CircuitDQ for OneCounter {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        let mut count = b4(0);
        let input = input.val();
        for i in 0..8 {
            //      👇 Test that i-th bit is set
            if input & (1 << i) != 0 {
                count += 1;
            }
        }
        (signal(count), ())
    }
    // ANCHOR_END: ones-step-4

    // ANCHOR: ones-step-5
    #[test]
    fn test_ones_counter() -> miette::Result<()> {
        let inputs = (0..256).map(b8).map(signal).uniform(100);
        let uut = OneCounter {};
        uut.run(inputs).for_each(|s| {
            let input = s.input.val();
            //      Gets the `u128` under a Bits<N> 👇
            let output_count = s.output.val().raw();
            //         Standard Rust built-in method 👇
            let count_expected = input.raw().count_ones() as u128;
            assert_eq!(output_count, count_expected);
        });
        Ok(())
    }
    // ANCHOR_END: ones-step-5

    // ANCHOR: ones-step-6
    #[test]
    fn test_rtl_testbench() -> miette::Result<()> {
        let inputs = (0..256).map(b8).map(signal).uniform(100);
        let uut = OneCounter {};
        let tb: TestBench<_, _> = uut.run(inputs).collect();
        let tb = tb.rtl(&uut, &TestBenchOptions::default())?;
        std::fs::write("ones_rtl_tb.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: ones-step-6

    // ANCHOR: ones-step-7
    #[test]
    fn test_ntl_testbench() -> miette::Result<()> {
        let inputs = (0..256).map(b8).map(signal).uniform(100);
        let uut = OneCounter {};
        let tb: TestBench<_, _> = uut.run(inputs).collect();
        let tb = tb.ntl(&uut, &TestBenchOptions::default())?;
        std::fs::write("ones_ntl_tb.v", tb.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: ones-step-7

    // ANCHOR: ones-step-8
    #[test]
    fn test_svg() -> miette::Result<()> {
        let inputs = (0..256).map(b8).cycle().take(257).map(signal).uniform(100);
        let uut = OneCounter {};
        let svg: SvgFile = uut.run(inputs).skip_while(|t| t.time < 25000).collect();
        svg.write_to_file("ones.svg", &SvgOptions::default())
            .into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: ones-step-8

    // ANCHOR: ones-step-9
    #[test]
    fn test_vcd() -> miette::Result<()> {
        let inputs = (0..256).map(b8).cycle().take(257).map(signal).uniform(100);
        let uut = OneCounter {};
        let vcd: VcdFile = uut.run(inputs).skip_while(|t| t.time < 25000).collect();
        vcd.write_to_file("ones.vcd", &VcdOptions::default())
            .into_diagnostic()?;
        Ok(())
    }
    // ANCHOR_END: ones-step-9

    // ANCHOR: ones-step-10
    #[test]
    fn test_make_fixture() -> miette::Result<()> {
        let mut fixture = Fixture::new("ones_top", OneCounter {});
        let (input, output) = fixture.io_dont_care();
        bind!(fixture, dips -> input.val());
        bind!(fixture, leds <- output.val());
        let vlog = fixture.module()?;
        std::fs::write("ones_fixture.v", vlog.to_string()).unwrap();
        Ok(())
    }
    // ANCHOR_END: ones-step-10

    // ANCHOR: ones-step-11
    #[test]
    #[ignore]
    fn test_build_flash() -> miette::Result<()> {
        const PCF: &str = "
set_io dips[0] H11
set_io dips[1] G11
set_io dips[2] F11
set_io dips[3] E11
set_io dips[4] D11
set_io dips[5] D10
set_io dips[6] G1
set_io dips[7] D9
set_io leds[0] E12
set_io leds[1] D14
set_io leds[2] F12
set_io leds[3] E14
    ";
        let mut fixture = Fixture::new("ones_top", OneCounter {});
        let (input, output) = fixture.io_dont_care();
        bind!(fixture, dips -> input.val());
        bind!(fixture, leds <- output.val());
        rhdl_toolchains::icestorm::IceStorm::new("hx8k", "cb132", "/tmp/ice-ones-step-11/build")
            .clean()?
            .build_and_flash(fixture, PCF)
    }
    // ANCHOR_END: ones-step-11

    // ANCHOR: ones-step-12
    #[test]
    fn test_base_timing() -> miette::Result<()> {
        let uut = OneCounter {};
        let timing = rhdl_toolchains::icestorm::IceStorm::new(
            "hx8k",
            "cb132",
            "/tmp/ice-ones-step-12/build",
        )
        .time(uut)?;
        eprintln!("Timing: {timing:?}");
        eprintln!(
            "Total delay: {} nsec",
            timing.logic_delay + timing.routing_delay
        );
        Ok(())
    }
    // ANCHOR_END: ones-step-12
}

pub mod step_5 {
    // ANCHOR: ones-step-13
    use rhdl::prelude::*;

    #[kernel]
    pub fn count_ones<const N: usize, const M: usize>(x: Bits<N>) -> Bits<M>
    where
        rhdl::bits::W<N>: BitWidth,
        rhdl::bits::W<M>: BitWidth,
    {
        let mut count = bits(0);
        for i in 0..N {
            if x & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }
    // ANCHOR_END: ones-step-13

    // ANCHOR: ones-step-14
    #[test]
    fn test_count_ones() {
        assert_eq!(count_ones::<8, 4>(b8(0b1011_0010)), b4(4))
    }
    // ANCHOR_END: ones-step-14
}

pub mod step_6 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn count_ones<const N: usize, const M: usize>(x: Bits<N>) -> Bits<M>
    where
        rhdl::bits::W<N>: BitWidth,
        rhdl::bits::W<M>: BitWidth,
    {
        let mut count = bits(0);
        for i in 0..N {
            if x & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    // ANCHOR: ones-step-15
    #[derive(Circuit, Clone)]
    pub struct OneCounter {}

    impl CircuitIO for OneCounter {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter;
    }

    impl CircuitDQ for OneCounter {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn one_counter(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        let input = input.val();
        let count = count_ones::<8, 4>(input);
        (signal(count), ())
    }
    // ANCHOR_END: ones-step-15

    // ANCHOR: ones-step-16
    #[derive(Circuit, Clone)]
    pub struct OneCounterDivided {}

    impl CircuitIO for OneCounterDivided {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter_divided;
    }

    impl CircuitDQ for OneCounterDivided {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn one_counter_divided(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        let input = input.val();
        let low_nibble = input.resize::<4>();
        let high_nibble = (input >> 4).resize::<4>();
        let count = count_ones::<4, 4>(low_nibble) + count_ones::<4, 4>(high_nibble);
        (signal(count), ())
    }
    // ANCHOR_END: ones-step-16

    // ANCHOR: ones-step-17
    #[test]
    fn test_ones_counter_divided() -> miette::Result<()> {
        let inputs = (0..256).map(b8).map(signal).uniform(100);
        let uut = OneCounterDivided {};
        uut.run(inputs).for_each(|s| {
            let input = s.input.val();
            let output_count = s.output.val().raw();
            let count_expected = input.raw().count_ones() as u128;
            assert_eq!(output_count, count_expected);
        });
        Ok(())
    }
    // ANCHOR_END: ones-step-17

    // ANCHOR: ones-step-18
    #[test]
    fn test_timing_divided() -> miette::Result<()> {
        let uut = OneCounterDivided {};
        let timing = rhdl_toolchains::icestorm::IceStorm::new(
            "hx8k",
            "cb132",
            "/tmp/ice-ones-step-18/build",
        )
        .time(uut)?;
        eprintln!("Timing: {timing:?}");
        eprintln!(
            "Total delay: {} nsec",
            timing.logic_delay + timing.routing_delay
        );
        Ok(())
    }
    // ANCHOR_END: ones-step-18
}

pub mod step_7 {
    use rhdl::prelude::*;

    #[kernel]
    pub fn count_ones<const N: usize, const M: usize>(x: Bits<N>) -> Bits<M>
    where
        rhdl::bits::W<N>: BitWidth,
        rhdl::bits::W<M>: BitWidth,
    {
        let mut count = bits(0);
        for i in 0..N {
            if x & (1 << i) != 0 {
                count += 1;
            }
        }
        count
    }

    #[derive(Circuit, Clone)]
    pub struct OneCounterDividedFour {}

    impl CircuitIO for OneCounterDividedFour {
        type I = Signal<b8, Red>;
        type O = Signal<b4, Red>;
        type Kernel = one_counter_divided_four;
    }

    impl CircuitDQ for OneCounterDividedFour {
        type D = ();
        type Q = ();
    }

    // ANCHOR: ones-step-19
    #[kernel]
    pub fn one_counter_divided_four(input: Signal<b8, Red>, _q: ()) -> (Signal<b4, Red>, ()) {
        let input = input.val();
        let p1 = input.resize::<2>();
        let p2 = (input >> 2).resize::<2>();
        let p3 = (input >> 4).resize::<2>();
        let p4 = (input >> 6).resize::<2>();
        let count = count_ones::<2, 4>(p1)
            + count_ones::<2, 4>(p2)
            + count_ones::<2, 4>(p3)
            + count_ones::<2, 4>(p4);
        (signal(count), ())
    }
    // ANCHOR_END: ones-step-19

    // ANCHOR: ones-step-20
    #[test]
    fn test_timing_divided_four() -> miette::Result<()> {
        let uut = OneCounterDividedFour {};
        let timing = rhdl_toolchains::icestorm::IceStorm::new(
            "hx8k",
            "cb132",
            "/tmp/ice-ones-step-20/build",
        )
        .time(uut)?;
        eprintln!("Timing: {timing:?}");
        eprintln!(
            "Total delay: {} nsec",
            timing.logic_delay + timing.routing_delay
        );
        Ok(())
    }
    // ANCHOR_END: ones-step-20
}
