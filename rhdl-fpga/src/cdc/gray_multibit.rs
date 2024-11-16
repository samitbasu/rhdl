use rhdl::prelude::*;

use crate::{core::dff, gray::Gray};

use super::synchronizer;

#[derive(Clone, Circuit, CircuitDQ)]
pub struct U<R: Domain, W: Domain, const N: usize> {
    // This counter lives in the R domain, and
    // counts the number of input pulses.
    counter: Adapter<dff::U<Bits<N>>, R>,
    // This is the vector of synchronizers, one per
    // bit of the counter.
    syncs: [synchronizer::U<R, W>; N],
}

impl<R: Domain, W: Domain, const N: usize> Default for U<R, W, N> {
    fn default() -> Self {
        Self {
            counter: Adapter::new(dff::U::default()),
            syncs: array_init::array_init(|_| synchronizer::U::default()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Digital, Timed)]
pub struct I<R: Domain, W: Domain, const N: usize> {
    pub data: Signal<bool, R>,
    pub data_cr: Signal<ClockReset, R>,
    pub cr: Signal<ClockReset, W>,
}

impl<R: Domain, W: Domain, const N: usize> CircuitIO for U<R, W, N> {
    type I = I<R, W, N>;
    type O = Signal<Bits<N>, W>;
    type Kernel = gray_kernel<R, W, N>;
}

#[kernel]
fn gray_code<const N: usize>(i: Bits<N>) -> Gray<N> {
    Gray::<N>(i ^ (i >> 1))
}

#[kernel]
pub fn gray_decode<const N: usize>(i: Gray<N>) -> Bits<N> {
    let mut o = i.0;
    o ^= o >> 1;
    if ({ N } > 2) {
        o ^= o >> 2;
    }
    if ({ N } > 4) {
        o ^= o >> 4;
    }
    if ({ N } > 8) {
        o ^= o >> 8;
    }
    if ({ N } > 16) {
        o ^= o >> 16;
    }
    if ({ N } > 32) {
        o ^= o >> 32;
    }
    if ({ N } > 64) {
        o ^= o >> 64;
    }
    o
}

#[kernel]
pub fn gray_kernel<R: Domain, W: Domain, const N: usize>(
    input: I<R, W, N>,
    q: Q<R, W, N>,
) -> (Signal<Bits<N>, W>, D<R, W, N>) {
    let mut d = D::<R, W, { N }>::init();
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
    // Connect each synchronizer output to one bit of the output
    let mut o = bits(0);
    for i in 0..N {
        if q.syncs[i].val() {
            o |= bits(1 << i);
        }
    }
    // Decode this signal back to a binary count
    let o = gray_decode::<N>(Gray::<N>(o));
    (signal(o), d)
}

#[cfg(test)]
mod tests {
    use rand::random;

    use super::*;

    fn sync_stream() -> impl Iterator<Item = TimedSample<I<Red, Green, 8>>> {
        // Start with a stream of pulses
        let green = (0..).map(|_| random::<bool>()).take(500);
        // Clock them on the green domain
        let green = clock_pos_edge(stream(green), 100);
        // Create an empty stream on the red domain
        let red = stream(std::iter::repeat(false));
        let red = clock_pos_edge(red, 79);
        // Merge them
        merge(
            green,
            red,
            |g: (ClockReset, bool), r: (ClockReset, bool)| I {
                data: signal(g.1),
                data_cr: signal(g.0),
                cr: signal(r.0),
            },
        )
    }

    #[test]
    fn test_performance() {
        type UC = U<Red, Green, 8>;
        let uut = UC::default();
        let input = sync_stream();
        validate(
            &uut,
            input,
            &mut [glitch_check::<UC>(|i| i.value.cr.val().clock)],
            ValidateOptions::default().vcd("gray_sync.vcd"),
        )
    }

    #[test]
    fn test_hdl_generation_rtl() -> miette::Result<()> {
        type UC = U<Red, Green, 8>;
        let uut = UC::default();
        let options = TestModuleOptions {
            skip_first_cases: 10,
            vcd_file: Some("gray_sync_rtl.vcd".into()),
            flow_graph_level: false,
            hold_time: 1,
        };
        let stream = sync_stream();
        let test_mod = build_rtl_testmodule(&uut, stream, options)?;
        std::fs::write("gray_sync_rtl.v", test_mod.to_string()).unwrap();
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_hdl_generation_fg() -> miette::Result<()> {
        type UC = U<Red, Green, 8>;
        let uut = UC::default();
        let options = TestModuleOptions {
            skip_first_cases: 10,
            vcd_file: Some("gray_sync.vcd".into()),
            flow_graph_level: true,
            hold_time: 1,
        };
        let stream = sync_stream();
        let test_mod = build_rtl_testmodule(&uut, stream, options)?;
        std::fs::write("gray_sync.v", test_mod.to_string()).unwrap();
        test_mod.run_iverilog()?;
        Ok(())
    }
}
