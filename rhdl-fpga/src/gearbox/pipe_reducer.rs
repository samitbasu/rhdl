//! A PipeReducer Core
//!
//!# Purpose
//!
//! A PipeReducer Core takes arrays of type `T` and
//! splits it into it's constituent elements, each of
//! which is output in sequence.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [PipeReducer] buffer
//!
#![doc = badascii_formal!("
         +-+PipeReducer+--+        
 ?[T;N]  |                |  ?T    
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
//! The [PipeReducer] contains an entry flip flop to hold the input data (shown
//! below with an enable signal).  This DFF holds the current value being processed
//! and is needed to avoid a pipeline delay in the upstream pipeline producing a
//! new value to feed the reducer.  The tag and data are separated, and the
//! data element is selected using a counter fed from the control logic.
//! The tag is re-united in the packer, and then an [OptionCarloni] buffer
//! is used to isolate the input ready signal from the output ready signal.
//! Without this buffer, a combinatorial path will exist between the input and
//! outputs of the core, violating a general principle of latency insensitive
//! design.
//!
#![doc = badascii!(r"
                                        +                                               
         +DFF++        +-+unpck++       |\         +--+pck+--+      +-+?Carloni+--+     
         |    |        |        | [T;N] | +  T     |         | ?T   |             | ?T  
  ?[T;N] |    | ?[T;N] |    data+------>|n+------->|data  out+----->|data    data +---->
+------->|d  q+------->|in      |       | +        |         |      |             |     
         |    |        |     tag+--+    |/   +---->|tag      |   +--+ready   ready|<---+
         | en |        |        |  |    +^   |     |         |   |  |             |     
         +----+        +--------+  |     |   |     +---------+   |  +-------------+
           ^                       +-----+---+                   |      
           +-------------------+         |                       |       
                               |         |                       |                      
                            +--+---------+--+                    |                      
                            |next       sel |                    |                      
    ready                   |               |                    |                      
  <-------------------------+          ready|<-------------------+                      
                            |               |                                           
                            +--+Control+----+                                           
")]
//! Here is a rough timing diagram of how the control section operates.  I have
//! assumed that the pipline will run on every clock, which is only true if the
//! output [FifoToReadyValid] buffer is not `full` and data is present.  But
//! for brevity, I left it out.
//!
#![doc = badascii!("
data    D0  D0  D0  D0  D1  D1  D1  D1  D1  X  X  D2  D2 
           :   :   :   :   :   :   :   :   :  :  :   :   
sel      0   1   2   3   0   1   2   3   4  0  0   0   1 
           :   :   :   :   :   :   :   :   :  :  :   :   
                   +---+           +---+                 
next     +---------+   +-----------+   +----------------+
                                           :  :  :       
valid    +---------------------------------+     +------+
                                           +-----+       
")]
//!
//! From this diagram, a few key ideas emerge:
//!
//!   - The validity of the output data is taken from
//! the input data (as stored in the holding DFF).
//!   - The counter advances (with a wrap at `N-1`) as
//! long as the data is valid
//!   - The holding DFF is enabled when the counter reaches
//! `N-1`
//!
//!# Example
//!
//! Here is an example of running the pipelined reducer.
//!
//!```
#![doc = include_str!("../../examples/pipe_reducer.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../doc/pipe_reducer.md")]
//!
use crate::{
    core::{
        dff,
        option::{pack, unpack},
    },
    lid::option_carloni,
};

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The PipeReducer Core
///
/// This core takes a stream of `[T; N]`, and produces
/// a stream of `T`, reading out the input stream in
/// index order (`0, 1, 2..`).  
pub struct PipeReducer<M: BitWidth, T: Digital, const N: usize>
where
    [T; N]: Default,
    T: Default,
{
    store: dff::DFF<Option<[T; N]>>,
    count: dff::DFF<Bits<M>>,
    buffer: option_carloni::OptionCarloni<T>,
}

impl<M: BitWidth, T: Digital, const N: usize> Default for PipeReducer<M, T, N>
where
    [T; N]: Default,
    T: Default,
{
    fn default() -> Self {
        assert!((1 << M::BITS) >= N, "Expect that the bitwidth of the counter is sufficient to count the elements in the array.  I.e., (1 << M) >= N");
        Self {
            store: dff::DFF::new(None),
            count: dff::DFF::new(bits(0)),
            buffer: option_carloni::OptionCarloni::default(),
        }
    }
}

#[derive(Debug, PartialEq, Digital)]
/// Inputs for the [PipeReducer] core
pub struct In<T: Digital, const N: usize> {
    /// Input data elements that need to be reduced
    pub data: Option<[T; N]>,
    /// Input ready flag from downstream
    pub ready: bool,
}

#[derive(Debug, PartialEq, Digital)]
/// Outputs from the [PipeReducer] core
pub struct Out<T: Digital> {
    /// Output data elements from the input array
    pub data: Option<T>,
    /// Output ready flag to upstream
    pub ready: bool,
}

impl<M: BitWidth, T: Digital, const N: usize> SynchronousIO for PipeReducer<M, T, N>
where
    [T; N]: Default,
    T: Default,
{
    type I = In<T, N>;
    type O = Out<T>;
    type Kernel = kernel<M, T, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<M: BitWidth, T: Digital, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<M, T, N>,
) -> (Out<T>, D<M, T, N>)
where
    [T; N]: Default,
    T: Default,
{
    // Extract the tag from the input
    let (tag, data) = unpack::<[T; N]>(q.store);
    let mut out = Out::<T>::dont_care();
    // This boolean indicates the pipeline will advance
    let will_run = tag && q.buffer.ready;
    let mut d = D::<M, T, N>::dont_care();
    // The output value is the input data selected with
    // the tag copied from the input
    d.buffer.data = pack::<T>(tag, data[q.count]);
    d.buffer.ready = i.ready;
    // If we advance, then roll the counter
    d.count = q.count + if will_run { 1 } else { 0 };
    // The store DFF will normally hold state unless
    // it is empty.
    d.store = q.store;
    out.data = q.buffer.data;
    out.ready = false;
    // The two cases where it will be open to the input
    // bus is if it is empty/None, or if we will finish
    // with the contents.
    if !tag || (will_run && q.count == bits((N - 1) as u128)) {
        d.store = i.data;
        d.count = bits(0);
        out.ready = true;
    }
    (out, d)
}

#[cfg(test)]
mod tests {

    use crate::rng::xorshift::XorShift128;

    use super::*;

    fn mk_array<T, const N: usize>(t: &mut impl Iterator<Item = T>) -> [T; N]
    where
        [T; N]: Default,
    {
        let mut ret = <[T; N] as Default>::default();
        (0..N).for_each(|ndx| {
            ret[ndx] = t.next().unwrap();
        });
        ret
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = PipeReducer::<U2, b4, 4>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        type Uut = PipeReducer<U2, b4, 4>;
        let uut = Uut::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let mut dest_rng = source_rng.clone();
        let mut latched_input: Option<[b4; 4]> = None;
        uut.run_fn(
            move |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = super::In::<b4, 4>::dont_care();
                // Downstream is likely to run
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = !want_to_pause;
                // Decide if the producer will generate a new data item
                let willing_to_send = rand::random::<u8>() < 200;
                if out.ready {
                    // The pipeline wants more data
                    if willing_to_send {
                        latched_input = Some(mk_array(&mut source_rng));
                    } else {
                        latched_input = None;
                    }
                }
                input.data = latched_input;
                if input.ready && out.data.is_some() {
                    assert_eq!(dest_rng.next(), out.data);
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 100_000)
        .for_each(drop);
        Ok(())
    }
}
