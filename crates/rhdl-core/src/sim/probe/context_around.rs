//! Context around an event
use std::collections::VecDeque;

enum State {
    Scanning,
    YieldingBefore,
    YieldingAfter,
    Done,
}

/// An iterator adaptor that yields items around occurrences of an event defined by a predicate.
pub struct AroundEvent<I, S, F> {
    iter: I,
    predicate: F,
    before: usize,
    after: usize,
    buffer: VecDeque<S>,
    state: State,
    remaining_after: usize,
}

impl<I, S, F> Iterator for AroundEvent<I, S, F>
where
    I: Iterator<Item = S>,
    F: FnMut(&S) -> bool,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Scanning => {
                for item in self.iter.by_ref() {
                    if (self.predicate)(&item) {
                        // Found the event! Switch to yielding before items
                        self.state = State::YieldingBefore;
                        self.buffer.push_back(item);
                        return self.buffer.pop_front();
                    }
                    self.buffer.push_back(item);
                    if self.buffer.len() > self.before {
                        self.buffer.pop_front();
                    }
                }
                self.state = State::Done;
                None
            }
            State::YieldingBefore => {
                if let Some(item) = self.buffer.pop_front() {
                    Some(item)
                } else {
                    self.state = State::YieldingAfter;
                    self.remaining_after = self.after;
                    self.next()
                }
            }
            State::YieldingAfter => {
                if self.remaining_after > 0 {
                    self.remaining_after -= 1;
                    if self.remaining_after == 0 {
                        self.state = State::Scanning;
                    }
                    if let Some(item) = self.iter.next() {
                        Some(item)
                    } else {
                        self.state = State::Done;
                        None
                    }
                } else {
                    self.state = State::Done;
                    None
                }
            }
            State::Done => None,
        }
    }
}

/// Extension trait to make it ergonomic
pub trait AroundEventExt: Iterator {
    /// Create an iterator that yields items around occurrences of an event defined by a predicate.
    ///
    /// # Arguments
    /// * `before` - Number of items to yield before the event
    /// * `after` - Number of items to yield after the event
    /// * `predicate` - Function that defines the event
    ///
    /// # Example
    /// ```
    ///# use rhdl_core::sim::probe::context_around::AroundEventExt;
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let result: Vec<_> = data
    ///     .iter()
    ///     .around_event(2, 3, |&&x| x == 5)
    ///     .copied()
    ///     .collect();
    /// assert_eq!(result, vec![3, 4, 5, 6, 7, 8]);
    /// ```
    fn around_event<F>(
        self,
        before: usize,
        after: usize,
        predicate: F,
    ) -> AroundEvent<Self, Self::Item, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> bool,
    {
        AroundEvent {
            iter: self,
            predicate,
            before,
            after,
            buffer: VecDeque::new(),
            state: State::Scanning,
            remaining_after: 0,
        }
    }
}

impl<I: Iterator> AroundEventExt for I {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_around_event() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Get 2 items before and 3 items after where item == 5
        let result: Vec<_> = data
            .iter()
            .cycle()
            .take(30)
            .around_event(2, 3, |&&x| x == 5)
            .copied()
            .collect();

        assert_eq!(
            result,
            [3, 4, 5, 6, 7, 8]
                .into_iter()
                .cycle()
                .take(18)
                .collect::<Vec<_>>()
        );
    }
}
