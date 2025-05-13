//! Flatten Pipe Core
//!
//!# Purpose
//!
//! A [FlattenPipe] Core takes a sequence of arrays of
//! type `[T; N]` and splits them into individual items of
//! type `T`.  It is roughly equivalent to calling
//! `.iter().flatten()` on an iterator that returns `[T; N]` slices.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [FlattenPipe] buffer
//!
#![doc = badascii_formal!("
         +-+FlattenPipe+--+        
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
//! The [FlattenPipe] uses a loadable delay line to hold the array in a
//! set of chained flip flops.  The output is then clocked off the end
//! of the chain, one element at a time.  When it is empty, the delay
//! line can be reloaded from the input buffer.  Buffers at the input
//! and output eliminate combinatorial paths.  This design is a bit
//! register/flip flop heavy, so be careful with it's use.
//!
#![doc = badascii!(r"
        +IBuf+-----+        +-+unpck++                                                        
 ?[T;N] |          | ?[T;N] |        | [T;N]                                                  
+------>|data  data+------->|in  data+-------+---+---+                                        
        |          |        |        |       v   v   v        +--+pck+-+    +OBuf+-----+      
<-------+ready next|<----+  |     tag+-+   +------------+     |        | ?T |          | ?T   
        |          |     |  |        | |   | Delay Line +---->|data out+--->|data  data+----->
        +----------+     |  +--------+ |   |      run   |     |        |    |          |      
                         |             v   +------------+ +-->|tag     | +--+full ready|<----+
                         |  +-----------+          ^      |   |        | |  |          |      
                         |  |   Control +----------+      |   +--------+ |  +----------+      
                         +--+           +-----------------+              |                    
                            |           |<-------------------------------+                    
                            +-----------+                                                     
")]
//!
//! The control is governed by a simple two-state state machine.  The state diagram
//! is as follows:
#![doc = badascii!(r"
                           +---------+                          
                           |         |                          
   !full && cnt == N-1     | Loading |                          
     && !in_is_some     +->|         +--+    in_is_some         
   +----------------+  /   +---------+   \  +-----------+       
       cnt = 1        +                   +   next = 1          
       load = 0       |                   |   cnt = 0           
                      +                   +   load = 1          
                       \   +---------+   /                      
!full && cnt == N-1     +--+         |<-+                       
     && in_is_some         | Running |                          
+------------------+  +--->|         +-----+                    
    next = 1          |    |         |     | !full && cnt != N-1
    cnt = 0           |    |         |     | +-----------------+
    load = 1          |    |         |<----+    run = 1         
                      +----+         |          cnt += 1        
                           +-------+-+                          
                              ^    |                            
                              +----+                            
                               full                             
                              +----+                            
                               run=0                            
                               load=0                           
")]
//!# Example
//!
//! Here is an example of running the pipelined reducer.
//!
//!```
#![doc = include_str!("../../examples/flatten.rs")]
//!```
//!
//! with a trace file like this:
//!
#![doc = include_str!("../../doc/flatten.md")]
//!
use crate::{
    core::{dff, option::unpack},
    lid::{fifo_to_rv::FIFOToReadyValid, rv_to_fifo::ReadyValidToFIFO},
};

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use super::PipeIO;

#[derive(Debug, Default, PartialEq, Digital)]
#[doc(hidden)]
pub enum State {
    #[default]
    Loading,
    Running,
}

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The [FlattenPipe] Core
///
/// This core takes a stream of `[T; N]`, and produces
/// a stream of `T`, reading out the input stream in
/// index order (`0, 1, 2..`).  
pub struct FlattenPipe<M: BitWidth, T: Digital, const N: usize>
where
    [T; N]: Default,
    T: Default,
{
    input_buffer: ReadyValidToFIFO<[T; N]>,
    delay: [dff::DFF<T>; N],
    count: dff::DFF<Bits<M>>,
    output_buffer: FIFOToReadyValid<T>,
    state: dff::DFF<State>,
}

impl<M: BitWidth, T: Digital, const N: usize> Default for FlattenPipe<M, T, N>
where
    [T; N]: Default,
    T: Default,
{
    fn default() -> Self {
        assert!((1 << M::BITS) >= N, "Expect that the bitwidth of the counter is sufficient to count the elements in the array.  I.e., (1 << M) >= N");
        Self {
            delay: core::array::from_fn(|_| dff::DFF::default()),
            input_buffer: ReadyValidToFIFO::default(),
            count: dff::DFF::new(bits(0)),
            output_buffer: FIFOToReadyValid::default(),
            state: dff::DFF::new(State::Loading),
        }
    }
}

/// Inputs for the [FlattenPipe] core
pub type In<T, const N: usize> = PipeIO<[T; N]>;

/// Outputs from the [FlattenPipe] core
pub type Out<T> = PipeIO<T>;

impl<M: BitWidth, T: Digital, const N: usize> SynchronousIO for FlattenPipe<M, T, N>
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
pub fn kernel<M: BitWidth, T, const N: usize>(
    _cr: ClockReset,
    i: In<T, N>,
    q: Q<M, T, N>,
) -> (Out<T>, D<M, T, N>)
where
    [T; N]: Default,
    T: Digital + Default,
{
    let n_minus_1 = bits::<M>(N as u128 - 1);
    let mut d = D::<M, T, N>::dont_care();
    // Connect the input buffer to the input data stream
    d.input_buffer.data = i.data;
    // Do not advance the input buffer unless asked.
    d.input_buffer.next = false;
    // Control line to load the delay line from the
    // input buffer
    let mut load_line = false;
    // Control line to write the delay line output
    // to the output buffer (also advances the delay line)
    let mut write = false;
    // By default, do not change the count or state
    d.count = q.count;
    d.state = q.state;
    let out_full = q.output_buffer.full;
    let (in_some, idata) = unpack::<[T; N]>(q.input_buffer.data);
    // Update the state and compute transition actions
    match q.state {
        State::Loading => {
            if in_some {
                // Accept the input data
                d.input_buffer.next = true;
                // Load the data into the delay line
                load_line = true;
                // Reset the counter
                d.count = bits(0);
                d.state = State::Running;
            }
        }
        State::Running => {
            if !out_full {
                write = true;
                if q.count != n_minus_1 {
                    d.count = q.count + 1;
                } else if in_some {
                    // Finished, and on this write, we
                    // will load the next data (which is available)
                    d.input_buffer.next = true;
                    d.count = bits(0);
                    load_line = true;
                } else {
                    // No more data.  Go back to Loading
                    d.state = State::Loading;
                }
            }
        }
    }
    // By default, the delay line holds it's current
    // state
    for i in 0..N {
        d.delay[i] = q.delay[i];
    }
    if write {
        // The write signal indicates the delay line should
        // shift
        for i in 1..N {
            d.delay[i - 1] = q.delay[i]
        }
    }
    if load_line {
        // Reload the delay line from the input buffer
        for i in 0..N {
            d.delay[i] = idata[i]
        }
    }
    // Use the write flag to strobe data into the output FIFO
    d.output_buffer.data = if write { Some(q.delay[0]) } else { None };
    d.output_buffer.ready = i.ready;
    let o = Out::<T> {
        data: q.output_buffer.data,
        ready: q.input_buffer.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {

    use crate::rng::xorshift::XorShift128;

    use super::*;

    fn mk_array<T, const N: usize>(t: &mut impl Iterator<Item = T>) -> [T; N] {
        core::array::from_fn(|_| t.next().unwrap())
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = FlattenPipe::<U2, b4, 4>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation() -> miette::Result<()> {
        type Uut = FlattenPipe<U2, b4, 4>;
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
