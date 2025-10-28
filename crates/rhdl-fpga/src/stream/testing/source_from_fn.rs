//! Test Source From Function
//!
//!# Purpose
//!
//! For testing streams, it's often handy to have a source of data elements
//! that can be generated from a closure without worrying about creating
//! something that is synthesizable.  This struct takes an iterator and
//! wraps it into a simulatable (but _not_ synthesizable) core that can
//! be included in your simulations and test fixtures.
//!

use rhdl::prelude::*;

use crate::stream::Ready;

#[derive(Clone)]
/// The [SourceFromFn] core
///
/// This is the core to include in your design/test fixture if you want
/// to use a closure or other general Rust function to provide test
/// data to the pipe input.  It responds to the incoming backpressure signal
/// via the `ready` input.  Furthermore, if the iterator wants to stall,
/// it can return `None`.
pub struct SourceFromFn<T: Digital> {
    generator: std::sync::Arc<std::sync::Mutex<dyn Iterator<Item = Option<T>>>>,
}

impl<T: Digital> SourceFromFn<T> {
    /// Create a new [SourceFromFn] object from the given iterator
    ///
    /// Note that the iterator will return a sequence of [Option<T>].
    /// When the iterator is exhausted, no further data items will be
    /// returned.  _This does not stop the simulation_.  Stopping the
    /// simulation must be handled in some other way, as `rhdl` does
    /// not know when the last item might exit the stream.
    pub fn new<S: Iterator<Item = Option<T>> + 'static>(generator: S) -> Self {
        Self {
            generator: std::sync::Arc::new(std::sync::Mutex::new(generator)),
        }
    }
}

impl<T> SynchronousIO for SourceFromFn<T>
where
    T: Digital,
{
    // Ready signal
    type I = Ready<T>;
    // data element
    type O = Option<T>;
    type Kernel = NoKernel3<ClockReset, Ready<T>, (), (Option<T>, ())>;
}

impl<T> SynchronousDQ for SourceFromFn<T>
where
    T: Digital,
{
    type D = ();
    type Q = ();
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Init,
    Hold,
    Done,
    Stalled,
}

#[derive(Clone, PartialEq)]
#[doc(hidden)]
pub struct FromFnState<T: Digital> {
    state: State,
    value: Option<T>,
    prev_clock: Clock,
    latched_ready: bool,
}

impl<T: Digital> Synchronous for SourceFromFn<T> {
    type S = FromFnState<T>;

    fn init(&self) -> Self::S {
        FromFnState {
            value: None,
            state: State::Init,
            prev_clock: clock(false),
            latched_ready: false,
        }
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, me: &mut Self::S) -> Self::O {
        trace_push_path("from_fn_stream");
        trace("input", &input);
        let pos_edge = clock_reset.clock.raw() && !me.prev_clock.raw();
        // We use this a lot
        let mut gen_fn = || {
            let mut generator = self.generator.lock().unwrap();
            let value = generator.next();
            match value {
                None => {
                    me.value = None;
                    State::Done
                }
                Some(t) => match t {
                    None => {
                        me.value = None;
                        State::Stalled
                    }
                    Some(_) => {
                        me.value = t;
                        State::Hold
                    }
                },
            }
        };
        match me.state {
            State::Init => {
                if !clock_reset.reset.any() && clock_reset.clock.raw() {
                    me.state = gen_fn()
                }
            }
            State::Stalled => {
                if pos_edge {
                    // Positive edge.  Try to generate data
                    me.state = gen_fn()
                }
            }
            State::Hold => {
                // We are holding valid data.  If the ready signal was true when our
                // edge arrived, then we can release this value
                if pos_edge && me.latched_ready {
                    me.state = gen_fn()
                }
            }
            State::Done => {}
        }
        if !clock_reset.clock.raw() {
            me.latched_ready = input.raw;
        }
        me.prev_clock = clock_reset.clock;
        trace("output", &me.value);
        trace_pop_path();
        me.value
    }

    fn descriptor(&self, _name: &str) -> Result<Descriptor, RHDLError> {
        Err(RHDLError::NotSynthesizable)
    }

    fn children(&self) -> impl Iterator<Item = Result<Descriptor, RHDLError>> {
        std::iter::empty()
    }
}
