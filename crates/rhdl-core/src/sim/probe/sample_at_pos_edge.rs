use crate::Clock;

/// This probe collects samples a stream of values _before_ a
/// positive clock edge.  You must provide a closure that extracts
/// the clock signal from the stream that you want to sample.  When
/// ever that clock signal experiences a positive edge, this probe will
/// emit the _previous_ value for the stream.
pub struct SampleAtPosEdge<S, F>
where
    S: Iterator,
{
    stream: S,
    clock_fn: F,
    clock: Clock,
    last: Option<S::Item>,
}

impl<S, F> Clone for SampleAtPosEdge<S, F>
where
    S: Clone + Iterator,
    F: Clone,
    <S as Iterator>::Item: Clone,
{
    fn clone(&self) -> Self {
        SampleAtPosEdge {
            stream: self.stream.clone(),
            clock_fn: self.clock_fn.clone(),
            clock: self.clock,
            last: self.last.clone(),
        }
    }
}

pub fn sample_at_pos_edge<S, F>(stream: S, clock_fn: F) -> SampleAtPosEdge<S, F>
where
    S: Iterator,
    F: Fn(&S::Item) -> Clock,
{
    SampleAtPosEdge {
        stream,
        clock_fn,
        clock: Clock::default(),
        last: None,
    }
}

impl<S, F> Iterator for SampleAtPosEdge<S, F>
where
    S: Iterator,
    F: Fn(&S::Item) -> Clock,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        loop {
            match self.stream.next() {
                None => return self.last.take(),
                Some(sample) => {
                    let clock = (self.clock_fn)(&sample);
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

    use super::super::ext::ProbeExt;
    use crate::{sim::extension::*, trace2::session::Session};
    use rhdl_bits::alias::*;

    #[test]
    fn test_before_pos_edge() {
        let data = vec![0, 0, 1, 1, 3, 3, 2, 2, 0, 9];
        let stream = data
            .iter()
            .copied()
            .map(b8)
            .without_reset()
            .clock_pos_edge(100);
        let session = Session::default();
        let stream = stream.map(|x| session.untraced(x, ()));
        let probe = stream.sample_at_pos_edge(|x| x.input.0.clock);
        let result: Vec<_> = probe.map(|t| t.input.1).collect();
        assert_eq!(result, data);
    }
}
