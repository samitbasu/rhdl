//! AXI Stream to RHDL Stream adapter
//!
//! This core is a very lightweight shim that
//! presents an AXI Stream interface on one side
//! and a RHDL Stream interface on the other side.
//! To be spec compliant, this core includes a
//! buffer on the AXI side to ensure there are no
//! combinatorial pathways from the AXI bus to the
//! RHDL stream-side logic.
//!
//!# Schematic Symbol
//!
#![doc = badascii_formal!(r"
      +----+AXI2RHDL+-----+       
      |  AXI     :  RHDL  |       
  T   |          :        | ?T    
+---->| tdata    :  data  +------>
      |          :        |       
+---->| tvalid   :        |       
      |          :        | R<T>      
<-----+ tready   :  ready |<-----+
      +-------------------+       
")]
//!
//!# Internal details
//!
//! The core is simple.  It simply [pack]'s the
//! data and the valid flag into an [Option], and
//! forwards the `ready` signal.  To isolate the
//! combinatorial logic from the bus, we also
//! use a [CarloniBuffer] on the input.
//!
#![doc = badascii!(r"
                                               ++pack++      
 tdata       +-----+Carloni+-------+   tdata   |      |      
+----------->| data_in    data_out +---------->|data  | data 
tready       |                     |   tvalid  |      +----->
<----+ ! <---+ stop_out   void_out +---------->|tag   |      
!tvalid      |                     +           |      |      
+----------->| void_in    stop_in  |<--+ ! <+  +------+      
             +---------------------+        |       Ready<T> 
                                            +---------------+
")]
//!
//!# Example
//!
//! The core is very simple, so the test does not really
//! show much.
//!
//!```
#![doc = include_str!("../../../examples/axi_2_rhdl_stream.rs")]
//!```
//!
//! With trace
//!
#![doc = include_str!("../../../doc/axi_2_rhdl_stream.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{core::option::pack, lid::carloni::Carloni, stream::Ready};

#[derive(Clone, Default, Synchronous, SynchronousDQ)]
/// AXI to RHDL Stream Shim
///
/// This core provides a shim to convert an AXI stream
/// into a RHDL stream.  The type `T` is the data type
/// being transported on the stream.  Note that this
/// core is purely combinatorial, and does not register
/// the inputs or outputs.
pub struct Axi2Rhdl<T: Digital> {
    inbuf: Carloni<T>,
}

#[derive(Debug, PartialEq, Digital, Clone, Copy)]
/// Inputs for the [Axi2Rhdl] core
pub struct In<T: Digital> {
    /// The data signal on the AXI (incoming) side
    pub tdata: T,
    /// The valid flag on the AXI (incoming) side
    pub tvalid: bool,
    /// The ready signal from the RHDL stream
    pub ready: Ready<T>,
}

#[derive(Debug, PartialEq, Digital, Clone, Copy)]
/// Outputs from the [Axi2Rhdl] core
pub struct Out<T: Digital> {
    /// The data to the RHDL stream
    pub data: Option<T>,
    /// The `tready` signal for the AXI (incoming) side
    pub tready: bool,
}

impl<T: Digital> SynchronousIO for Axi2Rhdl<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    d.inbuf.data_in = i.tdata;
    d.inbuf.void_in = !i.tvalid;
    d.inbuf.stop_in = !i.ready.raw;
    let packed = pack::<T>(!q.inbuf.void_out, q.inbuf.data_out);
    let mut o = Out::<T>::dont_care();
    o.tready = !q.inbuf.stop_out;
    o.data = packed;
    (o, d)
}

#[cfg(test)]
mod tests {
    // Easiest way to test this is to use a RHDL [SourceFromFn] for the
    // stream and then decode it to feed the AXI stream.  Kinda nuts.

    use std::iter;

    use rhdl::prelude::*;

    use crate::{
        core::option::unpack,
        rng::xorshift::XorShift128,
        stream::{
            ready,
            testing::{sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling},
        },
    };

    use super::Axi2Rhdl;

    #[derive(Clone, Synchronous, SynchronousDQ)]
    struct TestFixture {
        source: SourceFromFn<b8>,
        axi_2_rhdl: Axi2Rhdl<b8>,
        sink: SinkFromFn<b8>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = ();
        type Kernel = kernel;
    }

    #[kernel]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> ((), D) {
        let mut d = D::dont_care();
        let (valid, data) = unpack::<b8>(q.source, bits(0));
        d.axi_2_rhdl.tdata = data;
        d.axi_2_rhdl.tvalid = valid;
        d.sink = q.axi_2_rhdl.data;
        d.axi_2_rhdl.ready = q.sink;
        d.source = ready::<b8>(q.axi_2_rhdl.tready);
        ((), d)
    }

    #[test]
    fn test_axi_to_rhdl() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| b8((x & 0xFF) as u128));
        let b_rng = a_rng.clone();
        let a_rng = stalling(a_rng, 0.23);
        let uut = TestFixture {
            source: SourceFromFn::new(a_rng),
            axi_2_rhdl: Axi2Rhdl::default(),
            sink: SinkFromFn::new_from_iter(b_rng, 0.2),
        };
        let input = iter::repeat_n((), 10_000).with_reset(1).clock_pos_edge(100);
        uut.run(input).for_each(drop);
        Ok(())
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut: Axi2Rhdl<b8> = Axi2Rhdl::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }
}
