use crate::Digital;

use super::ResetOrData;

pub struct Stream<I> {
    reset_counter: usize,
    input: I,
}

impl<I> Iterator for Stream<I>
where
    I: Iterator,
    <I as Iterator>::Item: Digital,
{
    type Item = ResetOrData<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reset_counter > 0 {
            self.reset_counter -= 1;
            Some(ResetOrData::Reset)
        } else {
            self.input.next().map(ResetOrData::Data)
        }
    }
}

impl<I> Clone for Stream<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Stream {
            input: self.input.clone(),
            reset_counter: self.reset_counter,
        }
    }
}

pub fn stream<I>(input: I) -> Stream<I> {
    Stream {
        input,
        reset_counter: 0,
    }
}

pub fn stream_after_reset<I>(input: I, pulse: usize) -> impl Iterator<Item = ResetOrData<I::Item>>
where
    I: Iterator,
    <I as Iterator>::Item: Digital,
{
    Stream {
        input,
        reset_counter: pulse,
    }
}

pub trait TimedStreamExt<Q>: Iterator
where
    Q: Digital,
{
    fn stream(self) -> impl Iterator<Item = ResetOrData<Q>>;

    fn stream_after_reset(self, pulse: usize) -> impl Iterator<Item = ResetOrData<Q>>;
}

impl<I, Q> TimedStreamExt<Q> for I
where
    I: Iterator<Item = Q>,
    Q: Digital,
{
    fn stream(self) -> impl Iterator<Item = ResetOrData<Q>> {
        stream(self)
    }

    fn stream_after_reset(self, pulse: usize) -> impl Iterator<Item = ResetOrData<Q>> {
        stream_after_reset(self, pulse)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_stream_on_iterator() {
        let k = 0..10;
        let s = k.stream();
        let v = s.collect::<Vec<_>>();
        assert_eq!(v, (0..10).map(ResetOrData::Data).collect::<Vec<_>>());
    }
}
