//! A Lazy Random Stream Source
//!
//! This source stream generates a sequence of
//! 32-bit pseudo-random numbers with a probability
//! of "sleeping" after generating each one.  This
//! mimics a stalling source.  Note that this is
//! completely synthesizable.
//!
//!# Schematic Symbol
//! The lazy stream is very easy to use:
#![doc = badascii_formal!(r"
++LazyRng+--+         
|           | ?b32    
|      data +-------->
|           |         
|           | R<b32>        
|     ready |<-------+
|           |         
+-----------+         
")]
//! If you need something other than a [b32] output,
//! then use the [crate::stream::map::Map] to map
//! the stream to a different type.
//!
//!# Internal Details
//!
//! The [LazyRng] core is simply a [FIFOFiller]
//! coupled to a [FIFOToStream] core that implements
//! the stream interface.
//!
//! Roughly:
//!
#![doc = badascii!(r"
++FIFOFiller++             ++FifoToStrm+--+     
|            | ?b32        |              + ?b32
|      data  +------------>|data     data +---->
|            |             |              | R<b32>    
|            |        +----+full     ready|<---+
|       full |<-------+    |              |     
|            |             +error         |     
+------------+             +--------------+     
")]
//! The probability of sleeping, and the duration
//! of the sleep are controlled at construction time
//! and are passed to the constructor of the [FIFOFiller]
//! core.
//!# Example
//!
//! Here is an example of generating a bursty stream of
//! random values.
//!
//!```
#![doc = include_str!("../../../examples/lazy_rng.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../../doc/lazy_rng.md")]
//!

use badascii_doc::{badascii, badascii_formal};

use crate::{
    fifo::testing::filler::FIFOFiller,
    stream::{fifo_to_stream::FIFOToStream, Ready},
};
use rhdl::prelude::*;

#[derive(Clone, Synchronous, SynchronousDQ, Default)]
/// Lazy, bursty random number generator as
/// a stream.
pub struct LazyRng {
    filler: FIFOFiller<32>,
    buffer: FIFOToStream<b32>,
}

impl LazyRng {
    /// Construct a new [LazyRng] with controlled sleep
    /// durations and write probabilities
    pub fn new(sleep_len: u8, write_probability: f32) -> Self {
        Self {
            filler: FIFOFiller::new(sleep_len, write_probability),
            buffer: FIFOToStream::default(),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Input for the [LazyRng] the `ready` signal
/// provides backpressure.
pub struct In {
    /// Ready signal from downstream
    pub ready: Ready<b32>,
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Data from the [LazyRng] stream.
/// Will be [None] if the source is stalling.
pub struct Out {
    /// RNG to flow to downstream
    pub data: Option<b32>,
}

impl SynchronousIO for LazyRng {
    type I = In;
    type O = Out;
    type Kernel = kernel;
}

#[kernel]
#[doc(hidden)]
pub fn kernel(_cr: ClockReset, i: In, q: Q) -> (Out, D) {
    let mut d = D::dont_care();
    d.filler.full = q.buffer.full;
    d.buffer.data = q.filler.data;
    d.buffer.ready = i.ready;
    let o = Out {
        data: q.buffer.data,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use crate::stream::ready;

    use super::*;

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let input = (0..)
            .map(|_| rand::random_bool(0.8))
            .map(|r| In { ready: ready(r) })
            .with_reset(1)
            .clock_pos_edge(100)
            .take(1000);
        let uut = LazyRng::default();
        let vcd = uut.run(input).collect::<VcdFile>();
        vcd.dump_to_file("lazy_rng.vcd")?;
        Ok(())
    }
}
