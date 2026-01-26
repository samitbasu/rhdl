pub mod kern_1 {
    use rhdl::prelude::*;

    pub trait MeTraitAsync {
        type I: Digital;
        type O: Digital;
        type D: Digital;
        type Q: Digital;

        // ANCHOR: async_fn

        fn kernel(i: Self::I, q: Self::Q) -> (Self::O, Self::D);

        // ANCHOR_END: async_fn
    }

    pub trait MeTraitSync {
        type I: Digital;
        type O: Digital;
        type D: Digital;
        type Q: Digital;

        // ANCHOR: sync_fn

        fn kernel(cr: ClockReset, i: Self::I, q: Self::Q) -> (Self::O, Self::D);

        // ANCHOR_END: sync_fn
    }

    // ANCHOR: counter_fn

    pub fn counter(cr: ClockReset, enable: bool, q: b8) -> (b8, b8) {
        let next_count = if enable { q + bits(1) } else { q };
        let next_count = if cr.reset.any() { bits(0) } else { next_count };
        (q, next_count)
    }

    // ANCHOR_END: counter_fn

    // ANCHOR: test_counter_exhaustive

    #[test]
    fn test_counter_exhaustive() {
        let cr = clock_reset(clock(false), reset(false));
        for i in [false, true] {
            for q in (0..256).map(b8) {
                let (o, d) = counter(cr, i, q);
                if i {
                    assert_eq!(d, q + 1);
                } else {
                    assert_eq!(d, q);
                }
                assert_eq!(o, q);
            }
        }
    }

    // ANCHOR_END: test_counter_exhaustive
}

pub mod xor_gate {
    use rhdl::prelude::*;

    #[derive(Circuit, CircuitDQ)]
    pub struct XorGate;

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor;
    }

    #[kernel]
    pub fn xor(i: Signal<(bool, bool), Red>, _q: XorGateQ) -> (Signal<bool, Red>, XorGateD) {
        let (a, b) = i.val();
        (signal(a ^ b), XorGateD {})
    }

    // ANCHOR: test_iterators

    #[test]
    fn test_iterators() -> miette::Result<()> {
        let uut = XorGate;
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let it = inputs.into_iter().map(signal).uniform(100);
        let output = uut
            .run(it)
            .map(|y| format!("{:?} -> {}\n", y.input.val(), y.output.val()))
            .collect::<String>();
        std::fs::write("xor_trace.txt", output).unwrap();
        Ok(())
    }

    // ANCHOR_END: test_iterators
}

pub mod clock_pos_edge {
    use rhdl::{
        core::sim::{ResetOrData, iter::clock_pos_edge::ClockPosEdge},
        prelude::*,
    };

    /// Extension trait to provide a `clock_pos_edge` method on iterators.
    pub trait ClockPosEdgeExt<Q>: IntoIterator + Sized
    where
        Q: Digital,
    {
        /// Creates a ClockPosEdge iterator that produces clock and reset signals along with the input samples.
        fn clock_pos_edge(self, period: u64) -> ClockPosEdge<<Self as IntoIterator>::IntoIter, Q>;
    }

    // ANCHOR: clock-pos-edge-ext

    impl<I, Q> ClockPosEdgeExt<Q> for I
    where
        I: IntoIterator<Item = ResetOrData<Q>>,
        Q: Digital,
    {
        fn clock_pos_edge(self, period: u64) -> ClockPosEdge<Self::IntoIter, Q> {
            clock_pos_edge(self.into_iter(), period)
        }
    }

    // ANCHOR_END: clock-pos-edge-ext
}

pub mod clock_pos_edge_demo {
    #[test]
    fn test_clock_pos_edge_ext() {
        use rhdl::prelude::*;
        use std::io::Write;

        let samples =
// ANCHOR: cpe_demo
        (0..4).map(b8).without_reset().clock_pos_edge(10)
// ANCHOR_END: cpe_demo
        ;
        let table = std::fs::File::create("clock_pos_edge_demo.txt").unwrap();
        let mut writer = std::io::BufWriter::new(table);
        writeln!(writer, "| time | clock | reset | value |").unwrap();
        writeln!(writer, "|------|-------|-------|-------|").unwrap();
        for sample in samples {
            writeln!(
                writer,
                "| {} |   {}   |   {}   |  {:?} |",
                sample.time,
                sample.value.0.clock.raw(),
                sample.value.0.reset.raw(),
                sample.value.1
            )
            .unwrap();
        }
    }
}

