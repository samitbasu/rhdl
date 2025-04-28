//!# Clock domain crossing counter
//!
//!# Purpose
//!
//! This core provides a counter where the input pulses
//! come from one clock domain, and the output count
//! is in a different clock domain.  The count in the output
//! clock domain is guaranteed to lag behind an equivalent count
//! in the input clock domain.  
//!
//!# Connections
//!
//! The W domain is used for the "writer" to the counter, where the
//! counter increments are provided, and the R domain is used for
//! the "reader" of the counter, where the count is read.
//!
//!
#![doc = badascii_doc::badascii_formal!("
               +-----------------+                 
               |                 |                 
               |                 |                 
          +--->| incr      count +--->             
    W          |                 |         R       
  domain       |                 |      domain     
          +--->| incr_cr      cr |<---+            
               |                 |                 
               |                 |                 
               +-----------------+                 
")]
//!
//! The use is straight forward.  In the domain where the pulses to be counted
//! originate, you need to provide a clock and reset signal.  You also need
//! to provide a clock and reset in the domain where the count is provided.
//!
//!# Internals
//!
//! This core uses a vector of 1-bit synchronizers, but
//! with a Gray-coded counter to cross the clock domains.  
//! This is safe because the first stage
//! of registers in the synchronizers will sample the Gray-coded signal
//! essentially simultaneously.  The Gray-coded signal is guaranteed to
//! have at most one bit changing at any time point.  Thus, all bits
//! will be correct when sampled with the possible exception of the
//! bit that is changing at that time.  This bit may resolve to the correct
//! value, or it may not.  If it does not, the transition will be missed
//! and the counter will be off by one.  However, at the next sample point,
//! this bit will be correct.  As the counter is monotonic, it will always
//! lag behind the actual count.
//!
//! Here is a rough diagram of the contents of the block.
#![doc = badascii_doc::badascii!("
                                     Combinatorial Blocks                      
                               ++------------------------------+               
                               v                               v               
Input   +-+Counter+-+                     +---------+                         
incr    |           |    +----------+  +->|1-bit cdc+--+  +--------+           
  +---->|incr  count+--->|Gray Coder++-+  +---------+  |  |        |           
        |           |    +----------+  |       :  ^    +->| Gray   |           
        |           |                  |       :  +---+|  | Decode +--> Output 
Input   |           |                  |  +---------+ ||  |        |    count  
 cr +-->|clk/rst    |                  +->|1-bit cdc+-++  +--------+           
        +-----------+                     +---------+ |                        
                                               ^      |                        
                                               +------+---------------+ target 
                                                                        clk/rst
                                               +                               
                           Synchronous to   <+ | +>  Synchronous to            
                           input clock  W      +       output clock  R          
")]
//!
//! The counter is synchronous the input domain.  The output count is fed
//! into a Gray coder, which ensures only one bit changes at a time for each
//! input count.  The individual bits of the Gray coder can then be passed
//! through 1-bit synchronizers, as only one bit changes at a time.  The output
//! is then combinatorially decoded into a count.  Note that the output contains
//! combinatorial delays to the output DFFs of the CDCs.  A pipeline stage may
//! be needed to isolate that logic if high speed is required.
//!
//!# Example
//!
//!```
#![doc = include_str!("../../examples/cross_counter.rs")]
//!```
#![doc = include_str!("../../doc/cross_counter.md")]

use rhdl::prelude::*;

use crate::{
    core::dff,
    gray::{decode::gray_decode, encode::gray_code, Gray},
};

use super::synchronizer;

#[derive(Clone, Circuit, CircuitDQ)]
/// Unit to instantiate.
///
/// The type parameters are:
///   - `W`: The domain where the input pulses come from
///   - `R`: The domain where the output count is provided
///   - `N`: The number of bits in the counter
pub struct CrossCounter<W: Domain, R: Domain, const N: usize>
where
    Const<N>: BitWidth,
{
    /// This counter lives in the W domain, and
    /// counts the number of input pulses.
    counter: Adapter<dff::DFF<Bits<Const<N>>>, W>,
    /// This is the vector of synchronizers, one per
    /// bit of the counter.  The synchronizers hold
    /// the value of the count in the read domain
    /// as a gray encoded value.
    syncs: [synchronizer::Sync1Bit<W, R>; N],
}

impl<W: Domain, R: Domain, const N: usize> Default for CrossCounter<W, R, N>
where
    Const<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            counter: Adapter::new(dff::DFF::default()),
            syncs: array_init::array_init(|_| synchronizer::Sync1Bit::default()),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Timed)]
