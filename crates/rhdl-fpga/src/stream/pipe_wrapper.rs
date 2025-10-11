//! Wrap a Pipe into a Stream
//!
//!# Purpose
//!
//! This core is used to take a pipeline with no backpressure, and interface it into a stream.
//! The backpressure is handled by an internal FIFO where output elements of the pipe are
//! allocated space (like a credit-based system in networking).  
//!
//!# Details
//!
//! The original Latency Insensitive Design work focused on stallable pipelines.  That
//! is to say, that if the `ready` signal was taken away (or equivalently, if the
//! downstream process asserted `stop`), then the entire pipeline would stall
//! until the `ready` signal was reasserted.  In the original papers, this was done via
//! a gated clock or a clock-enable signal that was used to either advance a given
//! stage in the pipeline or hold it in it's current state.  Roughly, something like
//! this:
#![doc = badascii!(r"
      +--+Pipeline+--+          
  ?S  |              | ?T        
+---->|in        out +---->     
      |              |          
      |     clk_en   |          
      +--------------+          
              ^                 
              |          ready  
              +----------------+
")]
//!
//! However, the idea of a `clk_en` line doesn't always fit with a pipeline.  For example,
//! a DRAM controller can be seen as a pipeline (where `S` is the address to read from and
//! `T` are the data elements read back, for example).  The DRAM controller is generally
//! not stallable.  And it is fair to assume that the controller requires you to read out the data
//! elements from the output once you have committed a certain transaction.  
//!
//! Furthermore, suppose that each item `S` injected into the pipeline produces `N` items of
//! type `T` on the output of the pipeline. Then when can a new item be injected?  If the pipeline
//! is opaque, then we can only keep track of how many pending items need to be written to the
//! output.
//!
//! The obvious answer is to include an output FIFO at the end of the pipeline to hold the
//! items as they are produced by the pipeline.  These can then be served to the downstream
//! process as it manages the `ready` signal.
//!
#![doc = badascii!(r"
      +--+Pipeline+--+     +--+FIFO+---+      
  ?S  |              | ?T  |           | ?T   
+---->|in        out +---->| in    out +----> 
      |              |     |           | ready
      |              |     |       next|<----+
      +--------------+     +-----------+      
")]
//!
//! Ignoring, temporarily the problem of underflow of the output FIFO, the bigger problem is the lack
//! of backpressure handling by the pipeline.  If the output FIFO is full, how do we stall the pipeline?
//! If it has no clock enable or other means of stalling, we are still in the same situation as before.
//!
//! The proposed solution in this core is to introduce a credit-based system.  A control core that
//! tracks the number of open slots in the output FIFO, and only dispatches as many items `S` such
//! that the output is guaranteed to fit in the output FIFO.  Each clock for which the `ready` signal
//! is asserted will release an additional credit to the controller, and each `S` item that is consumed
//! will require `N` credits to be available, where `N` is the number of `T` items produced by each `S`.
//!
//! Thus, backpressure is moved upstream of the pipeline.  The pipeline itself does not need to support
//! backpressure, since the controller will stop the inflow of data when there is insufficient credit
//! in the FIFO to start processing more data elements.
//!
//! Furthermore this design is invariant to the latency introduced by the pipeline.  It can even be variable.
//! Each output slot in the FIFO is reserved for a pending computation, and
//! credit tracking makes no assumptions about how long those reservations are held for.
//!
#![doc = badascii!(r"
                           +                                                       
  ?S +-+RV2FIFO+--+ ?S     |\                                                      
+--->|data    data+------->|1+ ?S ++Pipeline++ ?T +-+FIFO+----+ ?T +-+FIFO2RV+-+ ?T
 R<S>|            |        | +--->|in    out +--->|data   data+--->|data   data+-->
<----+ready   next|<+None+>|0+    | delay N  |    |           |    |           | R<T>  
     +------------+ |      |/     +----------+ +--+full   next|  +-+full  ready|<-+
                    |      +^    +---------+   |  +-----------+  | +-----------+   
                    |       +----+ Control |<--+            ^    |                 
                    +------------+         |<--------------+|+---+                 
                                 +--------++                |                      
                                          +-----------------+                      