pub mod ext_uniform {
    use rhdl::prelude::*;

    #[derive(Circuit, CircuitDQ, Clone)]
    pub struct Thing;

    impl CircuitIO for Thing {
        type I = Signal<b8, Red>;
        type O = Signal<b8, Red>;
        type Kernel = thing;
    }

    #[kernel]
    pub fn thing(i: Signal<b8, Red>, _q: ThingQ) -> (Signal<b8, Red>, ThingD) {
        (i, ThingD {})
    }

    #[test]
    fn test_uniform() {
        let inputs = (0..10).map(b8).map(signal);
        let uut = Thing;
        // ANCHOR: uniform-summary

        // inputs is impl Iterator<Item=Thing::I>
        // outputs is impl Iterator<Item=Thing::O>
        let outputs = uut.run(inputs.uniform(5));

        // ANCHOR_END: uniform-summary
        let _ = outputs;
    }
}

pub mod ext_synchronous {
    use rhdl::prelude::*;

    #[derive(Synchronous, SynchronousDQ, Clone)]
    pub struct Thing;

    impl SynchronousIO for Thing {
        type I = b8;
        type O = b8;
        type Kernel = thing;
    }

    #[kernel]
    pub fn thing(_cr: ClockReset, i: b8, _q: ThingQ) -> (b8, ThingD) {
        (i, ThingD {})
    }

    #[test]
    fn test_synchronous() {
        let inputs = (0..10).map(b8);
        let uut = Thing;
        // ANCHOR: synchronous-summary
        // inputs is impl Iterator<Item=Thing::I>
        // outputs is impl Iterator<Item=Thing::O>
        let outputs = uut.run(inputs.with_reset(1).clock_pos_edge(100));
        // ANCHOR_END: synchronous-summary
        let _ = outputs;
    }
}

pub mod uniform {
    #[test]
    fn test_uniform() {
        use rhdl::prelude::*;
        use std::io::Write;
        // ANCHOR: uniform-with-map
        let inputs = (0..) // Take a sequence of integers
            .map(b8) // Make them b8
            .map(signal::<_, Red>) // Into Signal<b8, Red>
            .enumerate() // Enumerate
            .map(|(ndx, s)| timed_sample(ndx as u64 * 50, s)); // Map
        // ANCHOR_END: uniform-with-map
        let values = inputs.take(5).collect::<Vec<_>>();
        let file = std::fs::File::create("uniform_map.txt").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writeln!(writer, "| time | value |").unwrap();
        writeln!(writer, "|------|-------|").unwrap();
        for sample in values {
            writeln!(writer, "| {} |  {:?} |", sample.time, sample.value.val()).unwrap();
        }
    }
}

pub mod uniform_trait {
    use rhdl::{core::sim::iter::uniform::Uniform, prelude::*};

    pub trait UniformExt<Q>: IntoIterator + Sized
    where
        Q: Digital,
    {
        /// Create a `Uniform` iterator from the current iterator and the specified period.
        fn uniform(self, period: u64) -> Uniform<Self::IntoIter, Q>;
    }

    // ANCHOR: uniform_ext

    impl<I, Q> UniformExt<Q> for I
    where
        I: IntoIterator<Item = Q>,
        Q: Digital,
    {
        fn uniform(self, period: u64) -> Uniform<Self::IntoIter, Q> {
            uniform(self.into_iter(), period)
        }
    }

    // ANCHOR_END: uniform_ext
}

pub mod uniform_with_ext {
    #[test]
    fn test_uniform() {
        use rhdl::prelude::*;
        use std::io::Write;
        // ANCHOR: uniform-without-map
        let inputs = (0..) // Take a sequence of integers
            .map(b8) // Make them b8
            .map(signal::<_, Red>) // Into Signal<b8, Red>
            .uniform(50); // Equivalent to enumerate + map
        // ANCHOR_END: uniform-without-map
        let values = inputs.take(5).collect::<Vec<_>>();
        let file = std::fs::File::create("uniform_map_ext.txt").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writeln!(writer, "| time | value |").unwrap();
        writeln!(writer, "|------|-------|").unwrap();
        for sample in values {
            writeln!(writer, "| {} |  {:?} |", sample.time, sample.value.val()).unwrap();
        }
    }
}

