use rhdl::prelude::*;

use crate::gray::Gray;

use super::synchronizer;

#[derive(Debug, Clone, Circuit, CircuitDQ)]
pub struct U<R: Domain, W: Domain, const N: usize> {
    syncs: [synchronizer::U<R, W>; N],
}

impl<R: Domain, W: Domain, const N: usize> Default for U<R, W, N> {
    fn default() -> Self {
        Self {
            syncs: array_init::array_init(|_| synchronizer::U::default()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Digital, Timed)]
pub struct I<R: Domain, W: Domain, const N: usize> {
    pub data: Signal<Gray<N>, R>,
    pub cr: Signal<ClockReset, W>,
}

impl<R: Domain, W: Domain, const N: usize> CircuitIO for U<R, W, N> {
    type I = I<R, W, N>;
    type O = Signal<Gray<N>, W>;
    type Kernel = gray_kernel<R, W, N>;
}

#[kernel]
pub fn gray_kernel<R: Domain, W: Domain, const N: usize>(
    input: I<R, W, N>,
    q: Q<R, W, N>,
) -> (Signal<Gray<N>, W>, D<R, W, N>) {
    let mut d = D::<R, W, { N }>::init();
    // Connect each synchronizer to one bit of the input
    let raw = input.data.val().0;
    for i in 0..N {
        d.syncs[i].cr = input.cr;
        d.syncs[i].data = signal((raw & (1 << i)) != 0);
    }
    // Connect each synchronizer output to one bit of the output
    let mut o = bits(0);
    for i in 0..N {
        if q.syncs[i].val() {
            o |= bits(1 << i);
        }
    }
    (signal(Gray::<N>(o)), d)
}

#[cfg(test)]
mod tests {
    use rand::random;

    // TODO - fix this later...
    fn gray_code<const N: usize>(i: Bits<N>) -> Gray<N> {
        Gray::<N>(i ^ (i >> 1))
    }

    use super::*;

    fn sync_stream() -> impl Iterator<Item = TimedSample<I<Red, Green, 8>>> {
        // Start with a linear counter
        let green = (0..).map(|x| bits(x % 256)).take(500);
        // Convert to Gray code
        let green = green.map(|x| gray_code::<8>(x));
        let green = clock_pos_edge(stream(green), 100);
        let red = stream(std::iter::repeat(false));
        let red = clock_pos_edge(red, 79);
        merge(
            green,
            red,
            |g: (ClockReset, Gray<8>), r: (ClockReset, bool)| I {
                data: signal(g.1),
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
    fn test_hdl_generation() -> miette::Result<()> {
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
