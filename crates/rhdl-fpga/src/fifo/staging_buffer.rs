//! FIFO Staging Buffer
//!
//! We want the read side of the FIFO interface to look like the stream interface used
//! for streams in RHDL.  This means it takes a ready/valid handshake and outputs data
//! when the handshake is successful (ready and valid are both high).  However,
//! the read logic of the FIFO is actually a push pipeline - when the read address
//! is incremented, the data at the new address is not available until the next clock cycle.
//! Thus, there is a pipeline delay between the time we want to read a value out of the
//! FIFO, and the time we have the data to feed to the output. This is equivalent to the
//! problem solved by the PipeWrapper stream.  
//!
//! In this core, we create the following widget:
#![doc = badascii!("
     +-------------------+    
+--->|reserve            | ?T 
 ?T  |           data_out+--->
+--->|data_in            |    
     |                   |    
<---+|full         ready |<--+
     |                   |    
     +-------------------+    
")]
//! The reserve input is used to reserve the next value in the FIFO.  When reserve is high,
//! the core will decrement the available buffer count.  This allows the FIFO read logic to
//! "reserve" a slot in the output FIFO.  Data is actually written when `data_in.is_some()`.
//! The `full` signal is used to indicate that the staging buffer is fully committed (no more
//! reservations are possible).  The `ready` signal is used to indicate that the data on the
//! output can be accepted.  The staging buffer holds the output until `data_out.is_some()`
//! and `ready` is high, at which point the read pointer is increemented.
//!
//! Implementation
//!
//! This core holds as state
//!
//! - The current fill count (between 0 and 2, inclusive)
//! - The current empty count (between 0 and 2, inclusive)
//! - The write pointer (either 0 or 1)
//! - The read pointer (either 0 or 1)
//! - Two registers of type <T> to hold the data
use badascii_doc::badascii;
use rhdl::prelude::*;

use crate::core::dff::DFF;

#[derive(PartialEq, Debug, Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// A staging buffer for the FIFO read logic.  This core allows us to decouple the
/// ready/valid handshake of the FIFO output from the push pipeline nature of the FIFO read logic.
pub struct FifoStagingBuffer<T: Digital> {
    fill_count: DFF<Bits<2>>,
    empty_count: DFF<Bits<2>>,
    write_pointer: DFF<bool>,
    read_pointer: DFF<bool>,
    data_0: DFF<T>,
    data_1: DFF<T>,
}

impl<T: Digital> Default for FifoStagingBuffer<T> {
    fn default() -> Self {
        Self {
            fill_count: DFF::new(bits(0)),
            empty_count: DFF::new(bits(2)),
            write_pointer: DFF::new(false),
            read_pointer: DFF::new(false),
            data_0: DFF::new(T::dont_care()),
            data_1: DFF::new(T::dont_care()),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Outputs from the staging buffer
pub struct Out<T: Digital> {
    /// The data output from the staging buffer.  This is `Some` when the staging buffer has valid data to output.
    pub data_out: Option<T>,
    /// The full signal indicating that no more reservations are possible
    pub full: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Inputs to the staging buffer
pub struct In<T: Digital> {
    /// The input data to be staged.  This is `Some` when there is new data to be staged.
    pub data_in: Option<T>,
    /// The reserve signal to reserve the next slot in the staging buffer.  This should be
    /// asserted for one clock cycle to reserve a slot in the FIFO.
    pub reserve: bool,
    /// The ready signal that indicates the downstream logic is ready to accept data.
    pub ready: bool,
}

impl<T: Digital> SynchronousIO for FifoStagingBuffer<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
    // Outputs only depend on the current state
    let mut o = Out::<T>::dont_care();
    o.full = q.empty_count == 0;
    let data_valid = q.fill_count != 0;
    o.data_out = if data_valid {
        if q.read_pointer {
            Some(q.data_1)
        } else {
            Some(q.data_0)
        }
    } else {
        None
    };
    let mut d = D::<T>::dont_care();
    let will_reserve = i.reserve && !o.full;
    // Latch the DFFs at their previous values.
    d.data_0 = q.data_0;
    d.data_1 = q.data_1;
    // We assume that the upstream caller reserved a slot for each
    // write.  Under that assumption, any write to the FIFO is unconditional.
    let mut will_write = false;
    if let Some(data) = i.data_in {
        if q.write_pointer {
            d.data_1 = data;
        } else {
            d.data_0 = data;
        }
        will_write = true;
    }
    // If we will write, then shuffle the write pointer.
    d.write_pointer = if will_write {
        !q.write_pointer
    } else {
        q.write_pointer
    };
    // We will read if the output is valid, and the downstream is ready
    let will_drain = data_valid && i.ready;
    // If we will read, then shuffle the read pointer.
    d.read_pointer = if will_drain {
        !q.read_pointer
    } else {
        q.read_pointer
    };
    // Update the fill count - add 1 if we will write, and subtract 1 if we will read.
    d.fill_count = q.fill_count + if will_write { 1 } else { 0 } - if will_drain { 1 } else { 0 };
    // Update the empty count - subtract 1 if we will reserve, and add 1 if we will read.
    d.empty_count =
        q.empty_count - if will_reserve { 1 } else { 0 } + if will_drain { 1 } else { 0 };
    (o, d)
}

#[cfg(test)]
mod tests {
    use crate::rng::xorshift::XorShift128;

    use super::*;
    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = FifoStagingBuffer::<Bits<8>>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        let uut = FifoStagingBuffer::<Bits<8>>::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xFF) as u128));
        let mut dest_rng = source_rng.clone();
        let mut delay_line = [false, false];
        uut.run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let ready = rand::random_bool(0.5);
                let mut input = In::<b8> {
                    data_in: None,
                    reserve: false,
                    ready,
                };
                if ready && let Some(dout) = out.data_out {
                    assert_eq!(dout, dest_rng.next().unwrap());
                }
                // Decide if we will accept data.
                let can_issue = !out.full && rand::random_bool(0.6);
                // Check that delay line says data needs to be provided
                if delay_line[1] {
                    input.data_in = Some(source_rng.next().unwrap());
                }
                delay_line[1] = delay_line[0];
                delay_line[0] = can_issue;
                input.reserve = can_issue;
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )?
        .take_while(|t| t.time < 100_000)
        .for_each(drop);
        Ok(())
    }
}
