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

use rhdl::{
    core::hdl::ast::{index, unsigned_reg_decl, unsigned_wire_decl},
    prelude::*,
};

#[derive(PartialEq, Debug, Clone, Default)]
/// The [ResetConditioner] circuit.  Here `W` is the
/// domain where the `reset` circuit originates, and
/// `R` is the domain where the reset is being sent to
/// (i.e., is synchronous with).
pub struct ResetConditioner<W: Domain, R: Domain> {
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
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

    fn description(&self) -> String {
        format!("Reset synchronizer from {:?}->{:?}", W::color(), R::color())
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

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(&format!("{name}_inner"))?;
        let (input, output) = flow_graph.circuit_black_box::<Self>(hdl);
        flow_graph.inputs = vec![input];
        flow_graph.output = output;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            flow_graph,
            input_kind: <Self::I as Timed>::static_kind(),
            output_kind: <Self::O as Timed>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        let i_kind = <Self::I as Timed>::static_kind();
        module.ports = vec![
            port("i", Direction::Input, HDLKind::Wire, unsigned_width(2)),
            port("o", Direction::Output, HDLKind::Wire, unsigned_width(1)),
        ];
        module.declarations.extend([
            unsigned_wire_decl("i_reset", 1),
            unsigned_wire_decl("clock", 1),
            unsigned_reg_decl("reg1", 1),
            unsigned_reg_decl("reg2", 1),
        ]);
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign("i_reset", Path::default().field("reset").signal_value()),
            reassign("clock", Path::default().field("clock").signal_value()),
            continuous_assignment("o", id("reg2")),
        ]);
        /*
           The always block should be:
           always @(posedge clk, posedge reset) begin
               if (reset) begin
                   reg1 <= 1'b1;
                   reg2 <= 1'b1;
               end else begin
                   reg1 <= 1'b0;
                   reg2 <= reg1;
               end
           end
        */
        let reset_val = true.typed_bits().into();
        let normal_val = false.typed_bits().into();
        let if_block = if_statement(
            id("i_reset"),
            vec![
                non_blocking_assignment("reg1", bit_string(&reset_val)),
                non_blocking_assignment("reg2", bit_string(&reset_val)),
            ],
            vec![
                non_blocking_assignment("reg1", bit_string(&normal_val)),
                non_blocking_assignment("reg2", id("reg1")),
            ],
        );
        let events = vec![
            Events::Posedge("clock".into()),
            Events::Posedge("i_reset".into()),
        ];
        module.statements.push(always(events, vec![if_block]));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
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
        let uut = ResetConditioner::<Red, Blue>::default();
        let input = sync_stream();
        let tb = uut.run(input)?.collect::<TestBench<_, _>>();
        let hdl = tb.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        hdl.run_iverilog()?;
        let fg = tb.flow_graph(&uut, &TestBenchOptions::default().skip(10))?;
        fg.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_reset_conditioner_function() -> miette::Result<()> {
        let uut = ResetConditioner::<Red, Blue>::default();
        let input = sync_stream();
        let output = uut.run(input)?.collect::<Vcd>();
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
