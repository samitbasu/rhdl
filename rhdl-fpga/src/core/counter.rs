use rhdl::prelude::*;

use super::dff;

// A simple counter that counts the number of boolean true
// values it has seen.  It is parameterized by the number of
// bits in the counter.
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
    use rand::random;

    use super::*;
    use std::iter::{once, repeat};

    #[test]
    fn test_counter() {
        let reset_0 = stream::reset_pulse(4);
        let inputs_1 = stream::stream(repeat(true).take(100));
        let reset_1 = stream::reset_pulse(4);
        let inputs_2 = stream::stream(repeat(true).take(100));
        let stream = clock_pos_edge(reset_0.chain(inputs_1.chain(reset_1.chain(inputs_2))), 100);
        let uut: U<16> = U::default();
        simple_traced_synchronous_run(&uut, stream, "strobe.vcd");
    }

    #[test]
    fn test_counter_counts_correctly() {
        // To account for the delay, we need to end with a zero input
        let rand_set = (0..100)
            .map(|_| random::<bool>())
            .chain(once(false))
            .collect::<Vec<bool>>();
        let ground_truth = rand_set
            .iter()
            .fold(0, |acc, x| acc + if *x { 1 } else { 0 });
        let reset_0 = stream::reset_pulse(4);
        let inputs_1 = stream::stream(rand_set.into_iter());
        let stream = clock_pos_edge(reset_0.chain(inputs_1), 100);
        let uut: U<16> = U::default();
        let output = final_output_synchronous_simulation(&uut, stream);
        assert_eq!(output, Some(bits(ground_truth)));
    }
}
