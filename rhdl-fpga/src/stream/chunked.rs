//! Chunked Stream Core
//!
//!# Purpose
//!
//! A [Chunked] Stream Core takes a sequence of `T` data elements
//! and chunks them into an array of `N`.  It is roughly equivalent
//! to the `.chunks()` method on slices.  Note that each chunk
//! will contain a disjoint set of samples.  
//!
#![doc = badascii!(r"
      t0  t1  t2  t3  t4  t5  t6  t7  t8 ...
                                            
 in   d0  d1  d2  d3  d4  d5  d6  d7  d8    
                                            
out               [d0..d3]         [d4..d7] 
")]
//! If you want a sliding window, use the [WindowedPipe] Core instead.
//!
//!# Schematic Symbol
//!
//! Here is the schematic symbol for the [ChunkedPipe] core.
//!
#![doc = badascii_formal!("
     ++Chunked+-----+        
 ?T  |              | ?[T;N] 
+--->|data      data+------->
     |              |        
<----+ready    ready|<------+
     |              |        
     +--------------+        
")]
//!
//!# Internals
//!
//! Roughly, the internal of the [Chunked] core includes
//! a pipeline delay stage, along with taps to extract the
//! delayed signals.  Buffers are needed at the input and
//! output to isolate the combinatorial signals from each other.
//!
//! Note, in particular, that the `run` signal depends on the validity
//! of the input `data` element.  Without an input buffer, we would have
//! a combinatorial path between the input and output.  
//!
#![doc = badascii!(r"
                      ++unpck+-+    ++TappedDelay+---+ [T;N]  ++pck++      ++Fifo2St+-+      
     ++St2Fifo+-+     |        |  T |             out+------->|data |?[T;N]|          |      
 ?T  |          | ?T  |    data+--->|in     run      |        |  out+----->|data  data+----->
+--->|data  data+---->|in      |    +----------------+   +--->|tag  |      |          |      
     |          |     |     tag+--+          ^           |    |     |   +--+full ready|<----+
<----+ready next|<-+  |        |  |   +------+-----+     |    +-----+   |  |          |      
     |          |  |  +--------+  +-->|  Control   +-----+              |  +----------+      
     +----------+  |         =run     +-+----------+                    |                    
                   +--------------------+      ^                        |                    
                                               +------------------------+                    
")]
//!
//! The two pipelines (upstream and downstream) are connected with the buffered
//! tapped-delay line.  The control system is a simple two-state state machine.
#![doc = badascii!("
                     +------+                                  
           !in_some  |      v   +-------+ in_some && cnt != N-1
                     |    +-----+-+     | +-------------------+
                     +----+Loading|<----+   run = 1, cnt += 1  
                       +->|       |                            
in_some && !out_full   |  +-----+-+ in_some && cnt == N-1      
+------------------+   |        |   +-------------------+      
 cnt = 1, run = 1      |        |      run = 1, cnt = 0        
  do_write = 1         |        v                              
                       |     +----+                            
!in_some && !out_full  +-----+Full+--+                         
+--------------------+       +----+  |                         
 cnt = 0, run = 0               ^    | out_full                
  do_write = 1                  +----+                         
")]
//!
//!# Example
//!
//! The example includes some of the testing tools for generating and
//! sinking Pipe cores.  These are not synthesizable, but are handy for
//! testing and verification exercises.
//!
//!```
#![doc = include_str!("../../examples/chunk.rs")]
//!```
//!
//! The output trace demonstrates the core in action.
#![doc = include_str!("../../doc/chunk.md")]

use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::{
    core::{
        dff,
        option::{is_some, unpack},
    },
    stream::{fifo_to_stream, stream_to_fifo},
};

use super::StreamIO;

#[derive(Debug, Default, PartialEq, Digital)]
#[doc(hidden)]
pub enum State {
    #[default]
    Loading,
    Full,
}

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The Chunked Stream Core
///
/// This core takes a stream of `T` and produces
/// a stream of chunks `[T;N]`, assembling the array
/// in index order, so that `t0, t1, t2,...` are
/// packed such that the `out[0] = t0`, etc.
/// Note that `M` is a bitwidth for the internal counter
/// and must satisfy `1 << M <= N`.
pub struct Chunked<M: BitWidth, T: Digital, const N: usize>
where
    [T; N]: Default,
    T: Default,
{
    input_buffer: stream_to_fifo::StreamToFIFO<T>,
    delay_line: [dff::DFF<T>; N],
    count: dff::DFF<Bits<M>>,
    output_buffer: fifo_to_stream::FIFOToStream<[T; N]>,
    state: dff::DFF<State>,
}

