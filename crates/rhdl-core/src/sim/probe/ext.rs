//! Extension traits for probes
use std::path::Path;

use crate::{Clock, ClockReset, Digital, trace::trace_sample::TracedSample};

use super::{
    edges::{EdgeTime, edge_time},
    glitch_check::{GlitchCheck, glitch_check},
    sample_at_pos_edge::{SampleAtPosEdge, sample_at_pos_edge},
    synchronous_sample::{SynchronousSample, synchronous_sample},
    vcd_file::{VCDFile, vcd_file},
};

/// Extension trait to add probe methods to iterators
pub trait ProbeExt<I, S, U> {
    /// Create a probe that samples values from the supplied stream
    /// just before a positive edge of the clock extracted using
    /// the supplied function.
    fn sample_at_pos_edge<F>(self, clock_fn: F) -> SampleAtPosEdge<I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&I::Item) -> Clock;
    /// Create a glitch-checking iterator over the supplied stream of traced samples,
    /// using the supplied function to extract the clock and value to monitor for glitches. A glitch is defined as a change in the monitored value that occurs
    /// outside of a clock positive edge, and outside of the specified tolerance window
    /// (which is 1 time unit by default).
    fn glitch_check<F, T>(self, clock_fn: F) -> GlitchCheck<T, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TracedSample<S, U>) -> (Clock, T),
        S: Digital,
        U: Digital,
        T: Digital;
    /// Create a VCD file-writing probe over the supplied stream of traced samples.
    fn vcd_file(self, file: &Path) -> VCDFile<I>
    where
        Self: Sized,
        I: Iterator<Item = TracedSample<S, U>>,
        S: Digital,
        U: Digital;
    /// Create an edge-detecting iterator over the supplied stream of traced samples,
    /// using the supplied function to extract the value to monitor for edges/changes.
    fn edge_time<F, T>(self, data_fn: F) -> EdgeTime<T, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TracedSample<S, U>) -> T,
        T: Digital;
}

impl<I, S, U> ProbeExt<I, S, U> for I
where
    I: Iterator<Item = TracedSample<S, U>>,
    S: Digital,
    U: Digital,
{
    fn sample_at_pos_edge<F>(self, clock_fn: F) -> SampleAtPosEdge<I, F>
    where
        F: Fn(&I::Item) -> Clock,
    {
        sample_at_pos_edge(self, clock_fn)
    }

    fn glitch_check<F, T>(self, clock_fn: F) -> GlitchCheck<T, I, F>
    where
        F: Fn(&I::Item) -> (Clock, T),
        T: Digital,
    {
        glitch_check(self, clock_fn)
    }

    fn vcd_file(self, file: &Path) -> VCDFile<I> {
        vcd_file(self, file)
    }

    fn edge_time<F, T>(self, data_fn: F) -> EdgeTime<T, I, F>
    where
        F: Fn(&TracedSample<S, U>) -> T,
        T: Digital,
    {
        edge_time(self, data_fn)
    }
}

/// Extension trait to add synchronous sampling probe method to iterators
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

impl<I, P, O> SynchronousProbeExt<I, P, O> for I {
    fn synchronous_sample(self) -> SynchronousSample<I>
    where
        Self: Sized,
        I: Iterator<Item = TracedSample<(ClockReset, P), O>>,
        P: Digital,
        O: Digital,
    {
        synchronous_sample(self)
    }
}