pub mod xor_gate_uniform {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone, CircuitDQ)]
    pub struct XorGate;

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate;
    }

    #[kernel]
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: XorGateQ) -> (Signal<bool, Red>, XorGateD) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), XorGateD {})
    }
    // ANCHOR: xor-uniform-iter
    #[test]
    fn test_iterators() -> miette::Result<()> {
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
    // ANCHOR_END: xor-uniform-iter
}

pub mod circuit_run {
    use rhdl::prelude::*;

    // ANCHOR: run-circuit

    pub trait Circuit: 'static + CircuitIO + Sized {
        type S: Clone + PartialEq;

        fn init(&self) -> Self::S;
        fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;
        // snip
    }

    // ANCHOR_END: run-circuit
}

pub mod circuit_sim {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone, CircuitDQ)]
    pub struct XorGate;

    impl CircuitIO for XorGate {
        type I = Signal<(bool, bool), Red>;
        type O = Signal<bool, Red>;
        type Kernel = xor_gate;
    }

    #[kernel]
    pub fn xor_gate(i: Signal<(bool, bool), Red>, _q: XorGateQ) -> (Signal<bool, Red>, XorGateD) {
        let (a, b) = i.val(); // a and b are both bool
        let c = a ^ b; // Exclusive OR
        (signal(c), XorGateD {})
    }

    // ANCHOR: test_sim
    #[test]
    fn test_sim() {
        let uut = XorGate; // Get an instance of the circuit
        let mut state = uut.init(); // Initialize the state
        // Assemble the inputs to test
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let inputs = inputs.into_iter().map(signal);
        for i in inputs {
            // Simulate the circuit with the input and current state
            let o = uut.sim(i, &mut state);
            // Verify the output
            let (a, b) = i.val();
            let expected = a ^ b;
            assert_eq!(o.val(), expected, "For input {i:?}, expected {expected}");
        }
    }
    // ANCHOR_END: test_sim

    // ANCHOR: test_sim_ext
    #[test]
    fn test_sim_ext() {
        let uut = XorGate;
        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        // The `.uniform` turns the values into timed samples
        let inputs = inputs.into_iter().map(signal).uniform(100);
        for output in uut.run(inputs) {
            let input = output.input.val();
            let expected = input.0 ^ input.1;
            assert_eq!(
                output.output.val(),
                expected,
                "For input {input:?}, expected {expected}"
            );
        }
    }
    // ANCHOR_END: test_sim_ext
}

pub mod synchronous_sim {
    #[test]
    fn test_counter() {
        use rhdl::core::trace::trace_sample::TracedSample;
        use rhdl::prelude::*;
        use std::io::Write;

        let fs = std::fs::File::create("counter.txt").unwrap();
        let mut writer = std::io::BufWriter::new(fs);
        writeln!(writer, "| time | cr | input | output |").unwrap();
        writeln!(writer, "|------|----|----|--------|").unwrap();
        let mut post_process = |sample: TracedSample<(ClockReset, bool), b3>| {
            writeln!(
                writer,
                "| {time} | {cr} | {input} | {output} |",
                time = sample.time,
                cr = sample.input.0,
                input = sample.input.1,
                output = sample.output
            )
            .unwrap();
        };
        // ANCHOR: counter-iter-ext
        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        let inputs = std::iter::repeat_n(true, 4)
            .with_reset(1)
            .clock_pos_edge(100);
        for sample in uut.run(inputs) {
            post_process(sample);
        }
        // ANCHOR_END: counter-iter-ext

        let uut = rhdl_fpga::core::counter::Counter::<3>::default();
        let inputs = std::iter::repeat_n(true, 6)
            .with_reset(1)
            .clock_pos_edge(100);
        // ANCHOR: counter-iter-svg

        let svg = uut.run(inputs).collect::<SvgFile>();

        // ANCHOR_END: counter-iter-svg
        std::fs::write(
            "counter_iter.svg",
            svg.to_string(&SvgOptions::default()).unwrap(),
        )
        .unwrap();
    }
}

pub mod run_synch_ext {
    use rhdl::{core::trace::trace_sample::TracedSample, prelude::*};

