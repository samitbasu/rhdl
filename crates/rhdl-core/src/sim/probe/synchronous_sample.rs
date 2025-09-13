use crate::{Clock, ClockReset, Digital, TimedSample};

/// This probe collects samples a stream of values _before_ a
/// positive clock edge.  You must provide a closure that extracts
/// the clock signal from the stream that you want to sample.  When
/// ever that clock signal experiences a positive edge, this probe will
/// emit the _previous_ value for the stream.
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
    S: Iterator<Item = TimedSample<(ClockReset, I, O)>>,
    I: Digital,
    O: Digital,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<S::Item> {
        loop {
            match self.stream.next() {
                None => return self.last.take(),
                Some(sample) => {
                    let clock = sample.value.0.clock;
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
    use rhdl_bits::alias::*;
    use crate::sim::{clock_pos_edge::ClockPosEdgeExt, reset::TimedStreamExt};

    #[test]
    fn test_before_pos_edge() {
        let data: Vec<_> = vec![0, 0, 1, 1, 3, 3, 2, 2, 0, 9];
        let stream = data.iter().copied().map(b8).without_reset().clock_pos_edge(100);
        let output = stream.map(|t| t.map(|v| (v.0, v.1, v.1)));
        let probe = output.synchronous_sample();
        let result: Vec<_> = probe.map(|t| t.value.1).collect();
        assert_eq!(result, data);
    }
}
