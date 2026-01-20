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
