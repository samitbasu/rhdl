//!# Negating Conditioner Core
//!
//! It's fairly frequent that you need to couple an
//! asynchronous (active-low) reset into RHDL, which uses
//! a synchronous active-high reset.  This core combines
//! the [ResetNegation] core with the [ResetConditioner]
//! to provide a simple way to generate a synchronous
//! clock and [Reset] signal from an asynchronous
//! [ResetN] input.
//!
//! The schematic symbol is
//!
#![doc = badascii_doc::badascii_formal!(r"
              Negation               
        +----+Conditioner+---+       
        |                    |       
+------>| reset_n     reset  |------>
        |                    |       
        |             clock  |<-----+
        |                    |       
        +--------------------+       
")]
//!
//!# Example
//!
//! Here is a simple example of the core being
//! used.
//!
//!```
#![doc = include_str!("../../examples/reset_neg_cond.rs")]
//!```
//!
//!With a trace as below.
//!
#![doc = include_str!("../../doc/reset_neg_cond.md")]

use rhdl::prelude::*;

#[derive(Debug, Clone, Default, Circuit, CircuitDQ)]
/// The [NegatingConditioner] core. The reset
/// comes from some domain `W`, and is crossed (and
/// inverted) into domain `R`.
pub struct NegatingConditioner<W: Domain, R: Domain> {
    neg: super::negation::ResetNegation<W>,
    cond: super::conditioner::ResetConditioner<W, R>,
}

#[derive(PartialEq, Digital, Timed)]
/// Inputs for the [NegatingConditioner].
pub struct In<W: Domain, R: Domain> {
    /// The active-low reset signal
    pub reset_n: Signal<ResetN, W>,
    /// The clock to synchronize the signal to
    pub clock: Signal<Clock, R>,
}

impl<W: Domain, R: Domain> CircuitIO for NegatingConditioner<W, R> {
    type I = In<W, R>;
    type O = Signal<Reset, R>;
    type Kernel = negating_conditioner_kernel<W, R>;
}

#[kernel]
#[doc(hidden)]
pub fn negating_conditioner_kernel<W: Domain, R: Domain>(
    i: In<W, R>,
    q: Q<W, R>,
) -> (Signal<Reset, R>, D<W, R>) {
    let mut d = D::<W, R>::dont_care();
    d.neg = i.reset_n;
    d.cond.reset = q.neg;
    d.cond.clock = i.clock;
    let o = q.cond;
    (o, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use rand::{Rng, SeedableRng};

    fn istream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
        // Use a seeded RNG to get repeatable results
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
        let red = (0..)
            .map(move |_| rng.random::<u8>() < 200)
            .take(100)
            .without_reset()
            .clock_pos_edge(100);
        let blue = std::iter::repeat(()).without_reset().clock_pos_edge(79);
        red.merge(blue, |r, b| In {
            reset_n: signal(reset_n(r.1)),
            clock: signal(b.0.clock),
        })
    }

    #[test]
    fn test_stream_function() -> miette::Result<()> {
        let uut = NegatingConditioner::<Red, Blue>::default();
        let stream = istream();
        let vcd = uut.run(stream)?.collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("reset")
            .join("negating_conditioner");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["4a44c223925c8409bf648c99b6b98994f89760b12db7ffd114295f95d640eae6"];
        let digest = vcd
            .dump_to_file(root.join("negating_conditioner.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let expect = expect_test::expect![[r#"
            module top(input wire [1:0] i, output wire [0:0] o);
               wire [3:0] od;
               wire [2:0] d;
               wire [1:0] q;
               assign o = od[0:0];
               top_cond c0(.i(d[2:1]), .o(q[1:1]));
               ;
               top_neg c1(.i(d[0:0]), .o(q[0:0]));
               ;
               assign d = od[3:1];
               assign od = kernel_negating_conditioner_kernel(i, q);
               function [3:0] kernel_negating_conditioner_kernel(input reg [1:0] arg_0, input reg [1:0] arg_1);
                     reg [0:0] or0;
                     reg [1:0] or1;
                     // d
                     reg [2:0] or2;
                     reg [0:0] or3;
                     reg [1:0] or4;
                     // d
                     reg [2:0] or5;
                     reg [0:0] or6;
                     // d
                     reg [2:0] or7;
                     reg [0:0] or8;
                     reg [3:0] or9;
                     localparam ol0 = 3'bXXX;
                     begin
                        or1 = arg_0;
                        or4 = arg_1;
                        or0 = or1[0:0];
                        or2 = ol0;
                        or2[0:0] = or0;
                        or3 = or4[0:0];
                        or5 = or2;
                        or5[1:1] = or3;
                        or6 = or1[1:1];
                        or7 = or5;
                        or7[2:2] = or6;
                        or8 = or4[1:1];
                        or9 = {or7, or8};
                        kernel_negating_conditioner_kernel = or9;
                     end
               endfunction
            endmodule
            module top_cond(input wire [1:0] i, output wire [0:0] o);
               wire [0:0] i_reset;
               wire [0:0] clock;
               reg [0:0] reg1;
               reg [0:0] reg2;
               assign i_reset = i[0];
               assign clock = i[1];
               assign o = reg2;
               always @(posedge clock, posedge i_reset) begin
                  if (i_reset) begin
                     reg1 <= 1'b1;
                     reg2 <= 1'b1;
                  end else begin
                     reg1 <= 1'b0;
                     reg2 <= reg1;
                  end
               end
            endmodule
            module top_neg(input wire [0:0] i, output wire [0:0] o);
               assign o = ~i;
            endmodule
        "#]];
        let uut = NegatingConditioner::<Red, Blue>::default();
        let hdl = uut.hdl("top")?.as_module().pretty();
        expect.assert_eq(&hdl);
        let stream = istream();
        let tb = uut.run(stream)?.collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        hdl.run_iverilog()?;
        let fg = tb.ntl(&uut, &TestBenchOptions::default().skip(10))?;
        fg.run_iverilog()?;
        Ok(())
    }
}
