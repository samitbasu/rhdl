//! Chunked Pipe Core
//!
//!# Purppose
//!
//! A [Chunked] Pipe Core takes a sequence of `T` data elements
//! and chunks them into an array of size `N`.  It is equivalent to
//! calling `.chunks()` on slices.  Note that each chunk will contain
//! a disjoint set of samples.
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
     ++Chunked+---+        
 ?T  |            | ?[T;N] 
+--->|data    data+------->
     +------------+        
")]
//!
//!# Internals
//!
//! The [Chunked] core includes a pipeline delay stage with taps to
//! extract the delayed signals.  An input buffer is needed at the
//! input to isolate the delay chain from the input pipeline.
//!  
#![doc = badascii!(r"
                  ++unpck+-+    ++TappedDelay+---+ [T;N]  ++pck++      
     ++DFF+-+     |        |  T |             out+------->|data |?[T;N]
 ?T  |      | ?T  |    data+--->|in     run      |        |  out+----->
+--->|d    q+---->|in      |    +----------------+      +>|tag  |      
     +------+     |     tag+--+          ^              | |     |      
                  |        |  |   +------+---+   ++DFF++| +-----+      
                  +--------+  +-->|  Control +-->|d   q++              
                                  +-+--------+   +-----+               
")]
//!
//! The control system simply counts the number of valid elements
//! in the tapped delay line and sets the output `tag` to `true`
//! for one cycle when the count is equal to `N`.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/pipe_chunked.rs")]
//!```
//!
#![doc = include_str!("../../doc/pipe_chunked.md")]
//!
//!
use badascii_doc::{badascii, badascii_formal};
use rhdl::prelude::*;

use crate::core::{dff::DFF, option::is_some};

#[derive(Debug, Clone, Synchronous, SynchronousDQ)]
/// The Chunked Pipe Core
///
/// This core takes a pipeline of `T` and produces
/// a pipeline of `[T; N]`, assembling the array in
/// index order, so that `t0, t1, t2, ...` are
/// packed such that `out[0] = t0`, etc.
pub struct Chunked<T: Digital, const M: usize, const N: usize>
where
    rhdl::bits::W<M>: BitWidth,
    rhdl::bits::W<N>: BitWidth,
{
    input: DFF<Option<T>>,
    delay_line: [DFF<T>; N],
    count: DFF<Bits<M>>,
    valid: DFF<bool>,
}

impl<T: Digital, const M: usize, const N: usize> Default for Chunked<T, M, N>
where
    rhdl::bits::W<M>: BitWidth,
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        assert!(N > 1, "Can only chunk streams with N > 1");
        assert!((1 << M) >= N, "Expect that the bitwidth of the counter is sufficiently large to express values up to N");
        Self {
            input: DFF::new(None),
            delay_line: core::array::from_fn(|_| DFF::new(T::dont_care())),
            count: DFF::new(bits(0)),
            valid: DFF::new(false),
        }
    }
}

/// Inputs for the [Chunked] Pipe
pub type In<T> = Option<T>;

/// Outputs for the [Chunked] Pipe
pub type Out<T, const N: usize> = Option<[T; N]>;

impl<T: Digital, const M: usize, const N: usize> SynchronousIO for Chunked<T, M, N>
where
    rhdl::bits::W<M>: BitWidth,
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T>;
    type O = Out<T, N>;
    type Kernel = kernel<T, M, N>;
}

#[kernel]
#[doc(hidden)]
pub fn kernel<T: Digital, const M: usize, const N: usize>(
    _cr: ClockReset,
    i: In<T>,
    q: Q<T, M, N>,
) -> (Out<T, N>, D<T, M, N>)
where
    rhdl::bits::W<M>: BitWidth,
    rhdl::bits::W<N>: BitWidth,
{
    let n_minus_1 = bits::<M>(N as u128 - 1);
    let mut d = D::<T, M, N>::dont_care();
    d.input = i;
    let run = is_some::<T>(q.input);
    d.count = q.count;
    d.valid = false;
    if run {
        if q.count == n_minus_1 {
            d.count = bits(0);
            d.valid = true;
        } else {
            d.count = q.count + 1;
        }
    }
    // Implement the delay line
    d.delay_line[0] = q.delay_line[0];
    if run {
        if let Some(idata) = q.input {
            d.delay_line[0] = idata;
        }
    }
    for i in 1..N {
        d.delay_line[i] = if run {
            q.delay_line[i - 1]
        } else {
            q.delay_line[i]
        }
    }
    let o = if q.valid {
        let mut tmp = <[T; N]>::dont_care();
        for i in 0..N {
            tmp[N - 1 - i] = q.delay_line[i]
        }
        Some(tmp)
    } else {
        None
    };
    (o, d)
}

#[cfg(test)]
mod tests {
    use crate::{rng::xorshift::XorShift128, stream::testing::utils::stalling};

    use super::*;

    fn mk_array<T, const N: usize>(mut t: impl Iterator<Item = T>) -> impl Iterator<Item = [T; N]> {
        std::iter::from_fn(move || Some(core::array::from_fn(|_| t.next().unwrap())))
    }

    #[test]
    fn test_no_combinatorial_paths() -> miette::Result<()> {
        let uut = Chunked::<b4, 2, 4>::default();
        drc::no_combinatorial_paths(&uut)?;
        Ok(())
    }

    #[test]
    fn test_operation_n_is_2() -> miette::Result<()> {
        test_operation_for_n::<1, 2>()?;
        Ok(())
    }

    #[test]
    fn test_operation_n_is_4() -> miette::Result<()> {
        test_operation_for_n::<2, 4>()?;
        Ok(())
    }

    fn test_operation_for_n<const M: usize, const N: usize>() -> miette::Result<()>
    where
        [b4; N]: Default,
        rhdl::bits::W<M>: BitWidth,
        rhdl::bits::W<N>: BitWidth,
    {
        let uut = Chunked::<b4, M, N>::default();
        let source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let expected = source_rng.clone();
        let expected = mk_array(expected);
        let input = stalling(source_rng, 0.23).with_reset(1).clock_pos_edge(100);
        let output = uut.run(input).synchronous_sample().filter_map(|t| t.output);
        assert!(output.take(1_000).eq(expected.take(1_000)));
        Ok(())
    }

    #[test]
    fn test_basic() -> Result<(), RHDLError> {
        let uut = Chunked::<b4, 2, 4>::default();
        let source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
        let input = stalling(source_rng, 0.23)
            .with_reset(1)
            .clock_pos_edge(100)
            .take(100);
        let output = uut.run(input).collect::<VcdFile>();
        output.dump_to_file("chunked_pipe.vcd")?;
        Ok(())
    }
}
