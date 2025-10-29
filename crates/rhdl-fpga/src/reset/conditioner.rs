//! Reset Conditioner
//!
//! Given a reset signal in domain W that is asynchronous to
//! the clock of domain R, generate a reset signal in domain R
//! that is synchronous to the clock of domain R.
//!
//! Here is the schematic symbol for the reset conditioner
#![doc = badascii_doc::badascii_formal!("
     ++ResetConditioner+-+     
     |                   |     
+--->| reset      reset  +---> 
     |                   |     
     |            clock  |<---+
     |                   |     
     +-------------------+     
")]
//!
//! Internal circuitry
//!
//! The internal circuitry of the reset conditioner is
//! show below.  The reset conditioner is essentially a
//! [Sync1Bit] one-bit synchronizer, with a constant
//! input (and inverted output).  The difference is that
//! the resets of both flip flops are tied to the input
//! reset.  
#![doc = badascii_doc::badascii!(r"
           +-------+     +-------+     +        
           |       |     |       |     +\       
     1 +-->|d     q+---->|d     q+---->| +○+--->
           |       |     |       |     +/       
           |   r   |     |   r   |     +        
           +-------+     +-------+              
               ^             ^                  
reset (in)     |             |                  
     +---------+-------------+                  
")]
//!
//!
//!
//! Timing
//!
//! The behavior of the [ResetConditioner] is roughly depicted
//! below.
#![doc = badascii_doc::badascii!("
             t0    t1      t2         t3         
                   +-------+                     
reset (in)   +-----+       +--------------------+
                   :        :         :          
                  +----+    +----+    +----+     
clock        +----+    +----+    +----+    +----+
                   :        :         :          
                   +------------------+          
reset (out)  +-----+                  +---------+
")]
//!
//! The behavior is as follows.
//!
//! - In steady state, the output is zero (due to the inverter) at `t0`
//! - When the reset signal is asserted, both flops immediately go into reset
//! and the output goes high (again, due to the inverter) at `t1`
//! - When the reset is released, the circuit behaves like a 1-bit
//! synchronizer that "sees" a transition from `0 -> 1` at that
//! instant at `t2`.
//! - Depending on the timing of that transition relative to the
//! clock cycle, it may take either 1 or 2 clocks for the transition
//! to propagate to the output.  This depends on the difference
//! between `t2` and the next positive edge.  In this case, we assume
//! it is missed (arrives too close to the edge)
//! - The *start* and *duration* of the reset pulse are thus not
//! deterministic.
//! - The _end_ of the reset pulse, however, is aligned to a clock
//! edge, which is (generally) the important characteristic for an
//! synchronous reset.  This is the time `t3`
//!
//!# Example
//!
//! On it's own, a [ResetConditioner] is not particularly useful.  
//! But here is an example of how it operates with resets generated
//! in a different time domain from the one they are being crossed to.
//!
//!```
#![doc = include_str!("../../examples/reset_conditioner.rs")]
//!```
//!
//! The resulting trace file is here.
//!
#![doc = include_str!("../../doc/reset_conditioner.md")]

use quote::format_ident;
use rhdl::{core::ScopedName, prelude::*};
use syn::parse_quote;

#[derive(PartialEq, Debug, Clone, Default)]
/// The [ResetConditioner] circuit.  Here `W` is the
/// domain where the `reset` circuit originates, and
/// `R` is the domain where the reset is being sent to
/// (i.e., is synchronous with).
pub struct ResetConditioner<W: Domain, R: Domain> {
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// The inputs to the [ResetConditioner].
pub struct In<W: Domain, R: Domain> {
    /// The raw reset signal that is asserted asynchronously
    pub reset: Signal<Reset, W>,
    /// The clock signal to synchronize the reset to
    pub clock: Signal<Clock, R>,
}

impl<W: Domain, R: Domain> CircuitDQ for ResetConditioner<W, R> {
    type D = ();
    type Q = ();
}

impl<W: Domain, R: Domain> CircuitIO for ResetConditioner<W, R> {
    type I = In<W, R>;
    type O = Signal<Reset, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(Debug, Clone, PartialEq)]
#[doc(hidden)]
pub struct S {
    clock: Clock,
    prev_reset: Reset,
    reg1_next: bool,
    reg1_current: bool,
    reg2_next: bool,
    reg2_current: bool,
}

impl<W: Domain, R: Domain> Circuit for ResetConditioner<W, R> {
    type S = S;

