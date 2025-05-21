//! A FIFO-to-Ready/Valid buffer
//!
//!# Purpose
//! A FIFO-to-READY/VALID buffer is a highly specialized two element FIFO backed with a pair
//! of registers instead of a BRAM.  The idea is to allow two pipelines to be joined
//! where the supply side pipeline has "push" semantics (meaning that it is triggered
//! by some other process and produces data elements at it's own pace) and the demand
//! side pipeline has "pull" semantics - meaning that it is triggered at some rate that
//! moderates consumption of the data elements.
//!
//! The other way to conceptualize this is as a source and sink pair.  The supply side
//! pipeline is a data source - it produces data elements at it's own pace.  The demand
//! side pipeline is a data sink - it consumes data elements at it's own pace.  The push
//! pull buffer in general would be a FIFO, and indeed the Carloni papers show FIFOs as
//! the implementation of push-pull buffers.  However, in many cases, we only need minimal
//! buffering, and a pair of registers is sufficient.  
//!
//! Note that a normal synchronous FIFO as included in `rhdl` will not work here - if it has
//! only two slots in it, it cannot (by design) fill both slots.
//!
//! The design of the buffer uses a state to manage the fill level, and uses the extra
//! value of a fill level of 3 to indicate that the push-pull buffer is in an error condition
//! due to overflow of the input.  To make this buffer easy to use with Carloni
//! skid buffers, the output is presented as an `Option<T>`, and underflow is not possible,
//! as `None` is returned when the buffer is empty.
//!
//! Note that one use case for the [FIFOToStream] buffer is when we need to be able to
//! anticipate by a clock cycle that a pipeline is able to push data forward.  Here is
//! an example of the problem:
//!
#![doc = badascii!("
             +---------+               
ready  +-----+         +--------------+
                                       
                       +---+Some+-----+
data   +---------------+     T         
                                       
             +----+    +----+    +----+
clk     +----+    +----+    +----+     
                                       
             <--+t1+-->|<----+t2+----->
")]
//!
//! During the time `t1`, the downstream pipeline was available for us to push a new data item,
//! but our upstream process was not ready.  In time `t2`, the downstream pipeline is no longer
//! available, but the upstream process has produced a data item.  The upstream process must
//! stall and hold this output value until the downstream pipeline re-raises `ready` for a clock
//! cycle.
//!
//! With the [FIFOToStream] buffer, we have an addition invariant:
//!   - A FIFO that is not `full` on cycle `T` cannot be full on cycle `T+1` if we do not add data to it.
//!
//! This invariant means that the equivalent timing diagram with a [FIFOToStream] buffer
//! looks like this instead
//!
#![doc = badascii!("
       +-----+                         
full         +------------------------+
             :         :               
             :         +---+Some+-----+
data   +---------------+     T         
             :         :               
             +----+    +----+    +----+
clk     +----+    +----+    +----+     
                                       
             <--+t1+-->|<----+t2+----->
")]
//!
//! The important difference is that if the input stage is `!full`, as happens in interval `t1`, the
//! upstream pipeline is guaranteed to be able to run and produce a data item, even if it is in the
//! future.  Thus, if the upstream pipeline waits for the output to be `!full`, it can gaurantee that
//! one output item can be produced.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [FIFOToStream] buffer
//!
#![doc = badascii_formal!("
     +-+FifoToRV+---+     
 ?T  |              +?T   
+--->|data     data +---->
     |              |     
<----+full     ready|<---+
     |              |     
<----+error         |     
     +--------------+     
")]
//!
//!# Internals
//!
//! Effectively, the [FIFOToStream] buffer is simply a 2-element FIFO.  It is implemented with
//! a pair of registers and manual control logic, since the general FIFO logic does not handle
//! such small sizes well.
//!
//! Roughly the internal circuitry is equivalent to this:
//!
#![doc = badascii!(r"
 ?T  +----+FIFO+----+  ?T             
+--->|data      data+--------+--->    
     |              |        |is_some 
<----+full      next|<---+   +        
     |              |    +--+&        
     |              |        +  ready 
     +--------------+        +-------+
")]
//! The FIFO is advanced only if the output is `Some`, and if the `ready` signal is asserted.
//!
//! Note that there are no combinatorial paths between the inputs and
//! outputs, and a test is used to verify this property.
//!
//!# Example
//!
//! Here is an example of the interface.
//!
//!```
#![doc = include_str!("../../examples/fifo_to_stream.rs")]
//!```
//!
//! With an output.
//!
#![doc = include_str!("../../doc/fifo_to_rv.md")]
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    core::{dff, option::is_some},
    stream::StreamIO,
};

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
/// The [FIFOToStream] Buffer core.
///
/// `T` is the type of the data elements flowing in the pipeline.
pub struct FIFOToStream<T: Digital> {
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

impl<T: Digital> Default for FIFOToStream<T> {
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

/// Inputs to the [FIFOToStream] buffer
///
/// For inputs, the push pull buffer has a Option<T> input to combine the
/// write enable with the data signal, and provides a full signal back.
/// It is important that the full signal is not dependant on the consumer,
/// so that the pull-pull buffer isolates the producer from the consumer
/// and vice versa.
pub type In<T> = StreamIO<T>;

#[derive(PartialEq, Debug, Digital)]
/// Outputs from the [FIFOToStream] buffer
pub struct Out<T: Digital> {
    /// The consumers data
    pub data: Option<T>,
    /// The producers "Q is full" signal
    pub full: bool,
    /// An error flag to indicate that the core has
    /// overflowed.  This occurs if the producer attempts
    /// to write data when the FIFO is full.
    pub error: bool,
}

impl<T: Digital> SynchronousIO for FIFOToStream<T> {
    type I = StreamIO<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
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
    let mut o = Out::<T>::dont_care();
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

#[cfg(test)]
mod tests {
    use crate::rng::xorshift::XorShift128;

    use super::*;

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = FIFOToStream::<b16>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        // The buffer will manage items of 4 bits
        let uut = FIFOToStream::<b4>::default();
        // The test harness will include a consumer that
        // randomly pauses the upstream producer.
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let mut dest_rng = source_rng.clone();
        uut.run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = super::In::<b4>::dont_care();
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = !want_to_pause;
                // Decide if the producer will generate a data item
                let want_to_send = rand::random::<u8>() < 200;
                input.data = None;
                if !out.full && want_to_send {
                    input.data = source_rng.next();
                }
                if out.data.is_some() && input.ready {
                    assert_eq!(out.data, dest_rng.next());
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
