use crate::{Clock, Digital, TimedSample};

use super::{
    before_pos_edge::{before_pos_edge, BeforePosEdge},
    glitch_free::{glitch_free, GlitchFree},
};

pub trait ProbeExt<I, O> {
    fn before_pos_edge<F>(self, clock_fn: F) -> BeforePosEdge<I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&I::Item) -> Clock;

    fn glitch_free<F>(self, clock_fn: F) -> GlitchFree<O, I, F>
    where
        Self: Sized,
        I: Iterator,
        F: Fn(&TimedSample<O>) -> Clock,
        O: Digital;
}

impl<I, O> ProbeExt<I, O> for I
where
    I: Iterator<Item = TimedSample<O>>,
    O: Digital,
{
    fn before_pos_edge<F>(self, clock_fn: F) -> BeforePosEdge<I, F>
    where
        F: Fn(&I::Item) -> Clock,
    {
        before_pos_edge(self, clock_fn)
    }

    fn glitch_free<F>(self, clock_fn: F) -> GlitchFree<O, I, F>
    where
        F: Fn(&I::Item) -> Clock,
    {
        glitch_free(self, clock_fn)
    }
}
