//! RHDL Stream to AXI Stream adapter
//!
//! This core is a very lightweight shim that has a
//! RHDL Stream interface on the upstream side, and an
//! AXI Stream interface on the downstream side.  
//!
//!# Schematic Symbol
//!
#![doc = badascii_formal!(r"
      +---+RHDL2AXI+--+     
      |RHDL   :   AXI |     
  ?T  |       :       | T   
+---->|data   : tdata +---> 
      |       :       |     
      |       : tvalid+---> 
      |       :       |     
 <----+ready  : tready|<---+
      +---------------+     
")]
//!
//!# Internal Details
//!
//! The core is simple.  It simply [unpack]'s the
//! [Option] data input into a `tvalid` flag and
//! a `tdata` output to comply with the AXI stream
//! definition.  The `tready` signal is simply forwarded.
//!
#![doc = badascii!(r"
     +-+unpack++ T 
     |   tdata +-->
  ?T |         |   
+--->|data     |   
     |         |   
     |   tvalid+-->
     +---------+   
 ready      tready 
<-----------------+
")]
//!
//!# Example
//!
//! The core is very simple.  In this case, we simply go from
//! a RHDL Stream -> AXI and then back again.
//!
//!```
#![doc = include_str!("../../../examples/rhdl_2_axi_stream.rs")]
//!```
//!
//! With trace
//!
#![doc = include_str!("../../../doc/rhdl_2_axi_stream.md")]

use std::marker::PhantomData;

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::option::unpack;

#[derive(Clone, PartialEq, Default, Synchronous, SynchronousDQ)]
/// RHDL Stream to AXI Stream shim
///
/// This core provides a shim to convert a RHDL stream into
/// an AXI Stream.  The type `T` is the data type being transported
/// on the stream.  Note that this core is purely combinatorial, and
/// does not register the inputs or outputs.
pub struct Rhdl2Axi<T: Digital + Default> {
    marker: PhantomData<T>,
}

#[derive(Debug, PartialEq, Digital)]
/// Inputs for the [Rhdl2Axi] core
pub struct In<T: Digital> {
    /// The data signal on the RHDL upstream
    pub data: Option<T>,
    /// The `tready` signal from the AXI downstream
    pub tready: bool,
}

#[derive(Debug, PartialEq, Digital)]
/// Outputs from the [Rhdl2Axi] core
pub struct Out<T: Digital> {
    /// The data signal on the AXI downstream
    pub tdata: T,
    /// The valid signal on the AXI downstream
    pub tvalid: bool,
    /// The ready signal to the RHDL upstream
    pub ready: bool,
}

impl<T: Digital + Default> SynchronousIO for Rhdl2Axi<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital + Default>(_cr: ClockReset, i: In<T>, _q: Q<T>) -> (Out<T>, D<T>) {
    let (tvalid, tdata) = unpack::<T>(i.data);
    (
        Out::<T> {
            tdata,
            tvalid,
            ready: i.tready,
        },
        D::<T> { marker: () },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        axi4lite::stream::axi_to_rhdl::Axi2Rhdl,
        rng::xorshift::XorShift128,
        stream::testing::{
            sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling,
        },
    };

    #[derive(Clone, Synchronous, SynchronousDQ)]
    struct TestFixture {
        source: SourceFromFn<b8>,
        rhdl_2_axi: Rhdl2Axi<b8>,
        axi_2_rhdl: Axi2Rhdl<b8>,
        sink: SinkFromFn<b8>,
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
        d.rhdl_2_axi.data = q.source;
        d.source = q.rhdl_2_axi.ready;
        d.axi_2_rhdl.tdata = q.rhdl_2_axi.tdata;
        d.axi_2_rhdl.tvalid = q.rhdl_2_axi.tvalid;
        d.rhdl_2_axi.tready = q.axi_2_rhdl.tready;
        d.sink = q.axi_2_rhdl.data;
        d.axi_2_rhdl.ready = q.sink;
        ((), d)
    }

    #[test]
    fn test_axi_rhdl_back_again() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| b8((x & 0xFF) as u128));
        let mut b_rng = a_rng.clone();
        let a_rng = stalling(a_rng, 0.23);
        let consume = move |data: Option<b8>| {
            if let Some(data) = data {
                let orig = b_rng.next().unwrap();
                assert_eq!(data, orig);
            }
            rand::random::<f64>() > 0.2
        };
        let uut = TestFixture {
            source: SourceFromFn::new(a_rng),
            rhdl_2_axi: Rhdl2Axi::default(),
            axi_2_rhdl: Axi2Rhdl::default(),
            sink: SinkFromFn::new(consume),
        };
        let input = std::iter::repeat_n((), 10_000)
            .with_reset(1)
            .clock_pos_edge(100);
        uut.run_without_synthesis(input)?.for_each(drop);
        Ok(())
    }
}
