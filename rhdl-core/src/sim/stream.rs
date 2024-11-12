use std::iter::once;

use crate::{clock::clock, clock_reset, types::reset::reset, ClockReset, Digital, TimedSample};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum ResetData<T: Digital> {
    Reset,
    Data(T),
}

pub fn reset_pulse<T: Digital>(len: usize) -> impl Iterator<Item = ResetData<T>> {
    std::iter::repeat(ResetData::Reset).take(len)
}

pub fn stream<T: Digital>(input: impl Iterator<Item = T>) -> impl Iterator<Item = ResetData<T>> {
    input.map(ResetData::Data)
}

pub fn fill<T: Digital>(input: T) -> impl Iterator<Item = ResetData<T>> {
    std::iter::repeat(ResetData::Data(input))
}

pub fn clock_pos_edge<T: Digital>(
    stream: impl Iterator<Item = ResetData<T>>,
    period: u64,
) -> impl Iterator<Item = TimedSample<(ClockReset, T)>> {
    stream
        .flat_map(|x| once((clock(true), x)).chain(once((clock(false), x))))
        .enumerate()
        .map(move |(ndx, (clock, data))| match data {
            ResetData::Reset => TimedSample {
                value: (clock_reset(clock, reset(true)), T::init()),
                time: ndx as u64 * period,
            },
            ResetData::Data(data) => TimedSample {
                value: (clock_reset(clock, reset(false)), data),
                time: ndx as u64 * period,
            },
        })
}

pub struct Merge<A, B, S: Digital, T: Digital, F> {
    stream1: A,
    stream2: B,
    merge_fn: F,
    data1: Option<TimedSample<S>>,
    data2: Option<TimedSample<T>>,
    last1: S,
    last2: T,
}

pub fn merge<A, B, S: Digital, T: Digital, F>(
    mut stream1: A,
    mut stream2: B,
    merge_fn: F,
) -> Merge<A, B, S, T, F>
where
    A: Iterator<Item = TimedSample<S>>,
    B: Iterator<Item = TimedSample<T>>,
{
    let data1 = stream1.next();
    let data2 = stream2.next();
    Merge {
        data1,
        data2,
        stream1,
        stream2,
        merge_fn,
        last1: S::init(),
        last2: T::init(),
    }
}

impl<A, B, S: Digital, T: Digital, F: Fn(S, T) -> U, U: Digital> Iterator for Merge<A, B, S, T, F>
where
    A: Iterator<Item = TimedSample<S>>,
    B: Iterator<Item = TimedSample<T>>,
{
    type Item = TimedSample<U>;

    fn next(&mut self) -> Option<TimedSample<U>> {
        match (self.data1, self.data2) {
            (None, None) => None,
            (Some(d1), None) => {
                self.last1 = d1.value;
                let d1 = d1.map(|x| (self.merge_fn)(x, self.last2));
                self.data1 = self.stream1.next();
                Some(d1)
            }
            (None, Some(d2)) => {
                self.last2 = d2.value;
                let d2 = d2.map(|x| (self.merge_fn)(self.last1, x));
                self.data2 = self.stream2.next();
                Some(d2)
            }
            (Some(d1), Some(d2)) if d1.time < d2.time => {
                self.last1 = d1.value;
                let d1 = d1.map(|x| (self.merge_fn)(x, self.last2));
                self.data1 = self.stream1.next();
                Some(d1)
            }
            (Some(d1), Some(d2)) if d1.time > d2.time => {
                self.last2 = d2.value;
                let d2 = d2.map(|x| (self.merge_fn)(self.last1, x));
                self.data2 = self.stream2.next();
                Some(d2)
            }
            (Some(d1), Some(d2)) => {
                self.last1 = d1.value;
                self.last2 = d2.value;
                let d1 = d1.map(|x| (self.merge_fn)(x, d2.value));
                self.data1 = self.stream1.next();
                self.data2 = self.stream2.next();
                Some(d1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use crate::timed_sample;

    use super::*;

    #[test]
    fn test_merge_reset() {
        let rst = reset_pulse(4);
        let data = once(1);
        let merged = rst.chain(stream(data));
        let expected = vec![
            ResetData::Reset,
            ResetData::Reset,
            ResetData::Reset,
            ResetData::Reset,
            ResetData::Data(1),
        ];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_no_pulse() {
        let rst = reset_pulse(0);
        let data = once(1);
        let merged = rst.chain(stream(data));
        let expected = vec![ResetData::Data(1)];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_two_pulses() {
        let rst = reset_pulse(2)
            .chain(stream([1, 2, 3, 4].iter().copied()))
            .chain(reset_pulse(1));
        let expected = vec![
            ResetData::Reset,
            ResetData::Reset,
            ResetData::Data(1),
            ResetData::Data(2),
            ResetData::Data(3),
            ResetData::Data(4),
            ResetData::Reset,
        ];
        assert_eq!(rst.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_with_clock() {
        let rst = reset_pulse(1).chain(stream([1, 2, 3, 4].iter().copied()));
        let input = clock_pos_edge(rst, 10).collect::<Vec<_>>();
        let expected = vec![
            timed_sample(0, (clock_reset(clock(false), reset(true)), 0)),
            timed_sample(10, (clock_reset(clock(true), reset(true)), 0)),
            timed_sample(20, (clock_reset(clock(false), reset(false)), 1)),
            timed_sample(30, (clock_reset(clock(true), reset(false)), 1)),
            timed_sample(40, (clock_reset(clock(false), reset(false)), 2)),
            timed_sample(50, (clock_reset(clock(true), reset(false)), 2)),
            timed_sample(60, (clock_reset(clock(false), reset(false)), 3)),
            timed_sample(70, (clock_reset(clock(true), reset(false)), 3)),
            timed_sample(80, (clock_reset(clock(false), reset(false)), 4)),
            timed_sample(90, (clock_reset(clock(true), reset(false)), 4)),
        ];
        assert_eq!(input, expected);
    }

    #[test]
    fn test_merge_streams() {
        // Oh no!  Ghostbusters!
        let stream1: Vec<TimedSample<u8>> = vec![
            timed_sample(0, 0xa0),
            timed_sample(5, 0xa1),
            timed_sample(10, 0xa2),
        ];
        let stream2: Vec<TimedSample<u8>> = vec![
            timed_sample(1, 0xb1),
            timed_sample(3, 0xb2),
            timed_sample(6, 0xb3),
            timed_sample(10, 0xb4),
        ];
        let merged = merge(stream1.into_iter(), stream2.into_iter(), |a: u8, b: u8| {
            (a, b)
        });
        let stream_merged: Vec<TimedSample<(u8, u8)>> = vec![
            timed_sample(0, (0xa0, 0)),
            timed_sample(1, (0xa0, 0xb1)),
            timed_sample(3, (0xa0, 0xb2)),
            timed_sample(5, (0xa1, 0xb2)),
            timed_sample(6, (0xa1, 0xb3)),
            timed_sample(10, (0xa2, 0xb4)),
        ];
        assert_eq!(merged.collect::<Vec<_>>(), stream_merged);
    }
}
