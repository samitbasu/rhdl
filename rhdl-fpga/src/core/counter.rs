use rhdl::prelude::*;

use super::dff;

// A simple counter that increments by one each cycle when the
// input enable signal is high.  It is parameterized by the number of
// bits in the counter.  It will wrap around to zero when it reaches
// all ones.
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<const N: usize> {
    count: dff::U<Bits<N>>,
}

impl<const N: usize> Default for U<N> {
    fn default() -> Self {
        Self {
            count: dff::U::new(Bits::<N>::default()),
        }
    }
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = bool;
    type O = Bits<N>;
    type Kernel = counter<N>;
}

#[kernel]
pub fn counter<const N: usize>(cr: ClockReset, enable: bool, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if enable { q.count + 1 } else { q.count };
    let next_count = if cr.reset.any() { bits(0) } else { next_count };
    (next_count, D::<{ N }> { count: next_count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat;

    #[test]
    fn test_counter() {
        let reset_0 = stream::reset_pulse(4);
        let inputs_1 = stream::stream(repeat(true).take(100));
        let reset_1 = stream::reset_pulse(4);
        let inputs_2 = stream::stream(repeat(true).take(100));
        let stream = clock_pos_edge(reset_0.chain(inputs_1.chain(reset_1.chain(inputs_2))), 100);
        let uut: U<16> = U::default();
        traced_synchronous_simulation(&uut, stream, "strobe.vcd");
    }
}
