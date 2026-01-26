//! An iterator adaptor that clocks out a data/reset stream on the positive edge of a clock.
//!
//! Details are in the book.  But generally, given an iterator that generates a sequence of data
//! values `x1, x2, x3, ...`, this adaptor will produce a timed sequence of clock and reset
//! that looks like this:
//!
#![doc = badascii_doc::badascii!(
r#"
data     +----------+--------------------------------+-----+
            x1      |              x2                |  x3  
         +----------+--------------------------------+-----+
                +>|δ|<+                          +>|δ|<+    
                  +---------------+                +-------+
clk               | :             |                | :      
         +--------+ :             +----------------+ :      
                                                            
         ^        ^ ^             ^                ^ ^      
         +        + +             +                + +      
                                                            
 data   x1       x1 x2           x2               x2 x3     
 clk     F        T T             F                T T      
"#
)]
//!
//! The hold time `δ` is one time unit, and the clock period is configurable.
use crate::{
    Clock, ClockReset, Digital, TimedSample, clock::clock, clock_reset, sim::ResetOrData,
    timed_sample, types::reset::reset,
};

#[derive(Clone)]
enum State {
    Init,
    Hold,
    ClockLow,
    ClockHigh,
    TailStart,
    TailEnd,
    Done,
}

/// An iterator adaptor that takes samples and produces a timed stream of clock and
/// reset signals along with the samples.
///
/// Normally you would create this adapter using the `ClockPosEdgeExt` trait.
pub struct ClockPosEdge<I, S>
where
    S: Digital,
{
    input: I,
    sample: ResetOrData<S>,
    state: State,
    time: u64,
    next_time: u64,
    period: u64,
}

impl<I, S> ClockPosEdge<I, S>
where
    S: Digital,
{
    fn this_sample(&self, clock: Clock) -> TimedSample<(ClockReset, S)> {
        match self.sample {
            ResetOrData::Data(x) => timed_sample(self.time, (clock_reset(clock, reset(false)), x)),
            ResetOrData::Reset => {
                timed_sample(self.time, (clock_reset(clock, reset(true)), S::dont_care()))
            }
        }
    }
}

impl<I, S> Clone for ClockPosEdge<I, S>
where
    I: Clone,
    S: Clone + Digital,
{
    fn clone(&self) -> Self {
        ClockPosEdge {
            input: self.input.clone(),
            sample: self.sample,
            state: self.state.clone(),
            time: self.time,
            next_time: self.next_time,
            period: self.period,
        }
    }
}

//
// The waveform
//
//  data    ----------------*---------------------------*
//             x1           |      x2                   |
//          ----------------*---------------------------*
//
//                     *----------------*               *-----*
//  clk                |                |               |
//          *----------*                *---------------*
//
//  state  init        ^ hold ^ high    ^      low      ^ hold
//
//
//   clk    l          h      h         l               h
//    x     x1         x1     x2        x2              x2
//

impl<I, S> Iterator for ClockPosEdge<I, S>
where
    I: Iterator<Item = ResetOrData<S>>,
    S: Digital,
{
    type Item = TimedSample<(ClockReset, S)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Init => {
                if let Some(data) = self.input.next() {
                    self.sample = data;
                    self.state = State::Hold;
                    self.next_time = self.time + self.period / 2;
                    Some(self.this_sample(clock(false)))
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::ClockLow => {
                self.state = State::Hold;
                self.time = self.next_time;
                self.next_time = self.time + self.period / 2;
                Some(self.this_sample(clock(false)))
            }
            State::Hold => {
                self.state = State::ClockHigh;
                self.time = self.next_time;
                self.next_time += 1;
                Some(self.this_sample(clock(true)))
            }
            State::ClockHigh => {
                if let Some(data) = self.input.next() {
                    self.sample = data;
                    self.state = State::ClockLow;
                } else {
                    self.state = State::TailStart;
                }
                self.time = self.next_time;
                self.next_time += self.period / 2 - 1;
                Some(self.this_sample(clock(true)))
            }
            State::TailStart => {
                self.state = State::TailEnd;
                self.time = self.next_time;
                self.next_time = self.time + self.period / 2;
                Some(self.this_sample(clock(false)))
            }
            State::TailEnd => {
                self.state = State::Done;
                self.time = self.next_time;
                Some(self.this_sample(clock(false)))
            }
            State::Done => None,
        }
    }
}

/// Creates a ClockPosEdge iterator that produces clock and reset signals along with the input samples.
///
/// See the [book] for examples of how to use this iterator adaptor.
pub fn clock_pos_edge<I, S>(input: I, period: u64) -> ClockPosEdge<I, S>
where
    I: Iterator<Item = ResetOrData<S>>,
    S: Digital,
{
    ClockPosEdge {
        input,
        sample: ResetOrData::Reset,
        state: State::Init,
        time: 0,
        next_time: 0,
        period,
    }
}

/// Extension trait to provide a `clock_pos_edge` method on iterators.
pub trait ClockPosEdgeExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    /// Creates a ClockPosEdge iterator that produces clock and reset signals along with the input samples.
    fn clock_pos_edge(self, period: u64) -> ClockPosEdge<<Self as IntoIterator>::IntoIter, Q>;
}

impl<I, Q> ClockPosEdgeExt<Q> for I
where
    I: IntoIterator<Item = ResetOrData<Q>>,
    Q: Digital,
{
    fn clock_pos_edge(self, period: u64) -> ClockPosEdge<Self::IntoIter, Q> {
        clock_pos_edge(self.into_iter(), period)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rhdl_bits::alias::*;

    fn expected() -> Vec<TimedSample<(ClockReset, b8)>> {
        vec![
            timed_sample(0, (clock_reset(clock(false), reset(false)), b8(0))),
            timed_sample(5, (clock_reset(clock(true), reset(false)), b8(0))),
            timed_sample(6, (clock_reset(clock(true), reset(false)), b8(1))),
            timed_sample(10, (clock_reset(clock(false), reset(false)), b8(1))),
            timed_sample(15, (clock_reset(clock(true), reset(false)), b8(1))),
            timed_sample(16, (clock_reset(clock(true), reset(false)), b8(2))),
            timed_sample(20, (clock_reset(clock(false), reset(false)), b8(2))),
            timed_sample(25, (clock_reset(clock(true), reset(false)), b8(2))),
            timed_sample(26, (clock_reset(clock(true), reset(false)), b8(3))),
            timed_sample(30, (clock_reset(clock(false), reset(false)), b8(3))),
            timed_sample(35, (clock_reset(clock(true), reset(false)), b8(3))),
            timed_sample(36, (clock_reset(clock(true), reset(false)), b8(3))),
            timed_sample(40, (clock_reset(clock(false), reset(false)), b8(3))),
            timed_sample(45, (clock_reset(clock(false), reset(false)), b8(3))),
        ]
    }

    #[test]
    fn test_clock_pos_edge_on_iterator() {
        let k = (0..4).map(b8).map(ResetOrData::Data).clock_pos_edge(10);
        let v = k.collect::<Vec<_>>();
        assert_eq!(v, expected());
    }

    #[test]
    fn test_clock_pos_edge_on_vector() {
        let k = vec![0, 1, 2, 3]
            .into_iter()
            .map(b8)
            .map(ResetOrData::Data)
            .clock_pos_edge(10);
        let v = k.collect::<Vec<_>>();
        assert_eq!(v, expected());
    }
}
