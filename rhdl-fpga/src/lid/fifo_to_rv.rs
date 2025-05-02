use rhdl::prelude::*;

use crate::core::{dff, option::is_some};

/// A FIFO-to-READY/VALID buffer is a highly specialized two element FIFO backed with a pair
/// of registers instead of a BRAM.  The idea is to allow two pipelines to be joined
/// where the supply side pipeline has "push" semantics (meaning that it is triggered
/// by some other process and produces data elements at it's own pace) and the demand
/// side pipeline has "pull" semantics - meaning that it is triggered at some rate that
/// moderates consumption of the data elements.
///
/// The other way to conceptualize this is as a source and sink pair.  The supply side
/// pipeline is a data source - it produces data elements at it's own pace.  The demand
/// side pipeline is a data sink - it consumes data elements at it's own pace.  The push
/// pull buffer in general would be a FIFO, and indeed the Carloni papers show FIFOs as
/// the implementation of push-pull buffers.  However, in many cases, we only need minimal
/// buffering, and a pair of registers is sufficient.  
///
/// Note that a normal synchronous FIFO as included in `rhdl` will not work here - if it has
/// only two slots in it, it cannot (by design) fill both slots.
///
/// The design of the buffer uses a state to manage the fill level, and uses the extra
/// value of a fill level of 3 to indicate that the push-pull buffer is in an error condition
/// due to overflow of the input.  To make this buffer easy to use with Carloni
/// skid buffers, the output is presented as an `Option<T>`, and underflow is not possible,
/// as `None` is returned when the buffer is empty.
///
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
    state: dff::DFF<State>,
    /// The 0 slot of the buffer,
    zero_slot: dff::DFF<T>,
    /// The 1 slot of the buffer,
    one_slot: dff::DFF<T>,
    /// Where to write next item - in this case
    /// we use false for zero and true for one
    write_slot: dff::DFF<bool>,
    /// Where to read next item
    read_slot: dff::DFF<bool>,
}

impl<T: Digital> Default for U<T> {
    fn default() -> Self {
        Self {
            state: dff::DFF::default(),
            zero_slot: dff::DFF::new(T::dont_care()),
            one_slot: dff::DFF::new(T::dont_care()),
            write_slot: dff::DFF::default(),
            read_slot: dff::DFF::default(),
        }
    }
}

/// For inputs, the push pull buffer has a Option<T> input to combine the
/// write enable with the data signal, and provides a full signal back.
/// It is important that the full signal is not dependant on the consumer,
/// so that the pull-pull buffer isolates the producer from the consumer
/// and vice versa.
#[derive(PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    // The producers data and write enable
    pub data: Option<T>,
    // The consumers "stop/ready" signal
    pub ready: bool,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    // The consumers data
    pub data: Option<T>,
    // The producers "Q is full" signal
    pub full: bool,
    // An error flag to indicate that the core has
    // overflowed.
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
    let will_write = is_some::<T>(i.data);
    let can_read = i.ready;
    // Update the state machine
    d.state = match q.state {
        State::Empty => {
            if will_write {
                State::OneLoaded
            } else {
                State::Empty
            }
        }
        State::OneLoaded => match (will_write, can_read) {
            (false, false) => State::OneLoaded, // No change
            (true, false) => State::TwoLoaded,  // Producer wants to write
            (false, true) => State::Empty, // Consumer can read, and we have valid data, so we will be empty.
            (true, true) => State::OneLoaded, // Consumer can read, we have valid data, and producer wants to write.
        },
        State::TwoLoaded => {
            // Any write in this state is an error
            if will_write {
                State::Error
            } else if can_read {
                State::OneLoaded
            } else {
                State::TwoLoaded
            }
        }
        State::Error => State::Error,
    };
    // If we will write on this cycle, then copy the
    // data into the appropriate slot and then switch
    // buffers.  The buffers are otherwise unchanged.
    d.zero_slot = q.zero_slot;
    d.one_slot = q.one_slot;
    if let Some(data) = i.data {
        if !q.write_slot {
            d.zero_slot = data;
        } else {
            d.one_slot = data;
        }
    }
    let next_item = can_read && q.state != State::Empty && q.state != State::Error;
    // Toggle the read and write slots.
    d.write_slot = will_write ^ q.write_slot;
    d.read_slot = next_item ^ q.read_slot;
    // The output is set to void if we are empty, otherwise
    // the contents of the designated read slot
    let mut o = O::<T>::dont_care();
    if q.state == State::Empty {
        o.data = None
    } else if !q.read_slot {
        o.data = Some(q.zero_slot);
    } else {
        o.data = Some(q.one_slot);
    };
    o.full = q.state == State::TwoLoaded;
    o.error = q.state == State::Error;
    (o, d)
}
