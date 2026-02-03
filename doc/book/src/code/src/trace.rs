#[cfg(test)]
mod tests {
    use rhdl::prelude::*;
    // ANCHOR: svg-collect
    #[test]
    fn test_trace_collect_svg() {
        // Create an input iterator that enables the counter for 10 cycles
        let enable = std::iter::repeat_n(true, 10);
        // Add the reset and clock
        let inputs = enable.with_reset(1).clock_pos_edge(100);
        // Create the 3-bit counter UUT
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        // Run the simulation and collect the SVG output using the SvgFile container
        let svg = uut.run(inputs).collect::<SvgFile>();
        // Write the SVG to a file
        svg.write_to_file("counter_collect.svg", &SvgOptions::default())
            .expect("Failed to write SVG file");
    }
    // ANCHOR_END: svg-collect

    // ANCHOR: vcd-collect
    #[test]
    fn test_trace_collect_vcd() {
        // Create an input iterator that enables the counter for 10 cycles
        let enable = std::iter::repeat_n(true, 10);
        // Add the reset and clock
        let inputs = enable.with_reset(1).clock_pos_edge(100);
        // Create the 3-bit counter UUT
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        // Run the simulation and collect the VCD output using the VcdFile container
        let vcd = uut.run(inputs).collect::<VcdFile>();
        // Write the VCD to a file
        vcd.dump_to_file("counter_collect.vcd")
            .expect("Failed to write VCD file");
    }
    // ANCHOR_END: vcd-collect

    // ANCHOR: collect_svg_trace_skip_before
    #[test]
    fn test_trace_collect_svg_skip_6() {
        // Create an input iterator that enables the counter for 18 cycles
        let enable = std::iter::repeat_n(true, 18);
        // Add the reset and clock
        let inputs = enable.with_reset(1).clock_pos_edge(100);
        // Turn off tracing for the first 6 clock cycles
        let inputs = inputs.map(|t| if t.time < 600 { t.untrace() } else { t });
        // Create the 3-bit counter UUT
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        // Run the simulation and collect the SVG output using the SvgFile container
        let svg = uut.run(inputs).collect::<SvgFile>();
        // Write the SVG to a file
        svg.write_to_file("counter_collect_skip.svg", &SvgOptions::default())
            .expect("Failed to write SVG file");
    }
    // ANCHOR_END: collect_svg_trace_skip_before

    // ANCHOR: collect_vcd_trace_skip_before
    #[test]
    fn test_trace_collect_vcd_skip_6() {
        // Create an input iterator that enables the counter for 18 cycles
        let enable = std::iter::repeat_n(true, 18);
        // Add the reset and clock
        let inputs = enable.with_reset(1).clock_pos_edge(100);
        // Turn off tracing for the first 6 clock cycles
        let inputs = inputs.map(|t| if t.time < 600 { t.untrace() } else { t });
        // Create the 3-bit counter UUT
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        // Run the simulation and collect the VCD output using the VcdFile container
        let vcd = uut.run(inputs).collect::<VcdFile>();
        // Write the VCD to a file
        vcd.dump_to_file("counter_collect_skip.vcd")
            .expect("Failed to write VCD file");
    }
    // ANCHOR_END: collect_vcd_trace_skip_before

    // ANCHOR: collect_svg_near_rollover
    #[test]
    fn test_trace_near_rollover() {
        // Create an input iterator that enables the counter for 20 cycles
        let enable = std::iter::repeat_n(true, 20);
        // Add the reset and clock
        let inputs = enable.with_reset(1).clock_pos_edge(100);
        // Create the 3-bit counter UUT
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        let svg = uut
            .run(inputs)
            .skip_while(|t| t.output.raw() < 6)
            .take(20)
            .collect::<SvgFile>();
        // Write the SVG to a file
        svg.write_to_file("counter_near_rollover.svg", &SvgOptions::default())
            .expect("Failed to write SVG file");
    }
    // ANCHOR_END: collect_svg_near_rollover
}

pub mod around_event_trait {

    pub struct AroundEvent<I, T, F>
    where
        I: Iterator<Item = T>,
        F: FnMut(&T) -> bool,
    {
        _inner: I,
        _marker: std::marker::PhantomData<(T, F)>,
    }

    // ANCHOR: around-event-trait
    pub trait AroundEventExt: Iterator {
        fn around_event<F>(
            self,
            before: usize,
            after: usize,
            predicate: F,
        ) -> AroundEvent<Self, Self::Item, F>
        where
            Self: Sized,
            F: FnMut(&Self::Item) -> bool;
    }
    // ANCHOR_END: around-event-trait
}

#[cfg(test)]
mod around_event_tests {
    use rhdl::prelude::*;

    #[test]
    fn test_around_event() {
        // ANCHOR: around-event-test
        let input = std::iter::repeat_n(true, 40)
            .with_reset(1)
            .clock_pos_edge(100);
        let uut = rhdl_fpga::core::counter::Counter::<4>::default();
        let trace = uut
            .run(input)
            .around_event(5, 5, |t| t.output.raw() == 15)
            .collect::<SvgFile>();
        trace
            .write_to_file(
                "counter_around_event.svg",
                &SvgOptions::default()
                    .with_manual_gap_detection(55)
                    .with_tail_flush_time(50),
            )
            .expect("Failed to write SVG file");
        // ANCHOR_END: around-event-test
    }
}
