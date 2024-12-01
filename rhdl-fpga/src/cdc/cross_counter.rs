use rhdl::prelude::*;

use crate::{
    core::dff,
    gray::{decode::gray_decode, encode::gray_code, Gray},
};

use super::synchronizer;

/// This core provides a counter where the input pulses
/// come from one clock domain, and the output count
/// is in a different clock domain.  The count in the output
/// clock domain is guaranteed to lag behind an equivalent count
/// in the input clock domain.  
///
/// SAFETY - this core uses a vector of 1-bit synchronizers, but
/// with a Gray-coded counter to cross the clock domains.  
/// This is safe because the first stage
/// of registers in the synchronizers will sample the Gray-coded signal
/// essentially simultaneously.  The Gray-coded signal is guaranteed to
/// have at most one bit changing at any time point.  Thus, all bits
/// will be correct when sampled with the possible exception of the
/// bit that is changing at that time.  This bit may resolve to the correct
/// value, or it may not.  If it does not, the transition will be missed
/// and the counter will be off by one.  However, at the next sample point,
/// this bit will be correct.  As the counter is monotonic, it will always
/// lag behind the actual count.
///
/// The W domain is used for the "writer" to the counter, where the
/// counter increments are provided, and the R domain is used for
/// the "reader" of the counter, where the count is read.
#[derive(Clone, Circuit, CircuitDQ)]
pub struct U<W: Domain, R: Domain, const N: usize> {
    // This counter lives in the W domain, and
    // counts the number of input pulses.
    counter: Adapter<dff::U<Bits<N>>, W>,
    // This is the vector of synchronizers, one per
    // bit of the counter.  The synchronizers hold
    // the value of the count in the read domain
    // as a gray encoded value.
    syncs: [synchronizer::U<W, R>; N],
}

impl<W: Domain, R: Domain, const N: usize> Default for U<W, R, N> {
    fn default() -> Self {
        Self {
            counter: Adapter::new(dff::U::default()),
            syncs: array_init::array_init(|_| synchronizer::U::default()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Digital, Timed)]
pub struct I<W: Domain, R: Domain, const N: usize> {
    /// The input data pulses to be counted from the W clock domain
    pub data: Signal<bool, W>,
    /// The clock and reset for the W clock domain
    pub data_cr: Signal<ClockReset, W>,
    /// The clock and reset for the output clock domain R
    pub cr: Signal<ClockReset, R>,
}

#[derive(Debug, Copy, Clone, PartialEq, Digital, Timed)]
pub struct O<R: Domain, const N: usize> {
    /// The count in the R domain (combinatorial decode of internal registers)
    pub count: Signal<Bits<N>, R>,
}

impl<W: Domain, R: Domain, const N: usize> CircuitIO for U<W, R, N> {
    type I = I<W, R, N>;
    type O = O<R, N>;
    type Kernel = cross_counter_kernel<W, R, N>;
}

#[kernel]
pub fn cross_counter_kernel<W: Domain, R: Domain, const N: usize>(
    input: I<W, R, N>,
    q: Q<W, R, N>,
) -> (O<R, N>, D<W, R, N>) {
    let mut d = D::<W, R, { N }>::dont_care();
    // The counter increments each time the input is high
    d.counter.clock_reset = input.data_cr;
    d.counter.input = signal(q.counter.val() + if input.data.val() { 1 } else { 0 });
    // The current counter output is gray coded
    let current_count = gray_code::<N>(q.counter.val()).0;
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
    let read_o = gray_decode::<N>(Gray::<N>(read_o));
    // The read side of the output comes from o, the
    // write side is simply the output of the internal counter
    let mut o = O::<R, { N }>::dont_care();
    o.count = signal(read_o);
    (o, d)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rand::random;

    use super::*;

    fn sync_stream() -> impl Iterator<Item = TimedSample<I<Red, Blue, 8>>> {
        // Start with a stream of pulses
        let red = (0..).map(|_| random::<bool>()).take(100);
        // Clock them on the green domain
        let red = red.stream_after_reset(1).clock_pos_edge(100);
        // Create an empty stream on the red domain
        let blue = std::iter::repeat(false)
            .stream_after_reset(1)
            .clock_pos_edge(79);
        // Merge them
        merge(red, blue, |r: (ClockReset, bool), b: (ClockReset, bool)| {
            I {
                data: signal(r.1),
                data_cr: signal(r.0),
                cr: signal(b.0),
            }
        })
    }

    #[test]
    fn test_performance() -> miette::Result<()> {
        type UC = U<Red, Blue, 8>;
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
        type UC = U<Red, Blue, 8>;
        let uut = UC::default();
        let input = sync_stream();
        let outputs = uut
            .run(input)?
            .sample_at_pos_edge(|t| t.value.0.cr.val().clock)
            .vcd_file(&PathBuf::from("rw_counter.vcd"))
            .map(|t| t.value.1.count.val())
            .collect::<Vec<_>>();
        outputs.windows(2).for_each(|w| {
            assert!(w[0] <= w[1]);
        });
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        type UC = U<Red, Blue, 8>;
        let uut = UC::default();
        let input = sync_stream();
        let test_bench = uut.run(input)?.collect::<TestBench<_, _>>();
        let test_mod = test_bench.rtl(
            &uut,
            &TestBenchOptions::default()
                .vcd("split_counter.vcd")
                .skip(10),
        )?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.flow_graph(
            &uut,
            &TestBenchOptions::default()
                .vcd("split_counter_fg.vcd")
                .skip(10),
        )?;
        test_mod.run_iverilog()?;
        Ok(())
    }
}
