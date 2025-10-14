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
pub struct Delay<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    dffs: [dff::DFF<T>; N],
}

impl<T: Digital + Default, const N: usize> Default for Delay<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            dffs: core::array::from_fn(|_| dff::DFF::new(T::default())),
        }
    }
}

impl<T: Digital, const N: usize> Delay<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Initialize the delay line with an initial design
    pub fn new_with_init(init: T) -> Self {
        Self {
            dffs: core::array::from_fn(|_| dff::DFF::new(init)),
        }
    }
}

impl<T: Digital, const N: usize> SynchronousIO for Delay<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = T;
    type O = T;
    type Kernel = delay<T, N>;
}

#[kernel]
/// Kernel for delay core
pub fn delay<T: Digital, const N: usize>(_cr: ClockReset, i: T, q: Q<T, N>) -> (T, D<T, N>)
where
    rhdl::bits::W<N>: BitWidth,
{
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

    fn test_pulse() -> impl Iterator<Item = TimedSample<(ClockReset, Option<Bits<8>>)>> + Clone {
        std::iter::once(Some(bits(42)))
            .chain(std::iter::repeat(None))
            .take(100)
            .with_reset(1)
            .clock_pos_edge(100)
    }

    #[test]
    fn test_delay_trace() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<8>>, 4>::default();
        let input = test_pulse();
        let vcd = uut.run(input).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("delay");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["0cb2038195e29a2bceb98b7b274ee0093f5ef9a948a04b210fcdc17ae16e0520"];
        let digest = vcd.dump_to_file(root.join("delay.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_delay_works() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<8>>, 4>::default();
        let input = test_pulse();
        let output = uut.run(input).synchronous_sample();
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
    fn test_vlog_generation() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<4>>, 2>::default();
        let hdl = uut.hdl("top")?.as_module().pretty();
        let expect = expect![[r#"
            module top(input wire [1:0] clock_reset, input wire [4:0] i, output wire [4:0] o);
               wire [14:0] od;
               wire [9:0] d;
               wire [9:0] q;
               assign o = od[4:0];
               top_dffs c0(.clock_reset(clock_reset), .i(d[9:0]), .o(q[9:0]));
               assign d = od[14:5];
               assign od = kernel_delay(clock_reset, i, q);
               function [14:0] kernel_delay(input reg [1:0] arg_0, input reg [4:0] arg_1, input reg [9:0] arg_2);
                     // d
                     reg [9:0] r0;
                     reg [4:0] r1;
                     reg [9:0] r2;
                     reg [4:0] r3;
                     // d
                     reg [9:0] r4;
                     reg [4:0] r5;
                     reg [14:0] r6;
                     reg [1:0] r7;
                     localparam l0 = 10'bXXXXXXXXXX;
                     begin
                        r7 = arg_0;
                        r1 = arg_1;
                        r2 = arg_2;
                        r0 = l0;
                        r0[4:0] = r1;
                        r3 = r2[4:0];
                        r4 = r0;
                        r4[9:5] = r3;
                        r5 = r2[9:5];
                        r6 = {r4, r5};
                        kernel_delay = r6;
                     end
               endfunction
            endmodule
            module top_dffs(input wire [1:0] clock_reset, input wire [9:0] i, output wire [9:0] o);
               top_dffs_0 c0(.clock_reset(clock_reset), .i(i[4:0]), .o(o[4:0]));
               top_dffs_1 c1(.clock_reset(clock_reset), .i(i[9:5]), .o(o[9:5]));
            endmodule
            module top_dffs_0(input wire [1:0] clock_reset, input wire [4:0] i, output reg [4:0] o);
               wire  clock;
               wire  reset;
               assign clock = clock_reset[0];
               assign reset = clock_reset[1];
               initial begin
                  o = 5'b00000;
               end
               always @(posedge clock) begin
                  if (reset) begin
                     o <= 5'b00000;
                  end else begin
                     o <= i;
                  end
               end
            endmodule
            module top_dffs_1(input wire [1:0] clock_reset, input wire [4:0] i, output reg [4:0] o);
               wire  clock;
               wire  reset;
               assign clock = clock_reset[0];
               assign reset = clock_reset[1];
               initial begin
                  o = 5'b00000;
               end
               always @(posedge clock) begin
                  if (reset) begin
                     o <= 5'b00000;
                  end else begin
                     o <= i;
                  end
               end
            endmodule
        "#]];
        expect.assert_eq(&hdl);
        Ok(())
    }

    #[test]
    fn test_delay_hdl_works() -> miette::Result<()> {
        let uut = Delay::<Option<Bits<8>>, 4>::default();
        let input = test_pulse();
        let test_bench = uut.run(input).collect::<SynchronousTestBench<_, _>>();
        let tm = test_bench.rtl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        let tm = test_bench.ntl(&uut, &Default::default())?;
        tm.run_iverilog()?;
        Ok(())
    }
}
