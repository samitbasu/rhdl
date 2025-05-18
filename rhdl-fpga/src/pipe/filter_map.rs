//! Filter Map Pipe Core
//!
//!# Purpose
//!
//! A [FilterMap] Core takes a sequence of elements of type `T`
//! and a function `fn(T) -> Option<S>`, and keeps only those
//! items which are `Some`.  This is particularly handy for
//! processing streams of `enum` values, and then extracting
//! a particular variant, for example.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [FilterMapPipe] buffer
//!
#![doc = badascii_formal!("
         ++FilterMapPipe+-+        
 ?T      |                |  ?S    
+------->+ data     data  +------->
         |                |        
         |                |        
<--------+ ready    ready |<------+
         |                |        
         +----------------+       
")]
//!
//!# Internals
//!
//! Unlike [FlattenPipe] or [ChunkedPipe], the [FilterMapPipe] does not
//! impose any flow control on the upstream pipe.  Because it can
//! at most produce as many items as the source pipe, it can be
//! implemented with simple [OptionCarloni] buffers at the input
//! and output, which are needed to isolate the combinatorial
//! filter-map function from the remaining parts of the pipeline.  
//! Note that if you need a more expensive filter-map function (i.e., one
//! that itself is pipelined), then you cannot use this construct.
//!
#![doc = badascii!(r"
                                    ++func++   +                          
                                    |      |?S |\                         
                                  +>|in out+-->|1+                        
     +-+Input Buf++     +unpack+  | +------+   | +-+    ++Output Buf++    
 ?T  |            | ?T  |      |T |     None+->|0+ |    |            | ?S 
+--->|data    data+---->|in out+--+            |/  +--->|data    data+--->
     |            |     |      |               +^       |            |    
<----+ready  ready|<-+  |   tag+----------------+    +--+ready  ready|<--+
     +------------+  |  +------+                     |  +------------+    
       ?Carloni      +-------------------------------+    ?Carloni        
")]
//!# Example
//!
//! Here is an example of running the pipeline filter map.  It is
//! interesting, because it demonstrates the use of `enum` values
//! to thar are filtered and the payload stripped for further
//! processing.
//!
//!```
#![doc = include_str!("../../examples/filter_map.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../doc/filter_map.md")]
//!

use badascii_doc::{badascii, badascii_formal};

use rhdl::prelude::*;

use crate::{core::option::unpack, lid::option_carloni::OptionCarloni};

use super::PipeIO;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// The FilterMap Core
///
/// Here `T` is the input type, and `S` is the
/// output type.  A provided (combinatorial) function
/// performs the mapping function.
pub struct FilterMapPipe<T: Digital + Default, S: Digital + Default> {
    input_buffer: OptionCarloni<T>,
    func: Func<T, Option<S>>,
    output_buffer: OptionCarloni<S>,
}

impl<T, S> FilterMapPipe<T, S>
where
    T: Digital + Default,
    S: Digital + Default,
{
    /// Construct a Filter Map Pipe
    ///
    /// The argument to the filter map pipe
    /// `try_new` function is a synthesizable function
    /// (i.e., one marked with the `#[kernel]` attribute).
    /// It must have a signature `fn(T) -> Option<S>`.
    pub fn try_new<K>() -> Result<Self, RHDLError>
    where
        K: DigitalFn,
        K: DigitalFn2<A0 = ClockReset, A1 = T, O = Option<S>>,
    {
        Ok(Self {
            input_buffer: OptionCarloni::default(),
            func: Func::try_new::<K>()?,
            output_buffer: OptionCarloni::default(),
        })
    }
}

/// The input for the [FilterMapPipe]
pub type In<T> = PipeIO<T>;

/// The output type for the [FilterMapPipe]
pub type Out<S> = PipeIO<S>;

impl<T, S> SynchronousIO for FilterMapPipe<T, S>
where
    T: Digital + Default,
    S: Digital + Default,
{
    type I = In<T>;
    type O = Out<S>;
    type Kernel = kernel<T, S>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T, S>(_cr: ClockReset, i: In<T>, q: Q<T, S>) -> (Out<S>, D<T, S>)
where
    T: Digital + Default,
    S: Digital + Default,
{
    let mut d = D::<T, S>::dont_care();
    d.input_buffer.data = i.data;
    let (tag, data) = unpack::<T>(q.input_buffer.data);
    d.func = data;
    d.output_buffer.data = if !tag { None } else { q.func };
    d.output_buffer.ready = i.ready;
    d.input_buffer.ready = q.output_buffer.ready;
    let o = Out::<S> {
        data: q.output_buffer.data,
        ready: q.input_buffer.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::iter::repeat_n;

    use crate::{
        core::slice::lsbs,
        pipe::testing::{single_stage::single_stage, utils::stalling},
        rng::xorshift::XorShift128,
    };

    use super::*;

    #[kernel]
    fn filter_map_item(_cr: ClockReset, t: b4) -> Option<b2> {
        if (t & bits(1)).any() {
            None
        } else {
            Some(lsbs::<U2, U4>(t))
        }
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
        let mut b_rng = a_rng.clone().filter_map(|x| {
            if (x & bits(1)).any() {
                None
            } else {
                Some(lsbs::<U2, U4>(x))
            }
        });
        let a_rng = stalling(a_rng, 0.23);
        let consume = move |data: Option<b2>| {
            if let Some(data) = data {
                let orig = b_rng.next().unwrap();
                assert_eq!(data, orig);
            }
            rand::random::<f64>() > 0.2
        };
        let map = FilterMapPipe::try_new::<filter_map_item>()?;
        let uut = single_stage(map, a_rng, consume);
        // Run a few samples through
        let input = repeat_n((), 10_000)
            .stream_after_reset(1)
            .clock_pos_edge(100);
        uut.run_without_synthesis(input)?.for_each(drop);
        Ok(())
    }
}