    pub struct RunSynchronous<'a, T, I, S> {
        pub uut: &'a T,
        pub inputs: I,
        pub state: Option<S>,
        pub time: u64,
        pub session: Session,
    }

    impl<T, I, S> Iterator for RunSynchronous<'_, T, I, S>
    where
        T: Synchronous<S = S>,
        I: Iterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
    {
        type Item = TracedSample<(ClockReset, <T as SynchronousIO>::I), <T as SynchronousIO>::O>;

        fn next(&mut self) -> Option<Self::Item> {
            todo!();
        }
    }

    // ANCHOR: run_synchronous_ext

    /// Extension trait to provide a `run` method on synchronous circuits.
    pub trait RunSynchronousExt<I>: Synchronous + Sized {
        /// Runs the circuit with the given iterator of timed inputs.
        fn run(
            &self,
            iter: I,
        ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>
        where
            I: IntoIterator;
    }

    // ANCHOR_END: run_synchronous_ext

    // ANCHOR: run_synchronous_ext_impl

    impl<T, I> RunSynchronousExt<I> for T
    where
        T: Synchronous,
        I: IntoIterator<Item = TimedSample<(ClockReset, <T as SynchronousIO>::I)>>,
    {
        fn run(
            &self,
            _iter: I,
        ) -> RunSynchronous<'_, Self, <I as IntoIterator>::IntoIter, <Self as Synchronous>::S>
        {
            todo!()
        }
    }

    // ANCHOR_END: run_synchronous_ext_impl
}

#[cfg(test)]
pub mod reset_or_data {
    use std::iter::{once, repeat_n};

    // ANCHOR: reset-or-data
    #[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
    pub enum ResetOrData<T> {
        Reset,
        Data(T),
    }
    // ANCHOR_END: reset-or-data

    #[test]
    fn test_iter() {
        let i = 0..5;
        // ANCHOR: reset_then_data
        let seq = once(ResetOrData::Reset).chain(i.map(ResetOrData::Data));
        // ANCHOR_END: reset_then_data
        let collected = seq.collect::<Vec<_>>();
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Data(0),
            ResetOrData::Data(1),
            ResetOrData::Data(2),
            ResetOrData::Data(3),
            ResetOrData::Data(4),
        ];
        assert_eq!(collected, expected);
    }

    #[test]
    fn test_multi_reset() {
        const N: usize = 3;
        let i = 0..5;
        // ANCHOR: reset_N_then_data
        let seq = repeat_n(ResetOrData::Reset, N).chain(i.map(ResetOrData::Data));
        // ANCHOR_END: reset_N_then_data
        let collected = seq.collect::<Vec<_>>();
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(0),
            ResetOrData::Data(1),
            ResetOrData::Data(2),
            ResetOrData::Data(3),
            ResetOrData::Data(4),
        ];
        assert_eq!(collected, expected);
    }

    #[test]
    fn test_no_reset() {
        let i = 0..5;
        // ANCHOR: no-reset
        let seq = i.map(ResetOrData::Data);
        // ANCHOR_END: no-reset
        let collected = seq.collect::<Vec<_>>();
        let expected = vec![
            ResetOrData::Data(0),
            ResetOrData::Data(1),
            ResetOrData::Data(2),
            ResetOrData::Data(3),
            ResetOrData::Data(4),
        ];
        assert_eq!(collected, expected);
    }
}

pub mod reset_ext {

    #[test]
    fn test_with_reset() {
        use rhdl::core::sim::ResetOrData;
        use rhdl::prelude::*;
        let i = 0..5;
        let i = i.map(b8);
        const N: usize = 2;
        // ANCHOR: with-reset-usage
        let seq = i.with_reset(N);
        // ANCHOR_END: with-reset-usage
        let collected = seq.collect::<Vec<_>>();
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(b8(0)),
            ResetOrData::Data(b8(1)),
            ResetOrData::Data(b8(2)),
            ResetOrData::Data(b8(3)),
            ResetOrData::Data(b8(4)),
        ];
        assert_eq!(collected, expected);
    }

    #[test]
    fn test_without_reset() {
        use rhdl::core::sim::ResetOrData;
        use rhdl::prelude::*;
        let i = 0..5;
        let i = i.map(b8);
        // ANCHOR: without-reset-usage
        let seq = i.without_reset();
        // ANCHOR_END: without-reset-usage
        let collected = seq.collect::<Vec<_>>();
        let expected = vec![
            ResetOrData::Data(b8(0)),
            ResetOrData::Data(b8(1)),
            ResetOrData::Data(b8(2)),
            ResetOrData::Data(b8(3)),
            ResetOrData::Data(b8(4)),
        ];
        assert_eq!(collected, expected);
    }
}

#[cfg(test)]
pub mod merge_map {
    use rhdl::prelude::*;
    use std::io::Write;