    fn init(&self) -> Self::S {
        S {
            prev_reset: Reset::dont_care(),
            clock: Clock::dont_care(),
            reg1_next: false,
            reg1_current: false,
            reg2_next: false,
            reg2_current: false,
        }
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        const NO_RESET: bool = false;
        const RESET: bool = true;
        let clock = input.clock.val();
        let i_reset = input.reset.val();
        trace("clock", &clock);
        trace("reset", &i_reset);
        // if the clock is low, then chain the flops
        if !clock.raw() {
            state.reg1_next = NO_RESET;
            state.reg2_next = state.reg1_current;
        }
        // If there was an edge, then transition the values
        if (clock.raw() && !state.clock.raw()) || (i_reset.raw() && !state.prev_reset.raw()) {
            if !i_reset.raw() {
                state.reg1_current = state.reg1_next;
                state.reg2_current = state.reg2_next;
            } else {
                state.reg1_current = RESET;
                state.reg2_current = RESET;
                state.reg1_next = RESET;
                state.reg2_next = RESET;
            }
        }
        state.clock = clock;
        state.prev_reset = i_reset;
        trace("output", &state.reg2_current);
        signal(reset(state.reg2_current))
    }

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor, RHDLError> {
        let name = scoped_name.to_string();
        let mut descriptor = Descriptor {
            name: scoped_name,
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: <Self::D as Digital>::static_kind(),
            q_kind: <Self::Q as Digital>::static_kind(),
            kernel: None,
            hdl: Some(self.hdl(&name)?),
            circuit_type: CircuitType::Asynchronous,
            netlist: None,
        };
        descriptor.netlist = Some(circuit_black_box(&descriptor)?);
        Ok(descriptor)
    }
}

impl<W: Domain, R: Domain> ResetConditioner<W, R> {
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let module_name = format_ident!("{}", module_name);
        let reset_index = bit_range(
            <<Self as CircuitIO>::I as Digital>::static_kind(),
            &path!(.reset.val()),
        )?;
        let reset_index = syn::Index::from(reset_index.0.start);
        let clock_index = bit_range(
            <<Self as CircuitIO>::I as Digital>::static_kind(),
            &path!(.clock.val()),
        )?;
        let clock_index = syn::Index::from(clock_index.0.start);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_name(input wire [1:0] i, output wire [0:0] o);
                wire [0:0] i_reset;
                wire [0:0] clock;
                reg [0:0] reg1;
                reg [0:0] reg2;
                assign i_reset = i[#reset_index];
                assign clock = i[#clock_index];
                assign o = reg2;
                always @(posedge clock, posedge i_reset) begin
                    if (i_reset)
                    begin
                        reg1 <= 1'b1;
                        reg2 <= 1'b1;
                    end else begin
                        reg1 <= 1'b0;
                        reg2 <= reg1;
                    end
                end
            endmodule
        };
        Ok(HDLDescriptor {
            name: name.into(),
            modules: module.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use rand::{Rng, SeedableRng};

    fn sync_stream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
        // Assume the red stuff comes on the edges of a clock
        let red = (0..)
            .map(move |_| rng.random::<u8>() > 200)
            .take(100)
            .with_reset(1)
            .clock_pos_edge(100);
        let blue = std::iter::repeat(false).with_reset(1).clock_pos_edge(79);
        red.merge(blue, |r, g| In {
            reset: signal(reset(r.1)),
            clock: signal(g.0.clock),
        })
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let expect = expect_test::expect![[r#"
            module top(input wire [1:0] i, output wire [0:0] o);
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
        "#]];
        let uut = ResetConditioner::<Red, Blue>::default();
        let hdl = uut.hdl("top")?.modules.pretty();
        expect.assert_eq(&hdl);
        let input = sync_stream();
        let tb = uut.run(input).collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        hdl.run_iverilog()?;
        let fg = tb.ntl(&uut, &TestBenchOptions::default().skip(10))?;
        fg.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_reset_conditioner_function() -> miette::Result<()> {
        let uut = ResetConditioner::<Red, Blue>::default();
        let input = sync_stream();
        let output = uut.run(input).collect::<Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("reset")
            .join("conditioner");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["916eaf247cb94b037c4eef3c96cea34d53d7ff20998c38f794aaf898e7c7e16d"];
        let digest = output
            .dump_to_file(root.join("reset_conditioner.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
