//! A simple counter
//!
//! A simple counter that counts the number of boolean true
//! values it has seen.  It is parameterized by the number of
//! bits in the counter.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!("
      +-+Counter+-------+      
      |                 |      
 bool |                 |B<N>  
+---->+ input     count +----->
      |                 |      
      +-----------------+      
")]
//!
//!# Example
//!
//! Here's a simple example of a counter.
//!```
#![doc = include_str!("../../examples/counter.rs")]
//!```
//! The trace below demonstrates the result.
#![doc = include_str!("../../doc/counter.md")]
use rhdl::prelude::*;

use super::dff;

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
/// The counter core
///   `N` is the bitwidth of the counter
pub struct Counter<N: BitWidth> {
    count: dff::DFF<Bits<N>>,
}

impl<N: BitWidth> Default for Counter<N> {
    fn default() -> Self {
        Self {
            count: dff::DFF::new(Bits::<N>::default()),
        }
    }
}

impl<N: BitWidth> SynchronousIO for Counter<N> {
    type I = bool;
    type O = Bits<N>;
    type Kernel = counter<N>;
}

#[kernel]
/// Counter kernel function
pub fn counter<N: BitWidth>(cr: ClockReset, enable: bool, q: Q<N>) -> (Bits<N>, D<N>) {
    let next_count = if enable { q.count + 1 } else { q.count };
    let next_count = if cr.reset.any() { bits(0) } else { next_count };
    (q.count, D::<N> { count: next_count })
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rand::random;

    use super::*;
    use std::{iter::once, path::PathBuf};

    #[test]
    fn test_counter_on_vec() -> miette::Result<()> {
        let inputs = (0..100).map(|_| random::<bool>()).collect::<Vec<_>>();
        let inputs = inputs.with_reset(4);
        let inputs = inputs.clock_pos_edge(100);
        let inputs = inputs.collect::<Vec<_>>();
        let uut: Counter<U6> = Counter::default();
        let output = uut.run(inputs)?.count();
        assert_eq!(output, 311);
        Ok(())
    }

    #[test]
    fn test_counter() -> miette::Result<()> {
        let inputs_1 = std::iter::repeat_n(true, 100).with_reset(4);
        let inputs_2 = inputs_1.clone();
        let input = inputs_1.chain(inputs_2);
        let input = input.clock_pos_edge(100);
        let uut: Counter<U6> = Counter::default();
        let vcd: Vcd = uut.run(input)?.collect();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("counter");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["c4eaeed085952d1bddf969d706befee53583cd65545171da0f83892a269f8111"];
        let digest = vcd.dump_to_file(root.join("counter.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_counter_counts_correctly() -> miette::Result<()> {
        // To account for the delay, we need to end with a zero input
        let rand_set = (0..100)
            .map(|_| random::<bool>())
            .chain(once(false))
            .collect::<Vec<_>>();
        let ground_truth = rand_set
            .iter()
            .fold(0, |acc, x| acc + if *x { 1 } else { 0 });
        let stream = rand_set.with_reset(4).clock_pos_edge(100);
        let uut: Counter<U6> = Counter::default();
        let out_stream = uut.run(stream)?;
        let output = out_stream.clone().last().map(|x| x.value.2);
        assert_eq!(output, Some(bits(ground_truth)));
        let tb = out_stream.collect::<SynchronousTestBench<_, _>>();
        let tm = tb.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = tb.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
