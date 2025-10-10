//! Xfer Detector Core
//!
//!# Purpose
//!
//! A [Xfer] Core sits on a stream, and emits a pulse
//! (combinatorially) in each clock cycle that a valid
//! transfer takes place on the stream.  It is equivalent
//! to computing `data.is_some() && ready`, but can be
//! easier to use as a block than as an expression.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [Xfer] core
//!
#![doc = badascii_formal!(r"
     +--+Xfer+--------+      
 ?T  |                | ?T   
+--->+ data     data  +----> 
Ry<T>|                | Ry<T>
<----+ ready    ready |<---+ 
     |      run       |      
     +-------+--------+      
             v               
")]
//!
//!# Internals
//!
//! This core is very simple.  It passes the data and ready
//! signals through, and derives (combinatorially) the `run`
//! signal as `data.is_some() && ready`.
//!
#![doc = badascii!(r"
 ?T                             
+------+------------------->    
       |                        
       | +-------+        run   
       +>|is_some+--> & +-->    
         +-------+              
 Ready<T>             ^ Ready<T>
<---------------------+-----+   
")]
//!
//!# Example
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;
use std::marker::PhantomData;

use crate::core::option::is_some;

use super::{Ready, StreamIO};

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The [Xfer] core
///
/// Emits a `run` pulse on every clock cycle where a
/// transfer takes place on a stream.  This core is unbuffered,
/// and the output is combinatorially derived from the inputs.
pub struct Xfer<T: Digital> {
    marker: PhantomData<T>,
}

impl<T: Digital> Default for Xfer<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

/// Output of the [Xfer] core
#[derive(PartialEq, Clone, Digital)]
pub struct Out<T: Digital> {
    /// The data flowing out of the core
    pub data: Option<T>,
    /// The ready signal flowing out of the core
    pub ready: Ready<T>,
    /// A pulse that is high when a transfer takes place
    pub run: bool,
}

impl<T: Digital> SynchronousIO for Xfer<T> {
    type I = StreamIO<T, T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: StreamIO<T, T>, _q: Q<T>) -> (Out<T>, D<T>) {
    let d = D::<T> { marker: () };
    let run = is_some::<T>(i.data) & i.ready.raw;
    let o = Out::<T> {
        data: i.data,
        ready: i.ready,
        run,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use super::*;
    use crate::{
        core::dff::DFF,
        rng::xorshift::XorShift128,
        stream::testing::{
            sink_from_fn::SinkFromFn, source_from_fn::SourceFromFn, utils::stalling,
        },
    };

    #[derive(Clone, Synchronous, SynchronousDQ)]
    pub struct TestFixture {
        source: SourceFromFn<b4>,
        count: DFF<b16>,
        xfer: Xfer<b4>,
        sink: SinkFromFn<b4>,
    }

    impl SynchronousIO for TestFixture {
        type I = ();
        type O = b16;
        type Kernel = kernel;
    }

    #[kernel]
    #[doc(hidden)]
    pub fn kernel(_cr: ClockReset, _i: (), q: Q) -> (b16, D) {
        let mut d = D::dont_care();
        d.source = q.xfer.ready;
        d.sink = q.xfer.data;
        d.count = q.count;
        if q.xfer.run {
            d.count += 1;
        }
        d.xfer.data = q.source;
        d.xfer.ready = q.sink;
        (q.count, d)
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default()
            .map(|x| b4((x & 0xF) as u128))
            .take(10);
        let b_rng = a_rng.clone();
        let a_rng = stalling(a_rng, 0.23);
        let uut = TestFixture {
            source: SourceFromFn::new(a_rng),
            count: DFF::default(),
            xfer: Xfer::default(),
            sink: SinkFromFn::new_from_iter(b_rng, 0.3),
        };
        let input = repeat_n((), 1000).with_reset(1).clock_pos_edge(100);
        let last_output = uut.run_without_synthesis(input)?.last().unwrap();
        let last_count = last_output.value.2.raw();
        assert_eq!(last_count, 10);
        Ok(())
    }
}
