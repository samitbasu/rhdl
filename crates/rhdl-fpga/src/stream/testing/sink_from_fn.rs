//! Test Sink From Function
//!
//!# Purpose
//!
//! For testing stream processes, it's often handy to have a
//! sink for a stream that can be generated from a closure without
//! worrying that something that is synthesizable.  
//!
use rhdl::prelude::*;

use crate::stream::{ready, Ready};

#[derive(Clone)]
/// The [SinkFromFn] core
///
/// This is the core to include in your design if you want to  
/// use a closure or other general Rust function to assess the
/// correctness of the stream output.  It can also control the
/// backpressure to the stream, by returning a boolean that
/// is converted into the `ready` input.  
pub struct SinkFromFn<T: Digital> {
    consumer: std::sync::Arc<std::sync::Mutex<dyn FnMut(Option<T>) -> bool>>,
}

impl<T: Digital> SinkFromFn<T> {
    /// Create a new [SinkFromFn] object from the given function
    ///
    /// Note that the function is of the form `fn(Option<T>) -> bool`.
    /// The return type is _not_ an acceptance flag for the argument.
    /// Rather, it signals the pipeline readiness to the pipe stage
    /// immediately preceeding this one.
    ///
    pub fn new<S: FnMut(Option<T>) -> bool + 'static>(consumer: S) -> Self {
        Self {
            consumer: std::sync::Arc::new(std::sync::Mutex::new(consumer)),
        }
    }
}

impl<T: Digital + std::fmt::Debug> SinkFromFn<T> {
    /// Create a new [SinkFromFn] object from the given iterator
    ///
    /// This constructor will create a sink that expects each item from the
    /// sink to match an item from the generated iterator.  It will also
    /// return a random number indicating acceptance based on the passed probability.
    pub fn new_from_iter<S: Iterator<Item = T> + 'static>(
        mut iter: S,
        stall_probability: f32,
    ) -> Self {
        let func = move |x| {
            if let Some(res) = x {
                let y = iter.next().unwrap();
                assert_eq!(res, y);
            }
            rand::random::<f32>() > stall_probability
        };
        Self::new(func)
    }
}

impl<T> SynchronousIO for SinkFromFn<T>
where
    T: Digital,
{
    // Data signal
    type I = Option<T>;
    // Ready signal
    type O = Ready<T>;
    type Kernel = NoKernel3<ClockReset, Option<T>, (), (Ready<T>, ())>;
}

impl<T> SynchronousDQ for SinkFromFn<T>
where
    T: Digital,
{
    type D = ();
    type Q = ();
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Init,
    Run,
}

#[derive(Clone, PartialEq)]
#[doc(hidden)]
pub struct SinkFromFnState<T: Digital> {
    state: State,
    latched_value: Option<T>,
    prev_clock: Clock,
    ready: bool,
}

impl<T: Digital> Synchronous for SinkFromFn<T> {
    type S = SinkFromFnState<T>;

    fn init(&self) -> Self::S {
        SinkFromFnState {
            state: State::Init,
            latched_value: None,
            prev_clock: clock(false),
            ready: false,
        }
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, me: &mut Self::S) -> Self::O {
        trace_push_path("sink_from_fn");
        trace("input", &input);
        let pos_edge = clock_reset.clock.raw() && !me.prev_clock.raw();
        let process = || {
            let mut consumer = self.consumer.lock().unwrap();
            (consumer)(if !me.ready { None } else { me.latched_value })
        };
        match me.state {
            State::Init => {
                if !clock_reset.reset.any() && clock_reset.clock.raw() {
                    me.ready = process();
                    me.state = State::Run;
                }
            }
            State::Run => {
                if pos_edge {
                    me.ready = process();
                }
            }
        }
        if !clock_reset.clock.raw() {
            me.latched_value = input;
        }
        me.prev_clock = clock_reset.clock;
        trace("output", &me.ready);
        trace_pop_path();
        ready(me.ready)
    }

    fn descriptor(&self, _name: &str) -> Result<Descriptor, RHDLError> {
        Err(RHDLError::NotSynthesizable)
    }

    fn children(&self) -> impl Iterator<Item = Result<rhdl::core::Descriptor, RHDLError>> {
        std::iter::empty()
    }
}
