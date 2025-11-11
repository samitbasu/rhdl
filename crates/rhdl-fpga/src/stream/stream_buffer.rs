//!# Carloni Buffer with Option Interface
//!
//! This core wraps the [Carloni] skid buffer core
//! in a more ergonomic [Option] based interface.  The
//! `void_in` input and the `data_in` are combined into
//! a single `data_in` with type [Option<T>], and the
//! `void_out` and `data_out` are similarly combined
//! into a single `data_out` with type [Option<T>].  Furthermore
//! for compatibility with the `ready-valid` interface used
//! elsewhere in RHDL, the `stop` signals are inverted to be
//! `valid` signals.
//!
//!# Schematic symbol
//!
//! Here is the symbol for the buffer.
//!
#![doc = badascii_formal!("
        +-+OptionCarloni++      
    ?T  |                | ?T   
   +--->|data        data+----> 
Ready<T>|                | Ready<T>     
   <----+ready      ready|<----+
        |                |      
        +----------------+      
")]
//!
//!# Internal details
//!
//! Internally, the buffer is simply a [CarloniBuffer]
//! with `pack` and `unpack` cores to convert the
//! [Option<T>]  to a pair of `data` and `valid` lines.
//! The code is pretty short and self-expanatory.
//!
//! Here is a sketch of the internals
//!
#![doc = badascii!(r"
                            +-----+Carloni+-------+                          
      ++unpck++             |                     |           ++pack+-+      
 data |       |  +--------->| data_in    data_out +-------+   |       | data 
+---->|in    T+--+          |                     |       +-->|T   out+----->
      |       |        +----+ stop_out   stop_in  |<---+      |       |      
      |  valid+--+     |    |                     |    |      |       |      
      |       |  +-----+--->| void_in    void_out +----+----->|valid  |      
      +-------+        |    |                     |    |      +-------+      
                       |    +---------------------+    |                     
          +            |                               |        +            
 ready   /|            |                               |       /|  ready     
<-----+○+ |<-----------+                               +----+○+ |<------+    
         \|                                                    \|            
          +                                                     +            
")]
//!
//!# Example
//!
//! Here is the example from the [CarloniBuffer] with an [Option<T>]
//! based interface.
//!
//!```
#![doc = include_str!("../../examples/stream_buffer.rs")]
//!```
//!
//! With trace
//!
#![doc = include_str!("../../doc/option_carloni.md")]
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    core::option::pack,
    lid::carloni::Carloni,
    stream::{ready, StreamIO},
};

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
/// Option-based Carloni buffer core
///
/// Here `T` is the data type being transported through
/// the buffer.
pub struct StreamBuffer<T: Digital> {
    inner: Carloni<T>,
}

impl<T: Digital> Default for StreamBuffer<T> {
    fn default() -> Self {
        Self {
            inner: Carloni::default(),
        }
    }
}

/// Inputs to the [StreamBuffer] buffer core
pub type In<T> = StreamIO<T, T>;

/// Outputs from the [StreamBuffer] buffer core
pub type Out<T> = StreamIO<T, T>;

impl<T: Digital> SynchronousIO for StreamBuffer<T> {
    type I = In<T>;
    type O = Out<T>;
    type Kernel = option_carloni_kernel<T>;
}

#[kernel(allow_weak_partial)]
#[doc(hidden)]
pub fn option_carloni_kernel<T: Digital>(_cr: ClockReset, i: In<T>, q: Q<T>) -> (Out<T>, D<T>) {
    let mut d = D::<T>::dont_care();
    let (data_valid, data) = match i.data {
        Some(data) => (true, data),
        None => (false, T::dont_care()),
    };
    d.inner.data_in = data;
    d.inner.void_in = !data_valid;
    d.inner.stop_in = !i.ready.raw;
    let mut o = Out::<T>::dont_care();
    o.ready = ready::<T>(!q.inner.stop_out);
    o.data = pack::<T>(!q.inner.void_out, q.inner.data_out);
    (o, d)
}

#[cfg(test)]
mod tests {

    use crate::rng::xorshift::XorShift128;

    use super::*;

    #[test]
    fn test_option_carloni_buffer() {
        let uut = StreamBuffer::<b32>::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default();
        let mut output_rng = XorShift128::default();
        uut.run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(None);
                }
                let mut input = In::<b32>::dont_care();
                // Downstream reandomly wants to pause
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = ready(!want_to_pause);
                // Upstream may have paused
                let want_to_send = rand::random::<u8>() < 200;
                input.data = None;
                if out.ready.raw && want_to_send {
                    // The receiver did not tell us to stop, and
                    // we want to send something
                    input.data = Some(bits(source_rng.next().unwrap() as u128));
                }
                // Check output
                if out.data.is_some() && input.ready.raw {
                    // The output will advance on this clock cycle
                    assert_eq!(out.data, Some(bits(output_rng.next().unwrap() as u128)));
                }
                Some(Some(input))
            },
            100,
        )
        .take_while(|t| t.time < 100_000)
        .for_each(drop);
    }
}
