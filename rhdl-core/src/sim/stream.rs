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
        .flat_map(|x| once((clock(false), x)).chain(once((clock(true), x))))
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
            timed_sample((clock_reset(clock(true), reset(true)), 0), 0),
            timed_sample((clock_reset(clock(false), reset(true)), 0), 10),
            timed_sample((clock_reset(clock(true), reset(false)), 1), 20),
            timed_sample((clock_reset(clock(false), reset(false)), 1), 30),
            timed_sample((clock_reset(clock(true), reset(false)), 2), 40),
            timed_sample((clock_reset(clock(false), reset(false)), 2), 50),
            timed_sample((clock_reset(clock(true), reset(false)), 3), 60),
            timed_sample((clock_reset(clock(false), reset(false)), 3), 70),
            timed_sample((clock_reset(clock(true), reset(false)), 4), 80),
            timed_sample((clock_reset(clock(false), reset(false)), 4), 90),
        ];
        assert_eq!(input, expected);
    }
}
