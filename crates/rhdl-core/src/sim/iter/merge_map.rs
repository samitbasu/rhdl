//! Merge two timed streams into one and apply a mapping function.
use crate::{Digital, TimedSample};

/// An iterator that merges two timed streams into one, using a provided merging function.
///
/// Generally, you will create this with the `merge_map` method on the `MergeMapExt` trait.
pub struct MergeMap<A, B, S: Digital, T: Digital, F> {
    stream1: A,
    stream2: B,
    merge_fn: F,
    data1: Option<TimedSample<S>>,
    data2: Option<TimedSample<T>>,
    last1: S,
    last2: T,
}

impl<A, B, S, T, F> Clone for MergeMap<A, B, S, T, F>
where
    A: Clone,
    B: Clone,
    F: Clone,
    S: Clone + Digital,
    T: Clone + Digital,
{
    fn clone(&self) -> Self {
        MergeMap {
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

/// Creates a Merge iterator that merges two timed streams using the provided merging function.
pub fn merge_map<A, B, S: Digital, T: Digital, F>(
    mut stream1: A,
    mut stream2: B,
    merge_fn: F,
) -> MergeMap<A, B, S, T, F>
where
    A: Iterator<Item = TimedSample<S>>,
    B: Iterator<Item = TimedSample<T>>,
{
    let data1 = stream1.next();
    let data2 = stream2.next();
    MergeMap {
        data1,
        data2,
        stream1,
        stream2,
        merge_fn,
        last1: S::dont_care(),
        last2: T::dont_care(),
    }
}

impl<A, B, S: Digital, T: Digital, F: Fn(S, T) -> U, U: Digital> Iterator
    for MergeMap<A, B, S, T, F>
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

/// Extension trait to provide a `merge_map` method on iterators.
pub trait MergeMapExt<I, S, T>: IntoIterator + Sized {
    /// Creates a Merge iterator that merges two timed streams using the provided merging function.
    fn merge_map<F, U>(
        self,
        other: I,
        merge_fn: F,
    ) -> MergeMap<<Self as IntoIterator>::IntoIter, <I as IntoIterator>::IntoIter, S, T, F>
    where
        I: IntoIterator,
        F: Fn(S, T) -> U,
        S: Digital,
        T: Digital,
        U: Digital;
}

impl<I, O, S, T> MergeMapExt<O, S, T> for I
where
    I: IntoIterator<Item = TimedSample<S>>,
    O: IntoIterator<Item = TimedSample<T>>,
    S: Digital,
    T: Digital,
{
    fn merge_map<F, U>(
        self,
        other: O,
        merge_fn: F,
    ) -> MergeMap<<Self as IntoIterator>::IntoIter, <O as IntoIterator>::IntoIter, S, T, F>
    where
        F: Fn(S, T) -> U,
        U: Digital,
    {
        merge_map(self.into_iter(), other.into_iter(), merge_fn)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect_file;
    use rhdl_bits::alias::*;
    use std::iter::once;

    use crate::sim::extension::*;
    use crate::{sim::ResetOrData, timed_sample};

    use super::*;

    #[test]
    fn test_merge_reset() {
        let merged = once(b8(1)).with_reset(4);
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(b8(1)),
        ];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_no_pulse() {
        let merged = once(b8(1)).without_reset();
        let expected = vec![ResetOrData::Data(b8(1))];
        assert_eq!(merged.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_two_pulses() {
        let part_a = [1, 2, 3, 4].into_iter().map(b8).with_reset(2);
        let part_b = [].into_iter().with_reset(1);
        let rst = part_a.chain(part_b);
        let expected = vec![
            ResetOrData::Reset,
            ResetOrData::Reset,
            ResetOrData::Data(b8(1)),
            ResetOrData::Data(b8(2)),
            ResetOrData::Data(b8(3)),
            ResetOrData::Data(b8(4)),
            ResetOrData::Reset,
        ];
        assert_eq!(rst.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn test_merge_reset_with_clock() {
        let input = [1, 2, 3, 4]
            .into_iter()
            .map(b8)
            .with_reset(2)
            .clock_pos_edge(10)
            .collect::<Vec<_>>();
        let expected = expect_file!["merge_reset.expect"];
        expected.assert_debug_eq(&input);
    }

    #[test]
    fn test_merge_streams() {
        // Oh no!  Ghostbusters!
        let stream1: Vec<TimedSample<b8>> = vec![
            timed_sample(0, b8(0xa0)),
            timed_sample(5, b8(0xa1)),
            timed_sample(10, b8(0xa2)),
        ];
        let stream2: Vec<TimedSample<b8>> = vec![
            timed_sample(1, b8(0xb1)),
            timed_sample(3, b8(0xb2)),
            timed_sample(6, b8(0xb3)),
            timed_sample(10, b8(0xb4)),
        ];
        let merged = stream1
            .merge_map(stream2, |a: b8, b: b8| (a, b))
            .collect::<Vec<_>>();
        let stream_merged: Vec<TimedSample<(b8, b8)>> = vec![
            timed_sample(0, (b8(0xa0), b8(0))),
            timed_sample(1, (b8(0xa0), b8(0xb1))),
            timed_sample(3, (b8(0xa0), b8(0xb2))),
            timed_sample(5, (b8(0xa1), b8(0xb2))),
            timed_sample(6, (b8(0xa1), b8(0xb3))),
            timed_sample(10, (b8(0xa2), b8(0xb4))),
        ];
        assert_eq!(merged, stream_merged);
    }
}
