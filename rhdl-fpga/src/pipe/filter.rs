//! Filter Pipe Core
//!
//!# Purpose
//!
//! A [Filter] Pipe core takes a pipeline of elements of type `T`
//! and a function `fn(T) -> bool`, and keeps only those
//! items for which the function evaluates to `true`.  The filter
//! funciton is provided in the form of a synthesizable function.
//! This is equivalent to using `.filter()` on an iterator.
//!
//!# Schematic Symbol
//!
//! Here is a schematic symbol for the [Filter] core.
//!
#![doc = badascii_formal!("
      +--+Filter+------+        
 ?T   |                | ?T    
+---->+ data     data  +----->
      |                |        
<-----+ ready    ready |<----+
      +----------------+       
")]
//!
//!# Internals
//!
//! The [Filter] core includes a buffer to separate the
//! combinatorial filter function from the previous
//! pipe stage, but not an output buffer.  This way,
//! when pipe stages are connected together, the total
//! latency only increases by one per stage added.
//!
#![doc = badascii!(r"
                                 +-+Func+--+                    
                                 |         |                    
                               +>|in   keep+--+                 
     +-+DFF+-+     +-+upck+-+  | +---------+  |   +-+pck+-+     
 ?T  |       | ?T  |        |T |              |   |       |?T   
+--->|d     q+---->|in   out+--+-------------+|+->|in  out+---->
     +-------+     |        |                 +   |       |     
                   |     tag+---------------> &+->|tag    |     
                   +--------+                     +-------+     
")]
//!
//!# Example
//!
//! Here is an example of a filtering pipeline, in which
//! some of the elements are discarded by the filter function.
//! The single cycle of latency introduced by the input buffer
//! is used to isolate the combinatorial pathways.
//!
//!```
#![doc = include_str!("../../examples/pipe_filter.rs")]
//!```
//!
//! with the trace file:
//!
#![doc = include_str!("../../doc/pipe_filter.md")]
//!
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::{
    dff::DFF,
    option::{pack, unpack},
};

#[derive(Clone, Synchronous, SynchronousDQ)]
/// The [Filter] Pipe Core
///
/// Here `T` is the type flowing in the stream.
/// At constructino time, you provide a synthesizable
/// function to filter the contents of the stream.
/// Only items for which `fn(T)` returns `true` will
/// be passed on downstream.
pub struct Filter<T: Digital> {
    input: DFF<Option<T>>,
    func: Func<T, bool>,
}

impl<T> Filter<T>
where
    T: Digital,
{
    /// Construct a [Filter] Pipe
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
            input: DFF::new(None),
            func: Func::try_new::<S>()?,
        })
    }
}

/// The input for the [Filter]
pub type In<T> = Option<T>;

/// The output for the [Filter]
pub type Out<T> = Option<T>;

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
    d.input = i;
    let mut o = None;
    d.func = T::dont_care();
    if let Some(data) = q.input {
        d.func = data;
        if q.func {
            o = Some(data);
        }
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use crate::{rng::xorshift::XorShift128, stream::testing::utils::stalling};

    use super::*;

    #[kernel]
    fn keep_even(_cr: ClockReset, t: b4) -> bool {
        !(t & bits(1)).any()
    }

    #[test]
    fn test_operation() -> Result<(), RHDLError> {
        let input = XorShift128::default().map(|x| b4((x & 0xF) as u128));
        let expected = input.clone().filter(|&x| !(x & bits(1)).any());
        let input = stalling(input, 0.23);
        let uut = Filter::try_new::<keep_even>()?;
        let input = input.with_reset(1).clock_pos_edge(100);
        let output = uut.run(input)?.synchronous_sample();
        let output = output.filter_map(|t| t.value.2);
        assert!(output.take(10_000).eq(expected.take(10_000)));
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let input = XorShift128::default().map(|x| b4((x & 0xF) as u128));
        let input = stalling(input, 0.23).take(20);
        let uut = Filter::try_new::<keep_even>()?;
        let input = input.with_reset(1).clock_pos_edge(100);
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
