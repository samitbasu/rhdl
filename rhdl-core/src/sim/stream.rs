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

pub fn stream_after_reset<I>(input: I, pulse: usize) -> Stream<I>
where
    I: Iterator,
    <I as Iterator>::Item: Digital,
{
    Stream {
        input,
        reset_counter: pulse,
    }
}

pub trait TimedStreamExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    fn stream(self) -> Stream<<Self as IntoIterator>::IntoIter>;

    fn stream_after_reset(self, pulse: usize) -> Stream<<Self as IntoIterator>::IntoIter>;
}

impl<I, Q> TimedStreamExt<Q> for I
where
    I: IntoIterator<Item = Q>,
    Q: Digital,
{
    fn stream(self) -> Stream<I::IntoIter> {
        stream(self.into_iter())
    }

    fn stream_after_reset(self, pulse: usize) -> Stream<I::IntoIter> {
        stream_after_reset(self.into_iter(), pulse)
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

    #[test]
    fn test_stream_on_vector() {
        let k = vec![0, 1, 2, 3, 4];
        let s = k.stream();
        let v = s.collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![0, 1, 2, 3, 4]
                .into_iter()
                .map(ResetOrData::Data)
                .collect::<Vec<_>>()
        );
    }
}
