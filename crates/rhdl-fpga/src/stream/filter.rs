//! Filter Stream Core
//!
//!# Purpose
//!
//! A [Filter] Core takes a stream of elements of type `T`
//! and a function `fn(T) -> bool`, and keeps only those items for
//! which the function evaluates to `true`.  The filter function is
//! provided in the form of a synthesizable function.  This is
//! equivalent to using `.filter()` on an interator.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [Filter] core
//!
#![doc = badascii_formal!("
      +-+Filter+-----+        
 ?T   |              | ?T    
+---->+data     data +----->
 R<T> |              | R<T>       
<-----+ready    ready|<----+
      +--------------+       
")]
//!
//!# Internals
//!
//! Unlike [Flatten] or [Chunked], the [FilterPipe] does not
//! impose any flow control on the upstream.  Because it can
//! at most produce as many items as the source, it can be
//! implemented with a [StreamBuffer] buffers at the input
//! which is needed to isolate the combinatorial
//! filter function from the remaining parts of the stream.  
//! Note that if you need a more expensive filter function (i.e., one
//! that itself is pipelined), then you cannot use this construct.
//!
#![doc = badascii!(r"
                                      +-+Func+--+                        
                                      |         |                        
                                    +>|in   keep+--+                     
     +-+Input Buf++     +-+upck+-+  | +---------+  |   +-+pck+-+         
 ?T  |            | ?T  |        |T |              |   |       |?T data  
+--->|data    data+---->|in   out+--+-------------+|+->|in  out+-------->
R<T> |            |     |        |                 +   |       |   Ready<T>
<----+ready  ready|<-+  |     tag+---------------> &+->|tag    |  +-----+
     +------------+  |  +--------+                     +-------+  |      
                     |                                            |      
                     +--------------------------------------------+      
")]
//!# Example
//!
//! Here is an example of filtering a stream.
//!
//!```
#![doc = include_str!("../../examples/filter.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../doc/filter.md")]
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use super::{stream_buffer::StreamBuffer, StreamIO};

#[derive(Clone, Synchronous, SynchronousDQ)]
#[rhdl(dq_no_prefix)]
/// The [Filter] Stream Core
///
/// Here `T` is the type flowing in the stream.
/// At construction time, you provide a synthesizable
/// function to filter the contents of the stream.
/// Only items for which `fn(T)` returns `true` will
/// be passed on downstream.
pub struct Filter<T: Digital> {
    input_buffer: StreamBuffer<T>,
    func: Func<T, bool>,
}

impl<T> Filter<T>
where
    T: Digital,
{
    /// Construct a [Filter] Stream
    ///
    /// The argument to the filter `try_new` function
    /// is a synthesizable function (i.e., one marked with the
    /// `#[kernel]` attribute).  It must have a signature of
    /// `fn(ClockReset, T) -> bool`.
    pub fn try_new<S>() -> Result<Self, RHDLError>
    where
        S: DigitalFn,
        S: DigitalFn2<A0 = ClockReset, A1 = T, O = bool>,
    {
        Ok(Self {
            input_buffer: StreamBuffer::default(),
            func: Func::try_new::<S>()?,
        })
    }
}

/// The input for the [Filter]
pub type In<T> = StreamIO<T, T>;

/// The output of the [Filter]
pub type Out<T> = StreamIO<T, T>;

impl<T> SynchronousIO for Filter<T>
where
    T: Digital,
{
    type I = In<T>;
    type O = Out<T>;
    type Kernel = kernel<T>;
}

#[kernel(allow_weak_partial)]
#[doc(hidden)]
pub fn kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    d.input_buffer.data = i.data;
    d.func = T::dont_care();
    let mut tag = false;
    if let Some(data) = q.input_buffer.data {
        d.func = data;
        tag = true;
    }
    let tag = tag && q.func;
    d.input_buffer.ready = i.ready;
    let mut o = Out::<T> {
        data: None,
        ready: q.input_buffer.ready,
    };
    if let Some(data) = q.input_buffer.data {
        if tag {
            o.data = Some(data);
        }
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use crate::{
        rng::xorshift::XorShift128,
        stream::testing::{single_stage::single_stage, utils::stalling},
    };

    use super::*;

    #[kernel]
    fn keep_even(_cr: ClockReset, t: b4) -> bool {
        !(t & bits(1)).any()
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let filter = Filter::try_new::<keep_even>()?;
        drc::no_combinatorial_paths(&filter)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
        let a_rng = stalling(a_rng, 0.23);
        let consume = move |data: Option<b4>| {
            if let Some(data) = data {
                // Only even values kept
                assert!(data.raw() & 1 == 0);
            }
            rand::random::<f64>() > 0.2
        };
        let filter = Filter::try_new::<keep_even>()?;
        let uut = single_stage(filter, a_rng, consume);
        // Run a few samples through
        let input = repeat_n((), 10_000).with_reset(1).clock_pos_edge(100);
        uut.run(input).for_each(drop);
        Ok(())
    }
}
