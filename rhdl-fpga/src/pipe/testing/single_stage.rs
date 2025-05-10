//! Single Stage Test Fixture
//!
//! In this test fixture, we assemble a source, a uut, and a sink
//! into a single assembly:
//!
#![doc=badascii!(r"
+-+Source+-+     +-+UUT+------+      +-+Sink+---+
|          | ?T  |            | ?S   |          |
|     data +---->|in       out+----->|data      |
|          |     |            |      |          |
|          |     |            |      |          |
|    ready |<---+|ready  ready|<-----+ready     |
+----------+     +------------+      +----------+
")]
//! The source is provided by an iterator that yields [Option<T>]
//! values, which are injected into the UUT when `ready` is `true`.
//! The sink is provided by a consumer function that is called
//! with each accepted data element as a [Option<S>], with the
//! `Some` variant indicating that the data element is valid.
//! The consumer function provides the `ready` signal back to the
//! pipeline to supply backpressure.

use crate::pipe::PipeIO;

use super::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn};
use badascii_doc::badascii;
use rhdl::prelude::*;

#[derive(Clone)]
/// The [SingleStage] test fixture
///
/// Create this with the `single_stage` helper function.
/// The result can be simulated, but not synthesized.
pub struct SingleStage<S, T, C>
where
    S: Digital,
    T: Digital,
    C: Synchronous<I = PipeIO<S>, O = PipeIO<T>>,
{
    source: SourceFromFn<S>,
    uut: C,
    sink: SinkFromFn<T>,
}

#[derive(PartialEq, Digital)]
#[doc(hidden)]
pub struct D<S, T>
where
    S: Digital,
    T: Digital,
{
    source: bool,
    uut: PipeIO<S>,
    sink: PipeIO<T>,
}

#[derive(PartialEq, Digital)]
#[doc(hidden)]
pub struct Q<S, T>
where
    S: Digital,
    T: Digital,
{
    source: Option<S>,
    uut: PipeIO<T>,
    sink: bool,
}

impl<S, T, C> SynchronousDQ for SingleStage<S, T, C>
where
    S: Digital,
    T: Digital,
    C: Synchronous<I = PipeIO<S>, O = PipeIO<T>>,
{
    type D = D<S, T>;
    type Q = Q<S, T>;
}

/// Create a single stage test fixture
///
/// `uut` is the pipe stage to be tested
/// `source` is an iterator that returns items of type `Option<S>`
/// `sink` is a function that consumes elements of type `Option<T>`
/// and provides the backpressure signal as a return.
///
/// To see example usages, look at the [FlattenPipe] and
/// [ChunkPipe] examples.
pub fn single_stage<S, T, C>(
    uut: C,
    source: impl Iterator<Item = Option<S>> + 'static,
    sink: impl FnMut(Option<T>) -> bool + 'static,
) -> SingleStage<S, T, C>
where
    S: Digital,
    T: Digital,
    C: Synchronous<I = PipeIO<S>, O = PipeIO<T>>,
{
    SingleStage {
        source: SourceFromFn::new(source),
        uut,
        sink: SinkFromFn::new(sink),
    }
}

impl<S, T, C> SynchronousIO for SingleStage<S, T, C>
where
    S: Digital,
    T: Digital,
    C: Synchronous<I = PipeIO<S>, O = PipeIO<T>>,
{
    type I = ();
    type O = ();
    type Kernel = NoKernel3<ClockReset, (), Q<S, T>, ((), D<S, T>)>;
}

impl<S, T, C> Synchronous for SingleStage<S, T, C>
where
    S: Digital,
    T: Digital,
    C: Synchronous<I = PipeIO<S>, O = PipeIO<T>>,
{
    type S = (
        Self::Q,
        <SourceFromFn<S> as rhdl::core::Synchronous>::S,
        <C as rhdl::core::Synchronous>::S,
        <SinkFromFn<T> as rhdl::core::Synchronous>::S,
    );
    fn init(&self) -> Self::S {
        (
            <<Self as rhdl::core::SynchronousDQ>::Q as rhdl::core::Digital>::dont_care(),
            self.source.init(),
            self.uut.init(),
            self.sink.init(),
        )
    }
    fn descriptor(
        &self,
        _name: &str,
    ) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
        Err(RHDLError::NotSynthesizable)
    }
    fn hdl(&self, _name: &str) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
        Err(RHDLError::NotSynthesizable)
    }
    fn sim(
        &self,
        clock_reset: rhdl::core::ClockReset,
        input: <Self as SynchronousIO>::I,
        state: &mut Self::S,
    ) -> <Self as SynchronousIO>::O {
        rhdl::core::trace("input", &input);
        for _ in 0..rhdl::core::MAX_ITERS {
            let prev_state = state.clone();
            // Do the wiring here
            //
            // +-+Source+-+     +-+UUT+------+      +-+Sink+---+
            // |          | ?T  |            | ?S   |          |
            // |     data +---->|in       out+----->|data      |
            // |          |     |            |      |          |
            // |          |     |            |      |          |
            // |    ready |<---+|ready  ready|<-----+ready     |
            // +----------+     +------------+      +----------+
            //
            let mut internal_inputs = D::<S, T>::dont_care();
            internal_inputs.source = state.0.uut.ready;
            internal_inputs.uut.ready = state.0.sink;
            internal_inputs.uut.data = state.0.source;
            internal_inputs.sink.data = state.0.uut.data;
            let outputs = ();
            rhdl::core::trace_push_path(stringify!(source));
            state.0.source = self
                .source
                .sim(clock_reset, internal_inputs.source, &mut state.1);
            rhdl::core::trace_pop_path();
            rhdl::core::trace_push_path(stringify!(uut));
            state.0.uut = self.uut.sim(clock_reset, internal_inputs.uut, &mut state.2);
            rhdl::core::trace_pop_path();
            rhdl::core::trace_push_path(stringify!(sink));
            state.0.sink = self
                .sink
                .sim(clock_reset, internal_inputs.sink.data, &mut state.3);
            rhdl::core::trace_pop_path();
            if state == &prev_state {
                rhdl::core::trace("outputs", &outputs);
                return outputs;
            }
        }
        panic!("Simulation did not converge");
    }
}
