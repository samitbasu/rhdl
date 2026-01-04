//! Probe to check for glitches in a stream of traced samples
use crate::{Clock, Digital, trace::trace_sample::TracedSample};

/// The GlitchCheck struct.  Not intended to be used directly;
/// use the [glitch_check] function to create one, or use
/// the extension trait in [crate::sim::probe::ext].
/// See the [book] for an example of its use.
pub struct GlitchCheck<T, I, F> {
    clk: Clock,
    prev_val: T,
    iter: I,
    func: F,
    initialized: bool,
    edge_time: u64,
    tolerance: u64,
}

impl<T, I, F> Clone for GlitchCheck<T, I, F>
where
    I: Clone,
    T: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        GlitchCheck {
            clk: self.clk,
            prev_val: self.prev_val.clone(),
            iter: self.iter.clone(),
            func: self.func.clone(),
            initialized: self.initialized,
            edge_time: self.edge_time,
            tolerance: self.tolerance,
        }
    }
}

/// Create a glitch-checking iterator over the supplied stream of traced samples,
/// using the supplied function to extract the clock and value to monitor for glitches.
/// A glitch is defined as a change in the monitored value that occurs
/// outside of a clock positive edge, and outside of the specified tolerance window
/// (which is 1 time unit by default).
pub fn glitch_check<T, I, F>(stream: I, clock_fn: F) -> GlitchCheck<T, I, F>
where
    T: Digital,
{
    GlitchCheck {
        clk: Clock::default(),
        prev_val: T::dont_care(),
        iter: stream,
        func: clock_fn,
        initialized: false,
        edge_time: 0,
        tolerance: 1,
    }
}

impl<T, I, F, S, U> Iterator for GlitchCheck<T, I, F>
where
    T: Digital + std::fmt::Debug,
    I: Iterator<Item = TracedSample<S, U>>,
    F: Fn(&TracedSample<S, U>) -> (Clock, T),
    S: Digital,
    U: Digital,
{
    type Item = TracedSample<S, U>;

    fn next(&mut self) -> Option<TracedSample<S, U>> {
        if !self.initialized {
            if let Some(sample) = self.iter.next() {
                (self.clk, self.prev_val) = (self.func)(&sample);
                self.initialized = true;
                return Some(sample);
            } else {
                return None;
            }
        }
        if let Some(sample) = self.iter.next() {
            let (clk, value) = (self.func)(&sample);
            let pos_edge = clk.raw() && !self.clk.raw();
            let output_changed = value != self.prev_val;
            if output_changed && !pos_edge && sample.time - self.edge_time > self.tolerance {
                panic!(
                    "Glitch detected at time: {time}, sample = {value:?}, prev_val = {prev:?}",
                    time = sample.time,
                    prev = self.prev_val
                );
            }
            if pos_edge {
                self.edge_time = sample.time;
            }
            self.clk = clk;
            self.prev_val = value;
            Some(sample)
        } else {
            None
        }
    }
}
