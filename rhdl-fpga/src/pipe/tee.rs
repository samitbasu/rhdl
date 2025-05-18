//! Tee Pipe Core
//!
//!# Purpose
//!
//! A [TeePipe] Core takes a single pipeline as inputs
//! and yields two pipelines of outputs.  It is roughly
//! equivalent to `.unzip()` method on iterators.  The
//! [TeePipe] will merge backpressure from the two
//! destination pipes.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [TeePipe] core
//!
#![doc = badascii_formal!("
         +--+TeePipe+----+       
  ?(S,T) |               | ?S    
+------->|data    a.data +------>
         |               |       
 <-------+ready   a.ready|<-----+
         |               | ?T    
         |        b.data +------>
         |               |       
         |        b.ready|<-----+
         |               |       
         +---------------+          
")]
//!
//!# Internals
//!
//! The [TeePipe] contains a couple of buffers and
//! a combinatorial block to split the `Option<(S,T)>`
//! into `Option<S>` and `Option<T>`.
//!
#![doc = badascii!("
                                                      ++pack++      ++FIFO2RV+-+     
                         +unpck+++     +split+ S      |      |      |          | ?S  
       ++RVToFIFO++      |       |(S,T)|   .0+------->|in out+----->|data  data+---->
 ?(S,T)|          |?(S,T)|   data+---->|in   | T      |      |      |          |     
+----->|data  data+----->|in     |     |   .1+--+ +-->|tag   |  +---+full ready|<---+
       |          |      |    tag+-+   |     |  | |   +------+  |   |          |     
 <-----+ready next|<-+   |       | |   +-----+  | |             |   +----------+     
       |          |  |   +-------+ |            | |             |                    
       +----------+  |             v            | |             |                    
                     |      +----------+        | |   ++pack++  |   ++FIFO2RV+-+     
                     |   run| Control  |        | +   |      |  |   |          | ?T  
                     +------+          +----+   +---->|in out+-+v+->|data  data+---->
                            |      full|    |     +   |      |      |          |     
                            +----------+    +-----+-->|tag   |  OR+-+full ready|<---+
                                    ^                 +------+  +   |          |     
                                    |                           |   +----------+     
                                    +---------------------------+
")]
//!
//!# Example
//!
//! Here is an example of running the tee filter.
//!
//!```
#![doc = include_str!("../../examples/tee.rs")]
//!```
//!
//! With the resulting trace.
//!
#![doc = include_str!("../../doc/tee.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    core::option::{pack, unpack},
    lid::{fifo_to_rv::FIFOToReadyValid, rv_to_fifo::ReadyValidToFIFO},
};

#[derive(Debug, Clone, Synchronous, SynchronousDQ, Default)]
/// The [TeePipe] Core
///
/// This core takes a single pipe stream of type `(S,T)`, and connects to
/// two outgoing streams of type `S` and `T`.
pub struct TeePipe<S: Digital + Default, T: Digital + Default> {
    in_buffer: ReadyValidToFIFO<(S, T)>,
    s_buffer: FIFOToReadyValid<S>,
    t_buffer: FIFOToReadyValid<T>,
}

/// Input struct for the [TeePipe]
#[derive(PartialEq, Digital)]
pub struct In<S: Digital, T: Digital> {
    /// The input data for the [TeePipe]
    pub data: Option<(S, T)>,
    /// The downstream ready signal for the S pipe
    pub s_ready: bool,
    /// The downstream ready signal for the T pipe
    pub t_ready: bool,
}

/// Output struct for the [TeePipe]
#[derive(PartialEq, Digital)]
pub struct Out<S: Digital, T: Digital> {
    /// The output data for the S stream
    pub s_data: Option<S>,
    /// The output data for the T stream
    pub t_data: Option<T>,
    /// The upstream ready signal
    pub ready: bool,
}

impl<S: Digital + Default, T: Digital + Default> SynchronousIO for TeePipe<S, T> {
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
    let (tag, data) = unpack::<(S, T)>(q.in_buffer.data);
    let full = q.s_buffer.full || q.t_buffer.full;
    let tag = tag && !full;
    let s_val = pack::<S>(tag, data.0);
    let t_val = pack::<T>(tag, data.1);
    d.s_buffer.data = s_val;
    d.t_buffer.data = t_val;
    d.in_buffer.next = tag;
    d.in_buffer.data = i.data;
    d.s_buffer.ready = i.s_ready;
    d.t_buffer.ready = i.t_ready;
    let o = Out::<S, T> {
        s_data: q.s_buffer.data,
        t_data: q.t_buffer.data,
        ready: q.in_buffer.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use rhdl::core::SynchronousIO;

    use super::TeePipe;
    use super::*;
    use crate::pipe::testing::sink_from_fn::SinkFromFn;
    use crate::pipe::testing::source_from_fn::SourceFromFn;
    use crate::pipe::testing::utils::stalling;
    use crate::rng::xorshift::XorShift128;

    #[derive(Clone, Synchronous, SynchronousDQ)]
    struct TestFixture {
        source: SourceFromFn<(b4, b6)>,
        tee: TeePipe<b4, b6>,
        s_sink: SinkFromFn<b4>,
        t_sink: SinkFromFn<b6>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = ();
        type Kernel = kernel;
    }

    #[kernel]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
        let mut d = D::dont_care();
        d.tee.data = q.source;
        d.source = q.tee.ready;
        d.s_sink = q.tee.s_data;
        d.t_sink = q.tee.t_data;
        d.tee.s_ready = q.s_sink;
        d.tee.t_ready = q.t_sink;
        ((), d)
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| {
            let s = b4((x & 0xF) as u128);
            let t = b6(((x >> 8) & 0x3F) as u128);
            (s, t)
        });
        let mut c_rng = a_rng.clone();
        let mut d_rng = a_rng.clone();
        let a_rng = stalling(a_rng, 0.23);
        let consume_s = move |data| {
            if let Some(data) = data {
                let validation = c_rng.next().unwrap();
                assert_eq!(data, validation.0);
            }
            rand::random::<f64>() > 0.2
        };
        let consume_t = move |data| {
            if let Some(data) = data {
                let validation = d_rng.next().unwrap();
                assert_eq!(data, validation.1);
            }
            rand::random::<f64>() > 0.2
        };
        let uut = TestFixture {
            source: SourceFromFn::new(a_rng),
            tee: TeePipe::default(),
            s_sink: SinkFromFn::new(consume_s),
            t_sink: SinkFromFn::new(consume_t),
        };
        // Run a few samples through
        let input = repeat_n((), 10_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        uut.run_without_synthesis(input)?.for_each(drop);
        Ok(())
    }
}
