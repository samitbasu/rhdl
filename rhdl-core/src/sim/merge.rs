use crate::{Digital, TimedSample};

pub struct Merge<A, B, S: Digital, T: Digital, F> {
    stream1: A,
    stream2: B,
    merge_fn: F,
    data1: Option<TimedSample<S>>,
    data2: Option<TimedSample<T>>,
    last1: S,
    last2: T,
}

impl<A, B, S, T, F> Clone for Merge<A, B, S, T, F>
where
    A: Clone,
    B: Clone,
    F: Clone,
    S: Clone + Digital,
    T: Clone + Digital,
{
    fn clone(&self) -> Self {
        Merge {
            stream1: self.stream1.clone(),
            stream2: self.stream2.clone(),
            merge_fn: self.merge_fn.clone(),
            data1: self.data1,
            data2: self.data2,
            last1: self.last1,
            last2: self.last2,
        }
    }
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
        last1: S::maybe_init(),
        last2: T::maybe_init(),
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
                self.data1 = None;
                Some(d1)
            }
            (None, Some(d2)) => {
                self.last2 = d2.value;
                let d2 = d2.map(|x| (self.merge_fn)(self.last1, x));
                self.data2 = None;
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

pub trait MergeExt<I, S, T>: IntoIterator + Sized {
    fn merge<F, U>(
        self,
        other: I,
        merge_fn: F,
    ) -> Merge<<Self as IntoIterator>::IntoIter, <I as IntoIterator>::IntoIter, S, T, F>
    where
        I: IntoIterator,
        F: Fn(S, T) -> U,
        S: Digital,
        T: Digital,
        U: Digital;
}

impl<I, O, S, T> MergeExt<O, S, T> for I
where
    I: IntoIterator<Item = TimedSample<S>>,
    O: IntoIterator<Item = TimedSample<T>>,
    S: Digital,
    T: Digital,
{
    fn merge<F, U>(
        self,
        other: O,
        merge_fn: F,
    ) -> Merge<<Self as IntoIterator>::IntoIter, <O as IntoIterator>::IntoIter, S, T, F>
    where
        F: Fn(S, T) -> U,
        U: Digital,
    {
        merge(self.into_iter(), other.into_iter(), merge_fn)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::once;

    use crate::{
        clock::clock,
        clock_reset,
        sim::{clock_pos_edge::ClockPosEdgeExt, stream::TimedStreamExt, ResetOrData},
        timed_sample,
        types::reset::reset,
    };

    use super::*;

    #[test]
    fn test_merge_reset() {
        let merged = once(1).stream_after_reset(4);
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(1),
        ];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_no_pulse() {
        let merged = once(1).stream();
        let expected = vec![ResetOrData::Data(1)];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_two_pulses() {
        let part_a = [1, 2, 3, 4].into_iter().stream_after_reset(2);
        let part_b = [].into_iter().stream_after_reset(1);
        let rst = part_a.chain(part_b);
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(1),
            ResetOrData::Data(2),
            ResetOrData::Data(3),
            ResetOrData::Data(4),
            ResetOrData::Reset,
        ];
        assert_eq!(rst.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_with_clock() {
        let input = [1, 2, 3, 4]
            .into_iter()
            .stream_after_reset(2)
            .clock_pos_edge(10)
            .collect::<Vec<_>>();
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
        let merged = stream1
            .merge(stream2, |a: u8, b: u8| (a, b))
            .collect::<Vec<_>>();
        let stream_merged: Vec<TimedSample<(u8, u8)>> = vec![
            timed_sample(0, (0xa0, 0)),
            timed_sample(1, (0xa0, 0xb1)),
            timed_sample(3, (0xa0, 0xb2)),
            timed_sample(5, (0xa1, 0xb2)),
            timed_sample(6, (0xa1, 0xb3)),
            timed_sample(10, (0xa2, 0xb4)),
        ];
        assert_eq!(merged, stream_merged);
    }
}
