//! Probe to sample values before a positive clock edge for synchronous designs
use crate::{Clock, ClockReset, Digital, trace::trace_sample::TracedSample};

/// This probe collects samples a stream of values _before_ a
/// positive clock edge.  You must provide a closure that extracts
/// the clock signal from the stream that you want to sample.  When
/// ever that clock signal experiences a positive edge, this probe will
/// emit the _previous_ value for the stream.  
///
/// Note that unlike [crate::sim::probe::sample_at_pos_edge::SampleAtPosEdge], this probe assumes that the input
/// stream is synchronous, i.e., it contains clock and reset information
/// as part of its input type.
pub struct SynchronousSample<S>
where
    S: Iterator,
{
    stream: S,
    clock: Clock,
    last: Option<S::Item>,
}

impl<S> Clone for SynchronousSample<S>
where
    S: Clone + Iterator,
    <S as Iterator>::Item: Clone,
{
    fn clone(&self) -> Self {
        SynchronousSample {
            stream: self.stream.clone(),
            clock: self.clock,
            last: self.last.clone(),
        }
    }
}

/// Create a probe that samples values from the supplied stream
pub fn synchronous_sample<S>(stream: S) -> SynchronousSample<S>
where
    S: Iterator,
{
    SynchronousSample {
        stream,
        clock: Clock::default(),
        last: None,
    }
}

impl<S, I, O> Iterator for SynchronousSample<S>
where
    S: Iterator<Item = TracedSample<(ClockReset, I), O>>,
    I: Digital,
    O: Digital,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        loop {
            match self.stream.next() {
                None => return self.last.take(),
                Some(sample) => {
                    let clock = sample.input.0.clock;
                    if clock.raw() && !self.clock.raw() && self.last.is_some() {
                        return self.last.take();
                    }
                    self.last = Some(sample);
                    self.clock = clock;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::super::ext::SynchronousProbeExt;
    use crate::{sim::extension::*, trace::session::Session};
    use rhdl_bits::alias::*;

    #[test]
    fn test_before_pos_edge() {
        let data: Vec<_> = vec![0, 0, 1, 1, 3, 3, 2, 2, 0, 9];
        let stream = data
            .iter()
            .copied()
            .map(b8)
            .without_reset()
            .clock_pos_edge(100);
        let session = Session::default();
        let output = stream.map(|t| session.untraced(t, t.value.1));
        let probe = output.synchronous_sample();
        let result: Vec<_> = probe.map(|t| t.input.1).collect();
        assert_eq!(result, data);
    }
}