/// Inputs to the core
pub struct In<W: Domain, R: Domain> {
    /// The input data pulses to be counted from the W clock domain
    pub incr: Signal<bool, W>,
    /// The clock and reset for the W clock domain
    pub incr_cr: Signal<ClockReset, W>,
    /// The clock and reset for the output clock domain R
    pub cr: Signal<ClockReset, R>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
/// Outputs from the core
pub struct Out<R: Domain, const N: usize>
where
    Const<N>: BitWidth,
{
    /// The count in the R domain (combinatorial decode of internal registers)
    pub count: Signal<Bits<Const<N>>, R>,
}

impl<W: Domain, R: Domain, const N: usize> CircuitIO for CrossCounter<W, R, N>
where
    Const<N>: BitWidth,
{
    type I = In<W, R>;
    type O = Out<R, N>;
    type Kernel = cross_counter_kernel<W, R, N>;
}

#[kernel]
/// The kernel function for the cross-counter.
pub fn cross_counter_kernel<W: Domain, R: Domain, const N: usize>(
    input: In<W, R>,
    q: Q<W, R, N>,
) -> (Out<R, N>, D<W, R, N>)
where
    Const<N>: BitWidth,
{
    let mut d = D::<W, R, { N }>::dont_care();
    // The counter increments each time the input is high
    d.counter.clock_reset = input.incr_cr;
    d.counter.input = signal(q.counter.val() + if input.incr.val() { 1 } else { 0 });
    // The current counter output is gray coded
    let current_count = gray_code::<Const<N>>(q.counter.val()).0;
    // Each synchronizer is fed a bit from the gray coded count
    for i in 0..N {
        d.syncs[i].data = signal((current_count & (1 << i)) != 0);
        // The clock to the synchronizer is the destination clock
        d.syncs[i].cr = input.cr;
    }
    // Connect each synchronizer output to one bit of the output on the read side
    let mut read_o = bits(0);
    for i in 0..N {
        if q.syncs[i].val() {
            read_o |= bits(1 << i);
        }
    }
    // Decode this signal back to a binary count
    let read_o = gray_decode::<Const<N>>(Gray::<Const<N>>(read_o));
    // The read side of the output comes from o, the
    // write side is simply the output of the internal counter
    let mut o = Out::<R, { N }>::dont_care();
    o.count = signal(read_o);
    (o, d)
}

#[cfg(test)]
mod tests {

    use rand::random;

    use super::*;

    fn sync_stream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
        // Start with a stream of pulses
        let red = (0..).map(|_| random::<bool>()).take(100);
        // Clock them on the green domain
        let red = red.stream_after_reset(1).clock_pos_edge(100);
        // Create an empty stream on the red domain
        let blue = std::iter::repeat(())
            .stream_after_reset(1)
            .clock_pos_edge(79);
        // Merge them
        merge(red, blue, |r: (ClockReset, bool), b: (ClockReset, ())| In {
            incr: signal(r.1),
            incr_cr: signal(r.0),
            cr: signal(b.0),
        })
    }

    #[test]
    fn test_performance() -> miette::Result<()> {
        type UC = CrossCounter<Red, Blue, 8>;
        let uut = UC::default();
        let input = sync_stream();
        let _ = uut
            .run(input)?
            .glitch_check(|t| (t.value.0.cr.val().clock, t.value.1.count))
            .last();
        Ok(())
    }

    #[test]
    fn test_read_counter_is_monotonic() -> miette::Result<()> {
        type UC = CrossCounter<Red, Blue, 8>;
        let uut = UC::default();
        let input = sync_stream();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("cross");
        std::fs::create_dir_all(&root).unwrap();
        let outputs = uut
            .run(input)?
            .sample_at_pos_edge(|t| t.value.0.cr.val().clock)
            .vcd_file(&root.join("rw_counter.vcd"))
            .map(|t| t.value.1.count.val())
            .collect::<Vec<_>>();
        outputs.windows(2).for_each(|w| {
            assert!(w[0] <= w[1]);
        });
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        type UC = CrossCounter<Red, Blue, 8>;
        let uut = UC::default();
        let input = sync_stream();
        let test_bench = uut.run(input)?.collect::<TestBench<_, _>>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("cross");
        std::fs::create_dir_all(&root).unwrap();
        let test_mod = test_bench.rtl(
            &uut,
            &TestBenchOptions::default()
                .vcd(&root.join("split_counter.vcd").to_string_lossy())
                .skip(10),
        )?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.flow_graph(
            &uut,
            &TestBenchOptions::default()
                .vcd(&root.join("split_counter_fg.vcd").to_string_lossy())
                .skip(10),
        )?;
        test_mod.run_iverilog()?;
        Ok(())
    }
}
