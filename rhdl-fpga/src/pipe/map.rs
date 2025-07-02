//! Map Pipe Core
//!
//!# Purpose
//!
//! A [Map] pipe core takes a pipeline of elements of
//! type `T` and a synthesizable function `fn(T) -> S`
//! and outputs a pipeline that carries elements of
//! type `S`.  Pipes do not accept backpressure, but they
//! do accept a strobe in the form of an [Option] input.
//! This is equivalent to calling `.map()` on an iterator.
//! Note that the inputs are registered, but _not_ the outputs.
//! This is by design, so that the pipe cores can be chained
//! without redundant register slices between them.  If you want
//! an extra register slice, you can insert a [Buffer] in the
//! pipeline.  There are no combinatorial paths from the input
//! to the output, and this is verified via a test.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [Map] core.
//!
#![doc = badascii_formal!("
      +--+Map+-------+        
  ?T  |              | ?S   
+---->+ data   data  +----->
      +--------------+       
")]
//!
//!# Internals
//!
//! The [Map] core is simple.  A [DFF] registers the incoming
//! data.  When the input is `Some`, the provided function is
//! provided the data, and the output wrapped into an option
//! for passing down the pipeline.
#![doc = badascii!(r"
                              +-+Func+--+                   
                              |         | S                 
                            +>|in    out+--+                
     ++DFF++    +-+upck+-+  | +---------+  |   +-+pck+-+    
 ?T  |     | ?T |        |T |              |   |       |?S  
+--->|d   q+--->|in   out+--+              +-->|in  out+--->
     +-----+    |        |                     |       |    
                |     tag+-------------------->|tag    |    
                +--------+                     +-------+    
")]
//!
//!# Example
//!
//! Here is an example of a mapping a pipeline, transforming
//! elements.   Note that there is a single clock cycle of
//! latency introduced by the input buffer.
//!
//!```
#![doc = include_str!("../../examples/pipe_map.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../doc/pipe_map.md")]
//!

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::dff::DFF;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// The Map Core (Pipe Version)
///
/// Here `T` is the input type, and `S` is the
/// output type as carried by the pipe.  A provided
/// (combinatorial) function performs the mapping
/// function.
pub struct Map<T: Digital, S: Digital> {
    input: DFF<Option<T>>,
    func: Func<T, S>,
}

impl<T, S> Map<T, S>
where
    T: Digital,
    S: Digital,
{
    /// Construct a Map Pipe
    ///
    /// The argument to the map pipe `try_new` function
    /// is a synthesizable function (i.e., one marked with the
    /// `#[kernel]` attribute).  It must have a signature of
    /// `fn(ClockReset, T) -> S`.
    pub fn try_new<K>() -> Result<Self, RHDLError>
    where
        K: DigitalFn,
        K: DigitalFn2<A0 = ClockReset, A1 = T, O = S>,
    {
        Ok(Self {
            input: DFF::new(None),
            func: Func::try_new::<K>()?,
        })
    }
}

/// The input for the [Map] pipe
pub type In<T> = Option<T>;

/// The output for the [Map] pipe
pub type Out<S> = Option<S>;

impl<T, S> SynchronousIO for Map<T, S>
where
    T: Digital,
    S: Digital,
{
    type I = In<T>;
    type O = Out<S>;
    type Kernel = kernel<T, S>;
}

#[kernel(allow_weak_partial)]
#[doc(hidden)]
pub fn kernel<T, S>(_cr: ClockReset, i: In<T>, q: Q<T, S>) -> (Out<S>, D<T, S>)
where
    T: Digital,
    S: Digital,
{
    let mut d = D::<T, S>::dont_care();
    d.input = i;
    d.func = T::dont_care();
    let o = if let Some(data) = q.input {
        d.func = data;
        Some(q.func)
    } else {
        None
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::slice::lsbs, rng::xorshift::XorShift128, stream::testing::utils::stalling};

    #[kernel]
    fn map_item(_cr: ClockReset, t: b4) -> b2 {
        lsbs::<U2, U4>(t)
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let input = XorShift128::default().map(|x| b4(x as u128 & 0xF));
        let expected = input.clone().map(lsbs::<U2, U4>);
        let uut = Map::try_new::<map_item>()?;
        let input = stalling(input, 0.23);
        let input = input.with_reset(1).clock_pos_edge(100);
        let output = uut.run(input)?.synchronous_sample();
        let output = output.filter_map(|x| x.value.2);
        assert!(output.take(10_000).eq(expected.take(10_000)));
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let input = XorShift128::default().map(|x| b4(x as u128 & 0xF));
        let input = stalling(input, 0.23).take(20);
        let input = input.with_reset(1).clock_pos_edge(100);
        let uut = Map::try_new::<map_item>()?;
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
