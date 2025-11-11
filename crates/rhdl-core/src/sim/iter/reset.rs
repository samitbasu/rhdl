//! An iterator wrapper that prepends reset pulses to an input iterator.
use crate::Digital;

/// An iterator that prepends reset pulses to an input iterator.
pub struct ResetWrapper<I> {
    reset_counter: usize,
    input: I,
}

impl<I> Iterator for ResetWrapper<I>
where
    I: Iterator,
    <I as Iterator>::Item: Digital,
{
    type Item = Option<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reset_counter > 0 {
            self.reset_counter -= 1;
            Some(None)
        } else {
            self.input.next().map(Some)
        }
    }
}

impl<I> Clone for ResetWrapper<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        ResetWrapper {
            input: self.input.clone(),
            reset_counter: self.reset_counter,
        }
    }
}

/// Creates a ResetWrapper that does not prepend any reset pulses.
pub fn without_reset<I>(input: I) -> ResetWrapper<I> {
    ResetWrapper {
        input,
        reset_counter: 0,
    }
}

/// Creates a ResetWrapper that prepends the given number of reset pulses.
pub fn with_reset<I>(input: I, pulse: usize) -> ResetWrapper<I>
where
    I: Iterator,
    <I as Iterator>::Item: Digital,
{
    ResetWrapper {
        input,
        reset_counter: pulse,
    }
}

/// Extension trait to provide `with_reset` and `without_reset` methods on iterators.
pub trait TimedStreamExt<Q>: IntoIterator + Sized
where
    Q: Digital,
{
    /// Creates a ResetWrapper that does not prepend any reset pulses.
    fn without_reset(self) -> ResetWrapper<<Self as IntoIterator>::IntoIter>;

    /// Creates a ResetWrapper that prepends the given number of reset pulses.
    fn with_reset(self, pulse: usize) -> ResetWrapper<<Self as IntoIterator>::IntoIter>;
}

impl<I, Q> TimedStreamExt<Q> for I
where
    I: IntoIterator<Item = Q>,
    Q: Digital,
{
    fn without_reset(self) -> ResetWrapper<I::IntoIter> {
        without_reset(self.into_iter())
    }

    fn with_reset(self, pulse: usize) -> ResetWrapper<I::IntoIter> {
        with_reset(self.into_iter(), pulse)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rhdl_bits::alias::*;

    #[test]
    fn test_stream_on_iterator() {
        let k = 0..10;
        let s = k.map(b8).without_reset();
        let v = s.collect::<Vec<_>>();
        assert_eq!(v, (0..10).map(b8).map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn test_stream_on_vector() {
        let k = vec![0, 1, 2, 3, 4];
        let s = k.into_iter().map(b8).without_reset();
        let v = s.collect::<Vec<_>>();
        assert_eq!(
            v,
            vec![0, 1, 2, 3, 4]
                .into_iter()
                .map(b8)
                .map(Some)
                .collect::<Vec<_>>()
        );
    }
}
