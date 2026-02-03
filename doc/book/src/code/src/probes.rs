pub mod edge_trait {
    use rhdl::{core::trace::trace_sample::TracedSample, prelude::*};

    pub struct EdgeTime<T, I, F> {
        _inner: I,
        _data_fn: F,
        _phantom: std::marker::PhantomData<T>,
    }

    // ANCHOR: edge-time-trait

    pub trait ProbeExt<I, S, U> {
        // snip
        fn edge_time<F, T>(self, data_fn: F) -> EdgeTime<T, I, F>
        where
            Self: Sized,
            I: Iterator,
            F: Fn(&TracedSample<S, U>) -> T,
            T: Digital;
        // snip
    }

    // ANCHOR_END: edge-time-trait
}
#[cfg(test)]
pub mod edge_time {
    use rhdl::prelude::*;
    use std::io::Write;

    #[test]
    fn test_edge_time_probe() {
        // Create a stream of input values.
        let input_values = [0, 0, 5, 5, 3, 0];
        let input_stream = input_values.into_iter().map(b8);
        let timed_samples = input_stream.without_reset().clock_pos_edge(100);
        let outputs = timed_samples.map(|t| TracedSample {
            time: t.time,
            input: t.value,
            output: t.value.1 * b8(2),
            page: None,
        });
        let mut file = std::fs::File::create("edge_time_input.txt").unwrap();
        writeln!(file, "| Time | Clock Reset | Input | Output |").unwrap();
        writeln!(file, "|------|-------------|-------|--------|").unwrap();
        for item in outputs.clone() {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                item.time, item.input.0, item.input.1, item.output
            )
            .unwrap();
        }
        let edge_times = outputs
            .edge_time(|sample| sample.output)
            .collect::<Vec<_>>();
        let mut edge_file = std::fs::File::create("edge_time_output.txt").unwrap();
        writeln!(edge_file, "| Time | Clock Reset | Input | Output |").unwrap();
        writeln!(edge_file, "|------|-------------|-------|--------|").unwrap();
        for item in edge_times {
            writeln!(
                edge_file,
                "| {} | {} | {} | {} |",
                item.time, item.input.0, item.input.1, item.output
            )
            .unwrap();
        }
    }
}

pub mod glitch_check {
    use rhdl::prelude::*;

    pub struct GlitchCheck<T, I, F> {
        _inner: I,
        _clock_fn: F,
        _phantom: std::marker::PhantomData<T>,
    }

    // ANCHOR: glitch-trait
    pub trait ProbeExt<I, S, U> {
        // snip
        fn glitch_check<F, T>(self, clock_fn: F) -> GlitchCheck<T, I, F>
        where
            Self: Sized,
            I: Iterator,
            F: Fn(&TracedSample<S, U>) -> (Clock, T),
            S: Digital,
            U: Digital,
            T: Digital;
        //snip
    }
    // ANCHOR_END: glitch-trait
}

#[cfg(test)]
pub mod glitch_check_test {
    use rhdl::prelude::*;
    use std::io::Write;

    fn glitch_check_data() -> Vec<TracedSample<(ClockReset, b8), b8>> {
        // Create a stream of input values.
        let input_values = [0, 0, 5, 5, 3, 0];
        let input_stream = input_values.into_iter().map(b8);
        let timed_samples = input_stream.without_reset().clock_pos_edge(100);
        let outputs = timed_samples.map(|t| TracedSample {
            time: t.time,
            input: t.value,
            output: t.value.1 * b8(2),
            page: None,
        });
        let outputs_front = outputs.clone().take(3);
        let outputs_back = outputs.skip(3).take(5);
        outputs_front
            .chain(std::iter::once(TracedSample {
                time: 55,
                input: (clock_reset(clock(true), reset(false)), b8(3)),
                output: b8(6),
                page: None,
            }))
            .chain(outputs_back)
            .collect::<Vec<_>>()
    }

    #[test]
    fn test_generate_glitch_check_input_file() {
        let mut file = std::fs::File::create("glitch_check_input.txt").unwrap();
        let outputs = glitch_check_data();
        writeln!(file, "| Time | Clock Reset | Input | Output |").unwrap();
        writeln!(file, "|------|-------------|-------|--------|").unwrap();
        for item in &outputs {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                item.time, item.input.0, item.input.1, item.output
            )
            .unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn test_glitch_check_probe() {
        let outputs = glitch_check_data();
        // ANCHOR: glitch-check-demo
        outputs
            .into_iter()
            .glitch_check(|sample| (sample.input.0.clock, sample.output))
            .for_each(drop);
        // ANCHOR_END: glitch-check-demo
    }
}

pub mod sample_at_neg_edge_trait {
    use rhdl::prelude::*;

    pub struct SampleAtNegEdge<I, F> {
        _inner: I,
        _clock_fn: F,
    }

    // ANCHOR: sample-at-neg-edge-trait
    pub trait ProbeExt<I, S, U> {
        // snip
        fn sample_at_neg_edge<F>(self, clock_fn: F) -> SampleAtNegEdge<I, F>
        where
            Self: Sized,
            I: Iterator,
            F: Fn(&I::Item) -> Clock;
        // snip
    }
    // ANCHOR_END: sample-at-neg-edge-trait
}

#[cfg(test)]
pub mod sample_at_neg_edge_test {
    use rhdl::prelude::*;
    use std::io::Write;

