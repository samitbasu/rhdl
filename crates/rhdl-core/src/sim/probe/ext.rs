use std::path::Path;

use crate::{Clock, ClockReset, Digital, TimedSample};

use super::{
    edges::{edge_time, EdgeTime},
    glitch_check::{glitch_check, GlitchCheck},
    sample_at_pos_edge::{sample_at_pos_edge, SampleAtPosEdge},
    synchronous_sample::{synchronous_sample, SynchronousSample},
    vcd_file::{vcd_file, VCDFile},
};

pub trait ProbeExt<I, S> {
    fn sample_at_pos_edge<F>(self, clock_fn: F) -> SampleAtPosEdge<I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&I::Item) -> Clock;

    fn glitch_check<F, T>(self, clock_fn: F) -> GlitchCheck<T, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TimedSample<S>) -> (Clock, T),
        S: Digital,
        T: Digital;
    fn vcd_file(self, file: &Path) -> VCDFile<I>
    where
        Self: Sized,
        I: Iterator<Item = TimedSample<S>>,
        S: Digital;
    fn edge_time<F, T>(self, data_fn: F) -> EdgeTime<T, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TimedSample<S>) -> T,
        T: Digital;
}

impl<I, S> ProbeExt<I, S> for I
where
    I: Iterator<Item = TimedSample<S>>,
    S: Digital,
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
        F: Fn(&TimedSample<S>) -> T,
        T: Digital,
    {
        edge_time(self, data_fn)
    }
}

pub trait SynchronousProbeExt<I, P, O> {
    fn synchronous_sample(self) -> SynchronousSample<I>
    where
        Self: Sized,
        I: Iterator<Item = TimedSample<(ClockReset, P, O)>>,
        P: Digital,
        O: Digital;
}

impl<I, P, O> SynchronousProbeExt<I, P, O> for I {
    fn synchronous_sample(self) -> SynchronousSample<I>
    where
        Self: Sized,
        I: Iterator<Item = TimedSample<(ClockReset, P, O)>>,
        P: Digital,
        O: Digital,
    {
        synchronous_sample(self)
    }
}
