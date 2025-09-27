//! Filter Map Pipe Core
//!
//!# Purpose
//!
//! A [FilterMap] Core takes a sequence of elements of type `T`
//! and a function `fn(T) -> Option<S>`, and keeps only those
//! items which are `Some`.  This is particularly handy for pipelines
//! that operate on `enum` values, and extract only variants of
//! a particular kind.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [FilterMap] stream
//!
#![doc = badascii_formal!("
     ++FilterMap++        
 ?T  |           | ?S    
+--->+ in    out +---->
     +-----------+       
")]
//!
//!# Internals
//!
//! The [FilterMap] pipeline includes a simple DFF at the input
//! to isolate the function from the rest of the pipeline.
//!
#![doc = badascii!(r"
                              ++func++   +      
                              |      |?S |\     
                            +>|in out+-->|1+ ?S 
     +-+DFF++     +unpack+  | +------+   | +--->
 ?T  |      | ?T  |      |T |     None+->|0+    
+--->|d    q+---->|in out+--+            |/     
     +------+     |      |               +^     
                  |   tag+----------------+     
                  +------+                      
")]
//!
//!# Example
//!
//! Here is an example of running the pipeline filter map.  It is
//! interesting, because it demonstrates the use of `enum` values
//! that are filter-mapped and the payload stripped for further
//! processing.
//!
//!```
#![doc = include_str!("../../examples/pipe_filter_map.rs")]
//!```
//!
//! with the trace file:
//!
#![doc = include_str!("../../doc/pipe_filter_map.md")]
//!
//!

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::dff::DFF;

#[derive(Clone, Synchronous, SynchronousDQ)]
/// The FilterMap Core
///
/// Here `T` is the input type, and `S` is the
/// output type.  The provided combinatorial function
/// performs the mapping.  It must have a signature of
/// `fn(T) -> Option<S>`.
pub struct FilterMap<T: Digital, S: Digital> {
    input: DFF<Option<T>>,
    func: Func<T, Option<S>>,
}

impl<T, S> FilterMap<T, S>
where
    T: Digital,
    S: Digital,
{
    /// Construct a Filter Map Stream
    ///
    /// The argument to the filter map
    /// `try_new` function is a synthesizable function
    /// (i.e., one marked with the `#[kernel]` attribute).
    /// It must have a signature `fn(T) -> Option<S>`.
    pub fn try_new<K>() -> Result<Self, RHDLError>
    where
        K: DigitalFn,
        K: DigitalFn2<A0 = ClockReset, A1 = T, O = Option<S>>,
    {
        Ok(Self {
            input: DFF::new(None),
            func: Func::try_new::<K>()?,
        })
    }
}

/// The input for the [FilterMap] pipeline
pub type In<T> = Option<T>;

/// The output for the [FilterMap] pipeline
pub type Out<S> = Option<S>;

impl<T, S> SynchronousIO for FilterMap<T, S>
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
    let mut o = None;
    if let Some(data) = q.input {
        d.func = data;
        o = q.func;
    }
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{core::slice::lsbs, rng::xorshift::XorShift128, stream::testing::utils::stalling};

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
        let b_rng = a_rng.clone().filter_map(|x| {
            if (x & bits(1)).any() {
                None
            } else {
                Some(lsbs::<U2, U4>(x))
            }
        });
        let a_rng = stalling(a_rng, 0.23);
        let uut = FilterMap::try_new::<filter_map_item>()?;
        let input = a_rng.with_reset(1).clock_pos_edge(100);
        let output = uut
            .run(input)
            .synchronous_sample()
            .filter_map(|t| t.value.2);
        assert!(output.take(10_000).eq(b_rng.take(10_000)));
        Ok(())
    }
}