impl<M: BitWidth, T: Digital, const N: usize> Default for Chunked<M, T, N>
where
    [T; N]: Default,
    T: Default,
{
    fn default() -> Self {
        assert!(N > 1, "Can only chunk streams with N > 1");
        assert!((1 << M::BITS) >= N, "Expect that the bitwidth of the counter is sufficiently large to express values up to N");
        Self {
            input_buffer: stream_to_fifo::StreamToFIFO::default(),
            delay_line: core::array::from_fn(|_| dff::DFF::default()),
            count: dff::DFF::new(bits(0)),
            output_buffer: fifo_to_stream::FIFOToStream::default(),
            state: dff::DFF::new(State::Loading),
        }
    }
}

/// Inputs for the [Chunked] core
pub type In<T> = StreamIO<T>;

/// Outputs from the [Chunked] core
pub type Out<T, const N: usize> = StreamIO<[T; N]>;

impl<M: BitWidth, T: Digital, const N: usize> SynchronousIO for Chunked<M, T, N>
where
    [T; N]: Default,
    T: Default,
{
    type I = In<T>;
    type O = Out<T, N>;
    type Kernel = kernel<M, T, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<M: BitWidth, T, const N: usize>(
    _cr: ClockReset,
    i: In<T>,
    q: Q<M, T, N>,
) -> (Out<T, N>, D<M, T, N>)
where
    [T; N]: Default,
    T: Default + Digital,
{
    let n_minus_1 = bits::<M>(N as u128 - 1);
    let mut d = D::<M, T, N>::dont_care();
    d.input_buffer.data = i.data;
    d.output_buffer.ready = i.ready;
    let mut write = false;
    let mut run = false;
    d.count = q.count;
    d.state = q.state;
    let out_full = q.output_buffer.full;
    // Update the state and compute transition actions
    d.delay_line[0] = q.delay_line[0];
    match q.state {
        State::Loading => {
            if let Some(idata) = q.input_buffer.data {
                if q.count != n_minus_1 {
                    run = true;
                    d.count = q.count + 1;
                } else {
                    run = true;
                    d.state = State::Full;
                }
                d.delay_line[0] = idata;
            }
        }
        State::Full => {
            if !out_full {
                write = true;
                d.state = State::Loading;
                if let Some(idata) = q.input_buffer.data {
                    d.count = bits(1);
                    d.delay_line[0] = idata;
                    run = true;
                } else {
                    d.count = bits(0);
                }
            }
        }
    }
    // Implement the delay line
    for i in 1..N {
        d.delay_line[i] = if run {
            q.delay_line[i - 1]
        } else {
            q.delay_line[i]
        }
    }
    // Feed the run signal to the input buffer
    d.input_buffer.next = run;
    // Feed the tapped delay line output to the
    // output buffer, modulated by the write signal
    d.output_buffer.data = if write {
        let mut tmp = <[T; N]>::dont_care();
        for i in 0..N {
            tmp[N - 1 - i] = q.delay_line[i]
        }
        Some(tmp)
    } else {
        None
    };
    let o = Out::<T, N> {
        data: q.output_buffer.data,
        ready: q.input_buffer.ready,
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use crate::rng::xorshift::XorShift128;

    use super::*;

    fn mk_array<T, const N: usize>(mut t: impl Iterator<Item = T>) -> impl Iterator<Item = [T; N]> {
        std::iter::from_fn(move || Some(core::array::from_fn(|_| t.next().unwrap())))
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = Chunked::<U2, b4, 4>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation_n_is_2() -> miette::Result<()> {
        test_operation_for_n::<U1, 2>()?;
        Ok(())
    }

    #[test]
    fn test_operation_n_is_4() -> miette::Result<()> {
        test_operation_for_n::<U2, 4>()?;
        Ok(())
    }

    fn test_operation_for_n<M: BitWidth, const N: usize>() -> miette::Result<()>
    where
        [b4; N]: Default,
    {
        let uut = Chunked::<M, b4, N>::default();
        let mut need_reset = true;
        let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let dest_rng = source_rng.clone();
        let mut dest_rng = mk_array(dest_rng);
        let mut latched_input: Option<b4> = None;
        uut.run_fn(
            move |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = super::In::<b4>::dont_care();
                // Downstream is likely to run
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = !want_to_pause;
                // Decide if the producer will generate a new data item
                let willing_to_send = rand::random::<u8>() < 200;
                if out.ready {
                    // The pipeline wants more data
                    if willing_to_send {
                        latched_input = source_rng.next();
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
