//! Delay Line
//!
//! This module implements a delay line in which the
//! data type propagating through the delay is generic
//! of type `T`, and the length of the delay is compile
//! time configurable.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!("
    +--+Delay+---------+   
  T |                  | T 
+-->| input     output +-->
    |                  |   
    +------------------+   
")]
//!# Internals
//! Internally the delay is simply a chain of `N`
//! [DFF]s, in a linear chain.  Note that the flip flops
//! will reset to the default value for `T`, which is
//! why it is required for `T: Default`.
//!
#![doc = badascii_doc::badascii!("
       +----+   +----+       +----+       
       |DFF1|   |DFF2|       |DFFN|       
     T |    |   |    |  ...  |    | T     
  +--->|d  q+-->|d  q+->  +->|d  q+-->    
input  +----+   +----+       +----+ output
")]
//!
//!
//!# Example
//!
//! The delay is a fairly simple core, and
//! the example is pretty basic.  To make it slightly more
//! interesting, we demonstrate the case that the
//! data being carried is an enum.
//!
//!```
#![doc = include_str!("../../examples/delay.rs")]
//!```
//!
//! The resulting trace is show below.
#![doc = include_str!("../../doc/delay.md")]
//!
use rhdl::prelude::*;

use super::dff;

#[derive(PartialEq, Debug, Clone, Synchronous, SynchronousDQ)]
/// The Delay core
/// `T` is the type carried by the core
/// `N` is the length of the delay line
pub struct Delay<T: Digital, const N: usize> {
    dffs: [dff::DFF<T>; N],
}

impl<T: Digital + Default, const N: usize> Default for Delay<T, N> {
    fn default() -> Self {
        Self {
            dffs: core::array::from_fn(|_| dff::DFF::new(T::default())),
        }
    }
}

impl<T: Digital, const N: usize> Delay<T, N> {
    /// Initialize the delay line with an initial design
    pub fn new_with_init(init: T) -> Self {
        Self {
            dffs: core::array::from_fn(|_| dff::DFF::new(init)),
        }
    }
}

impl<T: Digital, const N: usize> SynchronousIO for Delay<T, N> {
    type I = T;
    type O = T;
    type Kernel = delay<T, N>;
}

#[kernel]
/// Kernel for delay core
pub fn delay<T: Digital, const N: usize>(_cr: ClockReset, i: T, q: Q<T, N>) -> (T, D<T, N>) {
    let mut d = D::<T, N>::dont_care();
    d.dffs[0] = i;
    for i in 1..N {
        d.dffs[i] = q.dffs[i - 1];
    }
    let o = q.dffs[N - 1];
    (o, d)
}

#[cfg(test)]
mod tests {
    // Check that a single value propagates through the delay line

    use expect_test::expect;

    use super::*;

    fn test_pulse() -> impl Iterator<Item = TimedSample<(ClockReset, Option<Bits<U8>>)>> + Clone {
        std::iter::once(Some(bits(42)))
            .chain(std::iter::repeat(None))
            .take(100)
            .with_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_delay_trace() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<U8>>, 4>::default();
        let input = test_pulse();
        let vcd = uut.run(input)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("delay");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["1effd605e24dde66b455cbaf15edfac857518732bbbc60bce54b3eeda19eee16"];
        let digest = vcd.dump_to_file(root.join("delay.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_delay_works() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<U8>>, 4>::default();
        let input = test_pulse();
        let output = uut.run(input)?.synchronous_sample();
        let count = output.clone().filter(|t| t.value.2.is_some()).count();
        assert!(count == 1);
        let start_delay = output
            .clone()
            .enumerate()
            .find_map(|(ndx, t)| t.value.1.map(|_| ndx))
            .unwrap();
        let end_delay = output
            .enumerate()
            .find_map(|(ndx, t)| t.value.2.map(|_| ndx))
            .unwrap();
        assert!(end_delay - start_delay == 4);
        Ok(())
    }

    #[test]
    fn test_delay_hdl_works() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<U8>>, 4>::default();
        let input = test_pulse();
        let test_bench = uut.run(input)?.collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.flow_graph(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