    #[test]
    fn test_merge_map() {
        // ANCHOR: stream1
        let stream1: Vec<TimedSample<b8>> = vec![
            timed_sample(0, b8(0xa0)),
            timed_sample(5, b8(0xa1)),
            timed_sample(10, b8(0xa2)),
        ];
        // ANCHOR_END: stream1

        // ANCHOR: stream2
        let stream2: Vec<TimedSample<b8>> = vec![
            timed_sample(1, b8(0xb1)),
            timed_sample(3, b8(0xb2)),
            timed_sample(6, b8(0xb3)),
            timed_sample(10, b8(0xb4)),
        ];
        // ANCHOR_END: stream2

        // ANCHOR: merge-stream
        let merged = stream1.merge_map(stream2, |a: b8, b: b8| (a, b));
        // ANCHOR_END: merge-stream

        let file = std::fs::File::create("merge_map.txt").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writeln!(writer, "| time | value |").unwrap();
        writeln!(writer, "|------|-------|").unwrap();
        for sample in merged.clone() {
            writeln!(writer, "| {} |  {:?} |", sample.time, sample.value).unwrap();
        }

        let merged = merged.collect::<Vec<_>>();
        let stream_merged: Vec<TimedSample<(b8, b8)>> = vec![
            timed_sample(0, (b8(0xa0), b8(0))),
            timed_sample(1, (b8(0xa0), b8(0xb1))),
            timed_sample(3, (b8(0xa0), b8(0xb2))),
            timed_sample(5, (b8(0xa1), b8(0xb2))),
            timed_sample(6, (b8(0xa1), b8(0xb3))),
            timed_sample(10, (b8(0xa2), b8(0xb4))),
        ];

        assert_eq!(merged, stream_merged);
    }

    #[test]
    fn test_red_blue_clocks() {
        // ANCHOR: red-blue-merge
        #[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
        pub struct In {
            pub cr_w: Signal<ClockReset, Red>,
            pub cr_r: Signal<ClockReset, Blue>,
        }
        let red_input = std::iter::repeat(()).with_reset(1).clock_pos_edge(50);
        let blue_input = std::iter::repeat(()).with_reset(1).clock_pos_edge(78);
        let input = red_input.merge_map(blue_input, |r, b| In {
            cr_w: signal(r.0),
            cr_r: signal(b.0),
        });
        // ANCHOR_END: red-blue-merge

        let file = std::fs::File::create("red_blue_clocks.txt").unwrap();
        let mut writer = std::io::BufWriter::new(file);
        writeln!(
            writer,
            "| time | red clock | red reset | blue clock | blue reset |"
        )
        .unwrap();
        writeln!(
            writer,
            "|------|-----------|-----------|------------|------------|"
        )
        .unwrap();
        for sample in input.take(16) {
            writeln!(
                writer,
                "| {} |     {}     |     {}     |     {}      |     {}      |",
                sample.time,
                sample.value.cr_w.val().clock.raw(),
                sample.value.cr_w.val().reset.raw(),
                sample.value.cr_r.val().clock.raw(),
                sample.value.cr_r.val().reset.raw(),
            )
            .unwrap();
        }
    }
}

#[cfg(test)]
pub mod roll_sim {
    use rhdl::prelude::*;

    #[derive(Circuit, Clone)]
    struct XorGate;

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
    fn xor_gate(i: Signal<(bool, bool), Red>, _q: ()) -> (Signal<bool, Red>, ()) {
        let (a, b) = i.val();
        let c = a ^ b;
        (signal(c), ())
    }

    #[test]
    fn test_simulation() {
        // ANCHOR: roll-sim

        let inputs = [(false, false), (false, true), (true, false), (true, true)];
        let outputs = [false, true, true, false];
        let gate = XorGate;
        let mut state = gate.init();
        for (inp, outp) in inputs.iter().zip(outputs.iter()) {
            let output = gate.sim(signal(*inp), &mut state);
            assert_eq!(output.val(), *outp);
        }

        // ANCHOR_END: roll-sim
    }
}

pub mod timed_sample {
    use rhdl::{core::types::timed_sample::TraceStatus, prelude::*};

    // ANCHOR: timed_sample

    /// A sample of a digital value at a specific time.
    #[derive(Copy, Clone, Debug, PartialEq, Hash)]
    pub struct TimedSample<T: Digital> {
        /// The digital value being sampled.
        pub value: T,
        /// The time at which the sample was taken.
        pub time: u64,
        /// The trace status of the sample.
        pub trace_status: TraceStatus,
    }

    // ANCHOR_END: timed_sample
}
