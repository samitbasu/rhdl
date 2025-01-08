use rhdl::prelude::*;

use crate::core::{dff, option::is_some};

/// A READY/VALID-to-FIFO converter is a highly specialized two element
/// FIFO backed with a pair of registers instead of a BRAM.  
#[derive(PartialEq, Digital, Default, Debug)]
pub enum State {
    #[default]
    Empty,
    OneLoaded,
    TwoLoaded,
    Error,
}

#[derive(PartialEq, Debug, Clone, SynchronousDQ, Synchronous)]
pub struct U<T: Digital> {
    /// The state of the buffer
    state: dff::U<State>,
    /// The 0 slot of the buffer,
    zero_slot: dff::U<T>,
    /// The 1 slot of the buffer,
    one_slot: dff::U<T>,
    /// Where to write next item - in this case
    /// we use false for zero and true for one
    write_slot: dff::U<bool>,
    /// Where to read next item
    read_slot: dff::U<bool>,
}

impl<T: Digital> Default for U<T> {
    fn default() -> Self {
        Self {
            state: dff::U::default(),
            zero_slot: dff::U::new(T::dont_care()),
            one_slot: dff::U::new(T::dont_care()),
            write_slot: dff::U::default(),
            read_slot: dff::U::default(),
        }
    }
}

/// For inputs, we accept an Option<T> input from the ready/valid bus
/// and a next signal to acknowledge that data had been consumed.
/// The output is an Option<T> and a ready signal to provide backpressure.
/// This buffer cannot overflow, since it consumes incoming data only when
/// ready.  However, it can underflow if the receiver signals a next
/// when there is no data available.
#[derive(PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    // The data from the bus
    pub data: Option<T>,
    // The next signal from the consumer
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    // The data to the consumer
    pub data: Option<T>,
    // The ready signal to the producer
    pub ready: bool,
    // An error flag to indicate that the core has
    // underflow-ed.
    pub error: bool,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = kernel<T>;
}

#[kernel]
pub fn kernel<T: Digital>(_cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let will_read = i.next;
    let can_write = is_some::<T>(i.data);
    // Update the state machine
    d.state = match q.state {
        State::Empty => {
            if can_write {
                State::OneLoaded
            } else if will_read {
                State::Error
            } else {
                State::Empty
            }
        }
        State::OneLoaded => match (can_write, will_read) {
            (false, false) => State::OneLoaded, // No change
            (true, false) => State::TwoLoaded, // Producer wants to write, consumer does not want to read
            (false, true) => State::Empty, // Consumer can read, and we have valid data, so we will be empty.
            (true, true) => State::OneLoaded, // Consumer can read, we have valid data, and producer wants to write.
        },
        State::TwoLoaded => {
            if will_read {
                State::OneLoaded
            } else {
                State::TwoLoaded
            }
        }
        State::Error => State::Error,
    };
    let write_is_allowed = q.state != State::TwoLoaded && q.state != State::Error;
    // Decide if we will write on this clock cycle
    let will_write = can_write && write_is_allowed;
    d.zero_slot = q.zero_slot;
    d.one_slot = q.one_slot;
    if let Some(data) = i.data {
        if will_write {
            if q.write_slot {
                d.one_slot = data;
            } else {
                d.zero_slot = data;
            }
        }
    }
    d.write_slot = will_write ^ q.write_slot;
    d.read_slot = will_read ^ q.read_slot;
    let mut o = O::<T>::dont_care();
    if q.state == State::Empty {
        o.data = None;
    } else if !q.read_slot {
        o.data = Some(q.zero_slot);
    } else {
        o.data = Some(q.one_slot);
    }
    o.ready = write_is_allowed;
    o.error = q.state == State::Error;
    (o, d)
}