")]
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [PipeWrapper].
//!
#![doc = badascii_formal!("
      +-+PipeWrapper+------+       
  ?S  |                    |  ?T   
+---->| data         data  +------>
 R<S> |                    | R<T>     
<-----+ ready        ready |<-----+
   ?S |                    |  ?T   
<-----+ to_pipe  from_pipe |<-----+
      +--------------------+       
")]
//!
//! It is understood that the pipline will start when fed `Some(S)` data
//! element, and will produce exactly one [Option<T>] output element at some
//! time in the future.  The internal FIFO size is exposed, since knowledge of how big the
//! output FIFO will need to be is a design decision.
//!
//!# Example
//!
//! An example of wrapping a pipeline with a [PipeWrapper] core
//! is here.
//!
//!```
#![doc = include_str!("../../examples/pipe_wrap.rs")]
//!```
//!
//! With the resulting trace.
//!
#![doc = include_str!("../../doc/pipe_wrap.md")]

use crate::{
    core::{dff::DFF, option::is_some},
    fifo::synchronous::SyncFIFO,
    stream::{fifo_to_stream::FIFOToStream, stream_to_fifo::StreamToFIFO},
};
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use super::Ready;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// [PipeWrapper] core for wrapping a pipeline into a stream
///
/// This core allows you to run a pipeline (that accepts no backpressure)
/// inside a stream.   An internal fifo with `N` address bits is used to
/// hold reserved slots for the output of the pipeline.  The input stream
/// carries elements of type `S` and the pipeline is assumed to produce elements
/// of type `T`.  This core assumes a 1-1 relationship, i.e., each `Some(S)` will
/// produce exactly one `Some(T)`.
pub struct PipeWrapper<S: Digital, T: Digital, N: BitWidth> {
    in_buffer: StreamToFIFO<S>,
    fifo: SyncFIFO<T, N>,
    out_buffer: FIFOToStream<T>,
    counter: DFF<Bits<N>>,
}

