//! Probe to sample values at a negative clock edge
use crate::Clock;

/// This probe collects samples a stream of values at a
/// negative clock edge.  You must provide a closure that extracts
/// the clock signal from the stream that you want to sample.  When
/// ever that clock signal experiences a negative edge, this probe will
/// emit the _previous_ value for the stream.  
///
/// The struct is not intended to be used directly; use the
/// [sample_at_neg_edge] function to create one, or use
/// the extension trait in [crate::sim::probe::ext].
pub struct SampleAtNegEdge<S, F>
where
    S: Iterator,
{
    stream: S,
    clock_fn: F,
    clock: Clock,
    last: Option<S::Item>,
}

impl<S, F> Clone for SampleAtNegEdge<S, F>
where
    S: Clone + Iterator,
    F: Clone,
    <S as Iterator>::Item: Clone,
{
    fn clone(&self) -> Self {
        SampleAtNegEdge {
            stream: self.stream.clone(),
            clock_fn: self.clock_fn.clone(),
            clock: self.clock,
            last: self.last.clone(),
        }
    }
}

/// Create a probe that samples values from the supplied stream
/// at each negative edge of the clock extracted using
/// the supplied function.
pub fn sample_at_neg_edge<S, F>(stream: S, clock_fn: F) -> SampleAtNegEdge<S, F>
where
    S: Iterator,
    F: Fn(&S::Item) -> Clock,
{
    SampleAtNegEdge {
        stream,
        clock_fn,
        clock: Clock::default(),
        last: None,
    }
}

impl<S, F> Iterator for SampleAtNegEdge<S, F>
where
    S: Iterator,
    F: Fn(&S::Item) -> Clock,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        loop {
            match self.stream.next() {
                None => return None,
                Some(sample) => {
                    let clock = (self.clock_fn)(&sample);
                    if !clock.raw() && self.clock.raw() && self.last.is_some() {
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
    use crate::{sim::extension::*, trace::session::Session};
    use rhdl_bits::alias::*;

    #[test]
    fn test_at_neg_edge() {
        let data = vec![3, 0, 1, 1, 3, 3, 2, 2, 0, 9];
        let stream = data
            .iter()
            .copied()
            .map(b8)
            .without_reset()
            .clock_pos_edge(100);
        let session = Session::default();
        let stream = stream.map(|x| session.untraced(x, ()));
        for t in stream {
            eprintln!(
                "{time} {clock} {value:?}",
                time = t.time,
                clock = t.input.0.clock.raw(),
                value = t.input.1
            );
        }
        /*

               let probe = stream.sample_at_neg_edge(|x| x.input.0.clock);
               let result: Vec<_> = probe.map(|t| t.input.1).collect();
               assert_eq!(result, data);
        */
    }
}
