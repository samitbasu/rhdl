//! A Stream-to-FIFO buffer
//!
//!# Purpose
//! A Stream-to-FIFO buffer is a highly specialized two element
//! FIFO backed with a pair of registers instead of a BRAM.  
//! Note that any FIFO can be interfaced to a stream by
//! simply setting `ready = !full`.  The particular use of this
//! [StreamToFIFO] buffer is to minimize the number of resources
//! needed.  As it only requires a couple of registers, it is generally
//! far less resource intensive than a full FIFO.  But it is not
//! special in any other meaningful way, and a regular [crate::fifo::synchronous::SyncFIFO]
//! might be a better choice.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [StreamToFIFO] buffer
//!
#![doc = badascii_formal!("
     ++Stm2FIFO+--+     
 ?T  |            | ?T  
+--->|data    data+---->
     |            |     
<----+ready   next|<---+
     |            |     
     |       error+---->
     +------------+     
")]
//!
//!# Internals
//!
//! Effectively, the [StreamToFIFO] buffer is simply a 2-element FIFO.
//! It is implemented with a pair of registers and manual control logic,
//! since the general FIFO logic doesn't work with such small sizes.
//!
//! Roughly the internal circuitry is equivalent to this:
//!
#![doc = badascii!("
         ?T   +----+FIFO+----+ ?T  
+------------>|data      data+---->
  ready       |              |     
<-----+! <----+full      next|<---+
              +--------------+     
")]
//! The FIFO will signal that it is `ready` as long as it is not `full`.
//! The consumer can use the `next` signal to accept the current `data`
//! element.
//!
//! Note that there are no combinatorial paths between the inputs and
//! outputs, and a test is used to verify this property.
//!
//!# Example
//!
//! Here is an example of the buffer in action.
//!
//!```
#![doc = include_str!("../../examples/stream_to_fifo.rs")]
//!```
//!
//! With the output.
#![doc = include_str!("../../doc/rv_to_fifo.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::{dff, option::is_some};

/// A READY/VALID-to-FIFO converter is a highly specialized two element
/// FIFO backed with a pair of registers instead of a BRAM.  
#[derive(PartialEq, Digital, Default, Debug)]
#[doc(hidden)]
pub enum State {
    #[default]
    Empty,
    OneLoaded,
    TwoLoaded,
    Error,
}

#[derive(PartialEq, Debug, Clone, SynchronousDQ, Synchronous)]
/// The [StreamToFIFO] Buffer Core
///
/// `T` is the type of the data elements flowing in the pipeline.
pub struct StreamToFIFO<T: Digital> {
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

impl<T: Digital> Default for StreamToFIFO<T> {
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

#[derive(PartialEq, Debug, Digital)]
/// Inputs to the [StreamToFIFO] buffer
///
/// For inputs, we accept an `Option<T>` input from the ready/valid bus
/// and a next signal to acknowledge that data had been consumed.
/// The output is an `Option<T>` and a ready signal to provide backpressure.
/// This buffer cannot overflow, since it consumes incoming data only when
/// ready.  However, it can underflow if the receiver signals a next
/// when there is no data available.
pub struct In<T: Digital> {
    /// The data from the bus
    pub data: Option<T>,
    /// The next signal from the consumer
    pub next: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// Outputs from the [StreamToFIFO] buffer
pub struct Out<T: Digital> {
    /// The data to the consumer
    pub data: Option<T>,
    /// The ready signal to the producer
    pub ready: bool,
    /// An error flag to indicate that the core has
    /// underflowed.
    pub error: bool,
}

impl<T: Digital> SynchronousIO for StreamToFIFO<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
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
    let mut o = Out::<T>::dont_care();
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

#[cfg(test)]
mod tests {
    use rhdl::prelude::*;

    use crate::rng::xorshift::XorShift128;

    use super::StreamToFIFO;

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = StreamToFIFO::<b16>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        // The buffer will manage items of 4 bits
        let uut = StreamToFIFO::<b4>::default();
        // The test harness will include a consumer that
        // randomly pauses the upstream producer.
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let mut dest_rng = source_rng.clone();
        let mut source_datum = source_rng.next();
        uut.run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = super::In::<b4>::dont_care();
                let may_accept = rand::random::<u8>() > 150;
                let will_accept = may_accept & out.data.is_some();
                input.next = false;
                if will_accept {
                    assert_eq!(out.data, dest_rng.next());
                    input.next = true;
                }
                let will_offer = rand::random::<u8>() > 150;
                if will_offer {
                    input.data = source_datum;
                } else {
                    input.data = None;
                }
                let will_advance = will_offer & out.ready;
                if will_advance {
                    source_datum = source_rng.next();
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 100_000)
        .for_each(drop);
        Ok(())
    }
}