impl<S: Digital, T: Digital, N: BitWidth> Default for PipeWrapper<S, T, N> {
    fn default() -> Self {
        Self {
            in_buffer: StreamToFIFO::default(),
            fifo: SyncFIFO::default(),
            out_buffer: FIFOToStream::default(),
            counter: DFF::new(Bits::<N>::MAX),
        }
    }
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Inputs for the [PipeWrapper]
pub struct In<S: Digital, T: Digital> {
    /// Input data for the upstream
    pub data: Option<S>,
    /// Input ready signal for the downstream
    pub ready: Ready<T>,
    /// The values that come from the pipeline
    pub from_pipe: Option<T>,
}

#[derive(PartialEq, Clone, Copy, Digital)]
/// Outputs from the [PipeWrapper]
pub struct Out<S: Digital, T: Digital> {
    /// Output data for the downstream
    pub data: Option<T>,
    /// Ready signal for the upstream
    pub ready: Ready<S>,
    /// Data to feed the pipeline
    pub to_pipe: Option<S>,
}

impl<S: Digital, T: Digital, N: BitWidth> SynchronousIO for PipeWrapper<S, T, N> {
    type I = In<S, T>;
    type O = Out<S, T>;
    type Kernel = kernel<S, T, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<S: Digital, T: Digital, N: BitWidth>(
    _cr: ClockReset,
    i: In<S, T>,
    q: Q<S, T, N>,
) -> (Out<S, T>, D<S, T, N>) {
    let mut d = D::<S, T, N>::dont_care();
    // Is there a slot available?
    let is_slot_available = (q.counter > 0) && !q.out_buffer.full;
    // If the data is available and a slot is available
    // then feed it a new data element
    let mut o = Out::<S, T>::dont_care();
    o.to_pipe = None;
    let mut will_accept = false;
    if is_slot_available {
        // Is more data available to feed the pipeline?
        if let Some(s_data) = q.in_buffer.data {
            will_accept = true;
            o.to_pipe = Some(s_data);
        }
    }
    o.ready = q.in_buffer.ready;
    o.data = q.out_buffer.data;
    d.in_buffer.next = will_accept;
    d.in_buffer.data = i.data;
    d.out_buffer.ready = i.ready;
    let t_tag = is_some::<T>(q.fifo.data);
    let will_unload = t_tag && !q.out_buffer.full;
    d.fifo.next = will_unload;
    d.fifo.data = i.from_pipe;
    d.counter = match (will_accept, will_unload) {
        (false, false) => q.counter,
        (true, true) => q.counter,
        (true, false) => q.counter - 1,
        (false, true) => q.counter + 1,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use delay::DelayLine;

    use super::*;
    use crate::{
        core::{dff::DFF, option::pack, slice::lsbs},
        rng::xorshift::XorShift128,
        stream::testing::{
            sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling,
        },
    };

    pub mod delay {
        use crate::core::option::unpack;

        use super::*;
        #[derive(Clone, Synchronous, SynchronousDQ, Default)]
        pub struct DelayLine {
            stage_0: DFF<Option<b6>>,
            stage_1: DFF<Option<b6>>,
            stage_2: DFF<Option<b4>>,
        }

        impl SynchronousIO for DelayLine {
            type I = Option<b6>;
            type O = Option<b4>;
            type Kernel = kernel;
        }

        #[kernel]
        pub fn kernel(_cr: ClockReset, i: Option<b6>, q: Q) -> (Option<b4>, D) {
            let mut d = D::dont_care();
            d.stage_0 = i;
            d.stage_1 = q.stage_0;
            let (tag, data) = unpack::<b6>(q.stage_1, bits(0));
            let data = lsbs::<U4, U6>(data);
            d.stage_2 = pack::<b4>(tag, data);
            (q.stage_2, d)
        }
    }

    ///
    /// Here is a sketch of the internals:
    ///
    #[doc = badascii!(r"
+Source+-+    +Wrapper+-----+     +Sink+--+
|        | ?T |             | ?S  |       |
|    data+--->|data     data+---->|data   |
|        |    |             |     |       |
|   ready|<---+ready   ready|<----+ready  |
+--------+    +--+------+---+     +-------+
           +-----+      +----+             
         ?T|  +------------+ |?S           
           +->|in       out+-+             
              +------------+               
")]
    #[derive(Clone, Synchronous, SynchronousDQ)]
    struct TestFixture {
        source: SourceFromFn<b6>,
        delay: DelayLine,
        wrapper: PipeWrapper<b6, b4, U2>,
        sink: SinkFromFn<b4>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = ();
        type Kernel = kernel;
    }

    #[kernel]
    #[doc(hidden)]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
        let mut d = D::dont_care();
        d.wrapper.data = q.source;
        d.source = q.wrapper.ready;
        d.sink = q.wrapper.data;
        d.wrapper.ready = q.sink;
        d.delay = q.wrapper.to_pipe;
        d.wrapper.from_pipe = q.delay;
        ((), d)
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let b_rng = XorShift128::default().map(|x| b6(((x >> 8) & 0x3F) as u128));
        let mut c_rng = b_rng.clone();
        let b_rng = stalling(b_rng, 0.13);
        let consume = move |data| {
            if let Some(data) = data {
                let validation = lsbs::<U4, U6>(c_rng.next().unwrap());
                assert_eq!(data, validation);
            }
            rand::random::<f64>() > 0.2
        };
        let uut = TestFixture {
            source: SourceFromFn::new(b_rng),
            delay: DelayLine::default(),
            wrapper: PipeWrapper::default(),
            sink: SinkFromFn::new(consume),
        };
        // Run a few samples through
        let input = repeat_n((), 10_000).with_reset(1).clock_pos_edge(100);
        uut.run_without_synthesis(input)?.for_each(drop);
        Ok(())
    }
}