    #[test]
    fn test_sample_at_neg_edge_probe() {
        // Create a stream of input values.
        let input_values = [5, 4, 7, 9, 2].into_iter().map(b8);
        let inputs = input_values.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let svg = dff.run(inputs).collect::<SvgFile>();
        std::fs::write(
            "dff_sample_at_neg_edge.svg",
            svg.to_string(&SvgOptions::default()).unwrap(),
        )
        .unwrap();
        // ANCHOR: sample-at-neg-edge-demo
        let input_values = [5, 4, 7, 9, 2].into_iter().map(b8);
        let inputs = input_values.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let outputs = dff
            .run(inputs)
            .sample_at_neg_edge(|x| x.input.0.clock)
            .collect::<Vec<_>>();
        // ANCHOR_END: sample-at-neg-edge-demo
        let mut file = std::fs::File::create("dff_sample_at_neg_edge.txt").unwrap();
        writeln!(file, "| Time | ClockReset | Input | Output |").unwrap();
        writeln!(file, "|------|------------|-------|--------|").unwrap();
        for item in outputs {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                item.time, item.input.0, item.input.1, item.output
            )
            .unwrap();
        }
    }
}

pub mod sync_sample_trait {
    use rhdl::prelude::*;

    pub struct SynchronousSample<I> {
        _inner: I,
    }

    // ANCHOR: synchronous-sample-trait

    pub trait SynchronousProbeExt<I, P, O> {
        /// Create a probe that samples values from the supplied stream
        /// just before a positive edge of the clock
        fn synchronous_sample(self) -> SynchronousSample<I>
        where
            Self: Sized,
            I: Iterator<Item = TracedSample<(ClockReset, P), O>>,
            P: Digital,
            O: Digital;
    }

    // ANCHOR_END: synchronous-sample-trait
}

#[cfg(test)]
pub mod synchronous_sample_demo {
    use rhdl::prelude::*;
    use std::io::Write;

    #[test]
    fn test_synchronous_sample_probe() {
        // Create a stream of input values.
        let input_values = [5, 4, 7, 9, 2].into_iter().map(b8);
        let inputs = input_values.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let svg = dff.run(inputs).collect::<SvgFile>();
        std::fs::write(
            "dff_synchronous_sample.svg",
            svg.to_string(&SvgOptions::default()).unwrap(),
        )
        .unwrap();
        // ANCHOR: synchronous-sample-demo
        let input_values = [5, 4, 7, 9, 2].into_iter().map(b8);
        let inputs = input_values.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let outputs = dff.run(inputs).synchronous_sample().collect::<Vec<_>>();
        // ANCHOR_END: synchronous-sample-demo
        let mut file = std::fs::File::create("dff_synchronous_sample.txt").unwrap();
        writeln!(file, "| Time | ClockReset | Input | Output |").unwrap();
        writeln!(file, "|------|------------|-------|--------|").unwrap();
        for item in outputs {
            writeln!(
                file,
                "| {} | {} | {} | {} |",
                item.time, item.input.0, item.input.1, item.output
            )
            .unwrap();
        }
    }
}

pub mod tap_trait {
    use rhdl::prelude::*;

    pub struct VcdTap<I> {
        _inner: I,
    }

    pub struct SvgTap<I> {
        _inner: I,
    }

    // ANCHOR: tap-trait

    pub trait ProbeExt<I, S, U> {
        // snip
        fn vcd_file(self, file: impl AsRef<Path>) -> VcdTap<I>
        where
            Self: Sized,
            I: Iterator<Item = TracedSample<S, U>>,
            S: Digital,
            U: Digital;
        fn svg_file(self, file: impl AsRef<Path>, options: SvgOptions) -> SvgTap<I>
        where
            Self: Sized,
            I: Iterator<Item = TracedSample<S, U>>,
            S: Digital,
            U: Digital;
        // snip
    }

    // ANCHOR_END: tap-trait
}

#[cfg(test)]
mod tap_tests {
    use rhdl::prelude::*;

    // ANCHOR: no-trace-tap
    #[test]
    fn test_no_taps() {
        let data = [3, 4, 7, 1, 2];
        let input = data.into_iter().map(b8);
        let timed = input.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let expected = [0, 0, 3, 4, 7, 1];
        let outputs = dff
            .run(timed)
            .synchronous_sample()
            .map(|t| t.output.raw())
            .collect::<Vec<_>>();
        assert_eq!(outputs, expected);
    }
    // ANCHOR_END: no-trace-tap

    // ANCHOR: with-svg-tap
    #[test]
    fn test_with_svg_tap() {
        let data = [3, 4, 7, 1, 2];
        let input = data.into_iter().map(b8);
        let timed = input.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let expected = [0, 0, 3, 4, 7, 1];
        let outputs = dff
            .run(timed)
            // 👇 New!  Put it _before_ the .synchronous_sample() call
            .svg_file("svg_tap_demo.svg", SvgOptions::default())
            .synchronous_sample()
            .map(|t| t.output.raw())
            .collect::<Vec<_>>();
        assert_eq!(outputs, expected);
    }
    // ANCHOR_END: with-svg-tap

    // ANCHOR: with-vcd-tap
    #[test]
    fn test_with_vcd_tap() {
        let data = [3, 4, 7, 1, 2];
        let input = data.into_iter().map(b8);
        let timed = input.with_reset(1).clock_pos_edge(100);
        let dff = rhdl_fpga::core::dff::DFF::<b8>::default();
        let expected = [0, 0, 3, 4, 7, 1];
        let outputs = dff
            .run(timed)
            // 👇 Changed!
            .vcd_file("vcd_tap_demo.vcd")
            .synchronous_sample()
            .map(|t| t.output.raw())
            .collect::<Vec<_>>();
        assert_eq!(outputs, expected);
    }
    // ANCHOR_END: with-vcd-tap
}
