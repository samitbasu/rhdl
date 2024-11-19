use crate::{Clock, Digital, TimedSample};

use super::{
    glitch_check::{glitch_check, GlitchCheck},
    sample_at_pos_edge::{sample_at_pos_edge, SampleAtPosEdge},
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
}
