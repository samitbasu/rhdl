//! Zip Stream Core
//!
//!# Purpose
//!
//! A [Zip] Core takes 2 streams as inputs and yields a
//! single pipeline of outputs consisting of tuples formed
//! from the two input pipelines.  It is roughly equivalent to
//! the `.zip()` method on iterators.  The [Zip] propogates
//! backpressure up the two source pipes.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [Zip] core
//!
#![doc = badascii_formal!("
        +--+Zip+---------+        
  ?S    |                |        
+------>|a.data          |        
        |                | ?(S,T) 
 <------+a.ready     data+------> 
  ?T    |                |        
+------>|b.data     ready|<------+
        |                |        
 <------+b.ready         |        
        +----------------+        
")]
//!
//!# Internals
//!
//! The [Zip] uses input FIFOs to buffer incoming data elements
//! on each of the two upstream pipes, and then advances them to the
//! output FIFO when both are ready.  Otherwise, the control logic
//! is straightfoward, and purely combinatorial.
//!
#![doc = badascii!(r"
      ++RVToFIFO+--+    ++unpck+-+     +conct++                                        
  ?S  |            | ?S |        |  S  |      |      +-+pack+-+      ++FIFOToRV+       
+---->| data  data +--->|in   out+---->|.0    |      |        |?(S,T)|         | ?(S,T)
      |            |    |        |     |   out+----->|data out+----->|in    out+------>
 <----+ ready next |<+  |     tag+-+ +>|.1    |      |        |      |         |       
      |            | |  |        | | | |      |   +->|tag     |   +--+full  rdy|<-----+
      +------------+ |  +--------+ | | +------+   |  |        |   |  |         |       
                     |             +-+---------+  |  +--------+   |  +---------+       
      ++RVToFIFO+--+ +  ++unpck+-+   |         v  |               |                    
  ?T  |            | ?T |        | T |    +-------+-------+       |                    
+---->| data  data +--->|in   out+---+    |               |       |                    
      |            | +  |        |        |    Control    |<------+                    
 <----+ ready next |<+  |     tag+------->|               |                            
      |            | |  |        |        |               |                            
      +------------+ |  +--------+        +-+-------------+                            
                     |                      |                                          
                     +----------------------+                                          
")]
//!
//!# Example
//!
//! An example of using a [Zip] is here.
//!
//!```
#![doc = include_str!("../../examples/zip.rs")]
//!```
//!
//! With the resulting trace.
//!
#![doc = include_str!("../../doc/zip.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    core::option::{pack, unpack},
    lid::{fifo_to_rv::FIFOToReadyValid, rv_to_fifo::ReadyValidToFIFO},
};

#[derive(Debug, Clone, Synchronous, SynchronousDQ, Default)]
/// The [Zip] Core
///
/// This core takes two streams.  One of type
/// `S`, and one of type `T`, and generates a stream
/// of `(S,T)` elements.
pub struct Zip<S: Digital + Default, T: Digital + Default> {
    a_buffer: ReadyValidToFIFO<S>,
    b_buffer: ReadyValidToFIFO<T>,
    out_buffer: FIFOToReadyValid<(S, T)>,
}

#[derive(PartialEq, Digital)]
/// Input struct for the [Zip]
pub struct In<S: Digital, T: Digital> {
    /// Input data for the `a` stream
    pub a_data: Option<S>,
    /// Input data for the `b` stream
    pub b_data: Option<T>,
    /// Ready signal for the downstream
    pub ready: bool,
}

#[derive(PartialEq, Digital)]
/// Output struct for the [Zip]
pub struct Out<S: Digital, T: Digital> {
    /// Ready signal for the `a`` stream
    pub a_ready: bool,
    /// Ready signal for the `b` stream
    pub b_ready: bool,
    /// Output data containing the tuples
    pub data: Option<(S, T)>,
}

impl<S: Digital + Default, T: Digital + Default> SynchronousIO for Zip<S, T> {
    type I = In<S, T>;
    type O = Out<S, T>;
    type Kernel = kernel<S, T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<S: Digital + Default, T: Digital + Default>(
    _cr: ClockReset,
    i: In<S, T>,
    q: Q<S, T>,
) -> (Out<S, T>, D<S, T>) {
    let mut d = D::<S, T>::dont_care();
    d.a_buffer.data = i.a_data;
    d.b_buffer.data = i.b_data;
    let (tag_a, data_a) = unpack::<S>(q.a_buffer.data);
    let (tag_b, data_b) = unpack::<T>(q.b_buffer.data);
    let out_tag = tag_a && tag_b && !q.out_buffer.full;
    let out_data = pack::<(S, T)>(out_tag, (data_a, data_b));
    d.a_buffer.next = out_tag;
    d.b_buffer.next = out_tag;
    d.out_buffer.data = out_data;
    d.out_buffer.ready = i.ready;
    let o = Out::<S, T> {
        a_ready: q.a_buffer.ready,
        b_ready: q.b_buffer.ready,
        data: q.out_buffer.data,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use super::*;
    use crate::{
        rng::xorshift::XorShift128,
        stream::testing::{
            sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling,
        },
    };

    #[derive(Clone, Synchronous, SynchronousDQ)]
    struct TestFixture {
        a_source: SourceFromFn<b4>,
        b_source: SourceFromFn<b6>,
        zip: Zip<b4, b6>,
        sink: SinkFromFn<(b4, b6)>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = ();
        type Kernel = kernel;
    }

    #[kernel]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
        let mut d = D::dont_care();
        d.zip.a_data = q.a_source;
        d.zip.b_data = q.b_source;
        d.sink = q.zip.data;
        d.zip.ready = q.sink;
        d.a_source = q.zip.a_ready;
        d.b_source = q.zip.b_ready;
        ((), d)
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
        let b_rng = XorShift128::default().map(|x| b6(((x >> 8) & 0x3F) as u128));
        let mut c_rng = a_rng.clone().zip(b_rng.clone());
        let a_rng = stalling(a_rng, 0.23);
        let b_rng = stalling(b_rng, 0.15);
        let consume = move |data| {
            if let Some(data) = data {
                let validation = c_rng.next().unwrap();
                assert_eq!(data, validation);
            }
            rand::random::<f64>() > 0.2
        };
        let uut = TestFixture {
            a_source: SourceFromFn::new(a_rng),
            b_source: SourceFromFn::new(b_rng),
            zip: Zip::default(),
            sink: SinkFromFn::new(consume),
        };
        // Run a few samples through
        let input = repeat_n((), 10_000).with_reset(1).clock_pos_edge(100);
        uut.run_without_synthesis(input)?.for_each(drop);
        Ok(())
    }
}
