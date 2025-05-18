//! A Credit-based Pipeline Wrapper
//!
//!# Purpose
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
//! Furthermore this design is invariant to the latency introduced by the pipeline.  It is irrelevant.  The
//! latency may even be variable.  Each output slot in the FIFO is reserved for a pending computation, and
//! credit tracking makes no assumptions about how long those reservations are held for.
//!
#![doc = badascii!(r"
                           +                                                       
  ?S +-+RV2FIFO+--+ ?S     |\                                                      
+--->|data    data+------->|1+ ?S ++Pipeline++ ?T +-+FIFO+----+ ?T +-+FIFO2RV+-+ ?T
     |            |        | +--->|in    out +--->|data   data+--->|data   data+-->
<----+ready   next|<+None+>|0+    | delay N  |    |           |    |           |   
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
//! Here is the schematic symbol for the [CreditPipelineWrapper].
//!
#![doc = badascii_formal!("
      +-+CreditWrapper+----+       
  ?S  |                    |  ?T   
+---->| data         data  +------>
      |                    |       
<-----+ ready        ready |<-----+
   ?S |                    |  ?T   
<-----+ to_pipe  from_pipe |<-----+
      +--------------------+       
")]
//!
//! It is understood that the pipline will start when fed `Some(S)` data
//! element, and will produce multiple [Option<T>] output elements.  The
//! number of outputs per input is a const generic argument to the core.
//! Also, the internal FIFO size is exposed, since knowledge of how big the
//! output FIFO will need to be is a design decision.
use crate::{
    core::{dff::DFF, option::unpack},
    fifo::synchronous::SyncFIFO,
    lid::{fifo_to_rv::FIFOToReadyValid, rv_to_fifo::ReadyValidToFIFO},
};
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

#[derive(Clone, Synchronous, SynchronousDQ)]
pub struct CreditWrapper<S: Digital + Default, T: Digital + Default, N: BitWidth> {
    in_buffer: ReadyValidToFIFO<S>,
    fifo: SyncFIFO<T, N>,
    out_buffer: FIFOToReadyValid<T>,
    counter: DFF<Bits<N>>,
}

impl<S: Digital + Default, T: Digital + Default, N: BitWidth> Default for CreditWrapper<S, T, N> {
    fn default() -> Self {
        Self {
            in_buffer: ReadyValidToFIFO::default(),
            fifo: SyncFIFO::default(),
            out_buffer: FIFOToReadyValid::default(),
            counter: DFF::new(Bits::<N>::MAX),
        }
    }
}

#[derive(PartialEq, Digital)]
/// Inputs for the [CreditWrapper]
pub struct In<S: Digital, T: Digital> {
    /// Input data for the pipeline
    data: Option<S>,
    /// Input ready signal for the downstream pipeline
    ready: bool,
    /// The values that come from the push-pipe
    from_pipe: Option<T>,
}

#[derive(PartialEq, Digital)]
/// Outputs from the [CreditWrapper]
pub struct Out<S: Digital, T: Digital> {
    /// Output data from the core
    data: Option<T>,
    /// Ready signal for the upstream pipeline
    ready: bool,
    /// To the push-pipe values
    to_pipe: Option<S>,
}

impl<S: Digital + Default, T: Digital + Default, N: BitWidth> SynchronousIO
    for CreditWrapper<S, T, N>
{
    type I = In<S, T>;
    type O = Out<S, T>;
    type Kernel = kernel<S, T, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<S: Digital + Default, T: Digital + Default, N: BitWidth>(
    _cr: ClockReset,
    i: In<S, T>,
    q: Q<S, T, N>,
) -> (Out<S, T>, D<S, T, N>) {
    let mut d = D::<S, T, N>::dont_care();
    // Is more data available to feed the pipeline
    let (s_tag, s_data) = unpack::<S>(q.in_buffer.data);
    // Is there a slot available?
    let is_slot_available = (q.counter > 0) && !q.out_buffer.full;
    // If the data is available and a slot is available
    // then feed it a new data element
    let will_accept = s_tag && is_slot_available;
    let mut o = Out::<S, T>::dont_care();
    o.to_pipe = if will_accept { Some(s_data) } else { None };
    o.ready = q.in_buffer.ready;
    o.data = q.out_buffer.data;
    d.in_buffer.next = will_accept;
    d.in_buffer.data = i.data;
    d.out_buffer.ready = i.ready;
    let (t_tag, t_data) = unpack::<T>(q.fifo.data);
    let will_unload = t_tag && !q.out_buffer.full;
    d.out_buffer.data = if will_unload { Some(t_data) } else { None };
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
        pipe::testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
        rng::xorshift::XorShift128,
    };

    pub mod delay {
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
            let (tag, data) = unpack::<b6>(q.stage_1);
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
        wrapper: CreditWrapper<b6, b4, U2>,
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
            wrapper: CreditWrapper::default(),
            sink: SinkFromFn::new(consume),
        };
        // Run a few samples through
        let input = repeat_n((), 205).stream_after_reset(1).clock_pos_edge(100);
        let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
        vcd.dump_to_file("credit.vcd")?;
        Ok(())
    }
}
