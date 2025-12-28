use std::path::Path;

use crate::{Clock, ClockReset, Digital, TimedSample, trace2::trace_sample::TracedSample};

use super::{
    edges::{EdgeTime, edge_time},
    glitch_check::{GlitchCheck, glitch_check},
    sample_at_pos_edge::{SampleAtPosEdge, sample_at_pos_edge},
    synchronous_sample::{SynchronousSample, synchronous_sample},
    vcd_file::{VCDFile, vcd_file},
};

pub trait ProbeExt<I, S, U> {
    fn sample_at_pos_edge<F>(self, clock_fn: F) -> SampleAtPosEdge<I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&I::Item) -> Clock;

    fn glitch_check<F, T>(self, clock_fn: F) -> GlitchCheck<T, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TracedSample<S, U>) -> (Clock, T),
        S: Digital,
        U: Digital,
        T: Digital;
    fn vcd_file(self, file: &Path) -> VCDFile<I>
    where
        Self: Sized,
        I: Iterator<Item = TracedSample<S, U>>,
        S: Digital,
        U: Digital;
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

pub trait SynchronousProbeExt<I, P, O> {
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
