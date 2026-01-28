//!# Single bit synchronizer
//!
//!# Purpose
//!
//! This core allows you to safely (relatively) cross clock domains
//! with a single bit signal.  The problem with crossing clock domains
//! is related to the potential violation of the invariants of flip
//! flops.  Consider the following super trivial example of an input
//! to a flip flop that transitions close to the clock edge, within
//! some time δt
#![doc = badascii_doc::badascii!("
          +----+    +----+    +----+  
clk   +---+    +----+    +----+    +-+
                  <+> δt              
                  +------------------+
 d   +------------+                   
                    ??????????+------+
 q   +-------------+??????????        
")]
//! The `??` states represent the fact that we cannot know for
//! sure what state the flip flop will be in during this time
//! interval.  And worse, any outputs connected to this FF will
//! also be in an indeterminate state.
//!
//! This phenomenon is known as `metastability`, and there are
//! plenty of resources on the web available to learn more.  The
//! "fix" is to note that for a real flop, it will (quickly) decide
//! one way or the other as to what it's going to take a `true` or
//! `false` value.  In that case, we can actually draw the diagram
//! something more like this.
#![doc = badascii_doc::badascii!("
           +----+    +----+    +----+     
clk   +----+    +----+    +----+    +----+
         <+> δt                           
         +------------------+             
 d1   +--+                  +------------+

 (resolves false) +  
                  |                     
           ??     v  +---------+          
 q1   +---+??+-------+         +---------+
 
 (resolves true)  + 
                  v      
           ??+-----------------+          
 q1   +---+??                  +---------+
                                          
    (arrives late)             +---------+
 q2   +------------------------+          
                                          
    (arrives early)  +-------------------+
 q2   +--------------+                    
")]
//! In the upper `q1` trace, the ff decided that the input was
//! `false`, and so went low for the cycle.  In the lower `q1`
//! trace, the ff decided that the input was `true` and so went
//! high for the cycle.  
//!
//! This should probably terrify you.  It basically means that you get
//! a random bit output because of the violation of the timing
//! for the flip flop.  The `fix` is to sample the output of the ff
//! with another flip flop, (`q2`), which will _at least_ resolve
//! to one of the two states at every clock edge.  The important
//! part here is _not_ that the signal made it out through the second
//! flop.  The important part is that there are no `??` sections in
//! the output `q2`.  So even though the bit may be lost, at least
//! the uncertainty stops propagating.  
//!
//! I've seen hardware designers put longer chains in place in the belief
//! that this somehow avoids the problem.  It does not.  The damage
//! was done at the first FF.  The subsequent ones are only to reduce the
//! probability of a `??` appearing at the output.  This is already
//! exceptionally unlikely as the amount of time the FF spends in the
//! `??` state is generally quite short with respect to the clock cycle
//! period.    However, a belt-and-suspenders approach suggests an
//! extra FF doesn't hurt.
//!
//! The root problem, however, remains.  The bit was lost in transition.
//! So what to do?  Simplest answer is **Do Not Use This**.  It's
//! best thought of as an `unsafe` primitive out of which safe cores
//! can be built.  You need to encode the signal crossing the clock
//! boundary so that effectively, this phenom doesn't happen.  If
//! you want to cross a counter (so that the output clock has the
//! count of pulses arriving at the input clock domain), then
//! look at [CrossCounter](super::cross_counter::CrossCounter).
//! If you want to send data between clock domains, use a FIFO,
//! such as [AsyncFIFO](super::super::fifo::asynchronous::AsyncFIFO).
//!
//! Finally, if you only want to cross data relatively slowly, use
//! a multi-bit handshake based method, like the [SlowCrosser].
//!
//! For more detail, see [this doc]
//! (http://cva.stanford.edu/people/davidbbs/classes/ee108a/winter0607%20labs/lect.9.Metastability-blackschaffer.ppt)
//! from Stanford University.  It has a lot of great detail on the phenomenon.
//!
//! And just in case you think I am exaggerating the danger here (which
//! is very analogous to a memory safety issue), here we end up
//! with a pulse that may or may not arrive in the new clock domain at all!
#![doc = badascii_doc::badascii!("
           +----+    +----+    +---+      
clk   +----+    +----+    +----+          
         <+> δt      :         :          
         +-------+                        
 d1   +--+       +-----------------+      
                                          
 (resolves false) +  :         :          
                  |  :         :     +-+  
           ??     v                    |  
 q1   +---+??+---------------------+   |  
                                       |  
 (resolves true)  +  :         :       |  
                  v  :         :     +-+-+
           ??+-------+                 | |
 q1   +---+??        +-------------+   | |
                     :         :       | |
    (never arrives)  :         :     <-+ |
 q2   +----------------------------+     |
                                         |
    (arrives ok)     +---------+     <---+
 q2   +--------------+         +---+      
")]
//!
//! Finally:  Note that `rhdl` does not simulate metastability.  Essentially
//! for DFFs in `rhdl`, the value of δt is `0`.  So be extra cautious when
//! assuming that your designs are safe if you are using synchronizers as
//! raw components.
//!
//!# Connections
//!
//! Here is the schematic symbol for the synchronizer.
//!
#![doc = badascii_doc::badascii_formal!("
     +----+Sync1Bit+---+    
     |                 |    
+--->| data     output +--->
     |                 |    
     |              cr |<---+
     |                 |    
     +-----------------+    
"
)]
//!
//!# Internals
//!
//! Internally, the structure is simple:
//!
#![doc = badascii_doc::badascii!("
      +-------+      +-------+     
      |       |      |       |     
+---->|d FF1 q+----->|d FF2 q+---->
      |       |      |       |     
   +->|clk/rst|  +-->|clk/rst|     
   |  |       |  |   |       |     
   |  +-------+  |   +-------+     
   |             |                 
+--+-------------+                 
"
)]
//!
//!# Example
//!
//! Here is a simple example of a 1 bit synchronizer being fed random pulses
//!```
#![doc = include_str!("../../examples/bit_sync.rs")]
//!```
//! With an output trace
#![doc = include_str!("../../doc/sync_cross.md")]

use quote::format_ident;
use rhdl::{
    core::{circuit::descriptor::AsyncKind, ScopedName},
    prelude::*,
};
use syn::parse_quote;

/// A simple two-register synchronizer for crossing
/// a single bit from the W domain to the R domain
#[derive(PartialEq, Debug, Clone, Default)]
pub struct Sync1Bit<W: Domain, R: Domain> {
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// Input to the synchronizer
pub struct In<W: Domain, R: Domain> {
    /// The data signal (comes from the input clock domain)
    pub data: Signal<bool, W>,
    /// The clock and reset signal from the output clock domain
    pub cr: Signal<ClockReset, R>,
}

impl<W: Domain, R: Domain> CircuitDQ for Sync1Bit<W, R> {
    type D = ();
    type Q = ();
}

impl<W: Domain, R: Domain> CircuitIO for Sync1Bit<W, R> {
    type I = In<W, R>;
    type O = Signal<bool, R>;
    type Kernel = NoCircuitKernel<Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
#[doc(hidden)]
pub struct S {
    clock: Clock,
    reg1_next: bool,
    reg1_current: bool,
    reg2_next: bool,
    reg2_current: bool,
}

impl<W: Domain, R: Domain> Circuit for Sync1Bit<W, R> {
    type S = S;

    fn init(&self) -> Self::S {
        S {
            clock: Clock::dont_care(),
            reg1_next: false,
            reg1_current: false,
            reg2_next: false,
            reg2_current: false,
        }
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        let clock = input.cr.val().clock;
        let reset = input.cr.val().reset;
        trace("clock", &clock);
        trace("reset", &reset);
        trace("input", &input.data);
        if !clock.raw() {
            state.reg1_next = input.data.val();
            state.reg2_next = state.reg1_current;
        }
        if clock.raw() && !state.clock.raw() {
            state.reg1_current = state.reg1_next;
            state.reg2_current = state.reg2_next;
        }
        if reset.raw() {
            state.reg1_next = false;
            state.reg2_next = false;
        }
        state.clock = clock;
        trace("output", &state.reg2_current);
        signal(state.reg2_current)
    }

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError> {
        let name = scoped_name.to_string();
        Descriptor::<AsyncKind> {
            name: scoped_name,
            input_kind: <<Self as CircuitIO>::I as Digital>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Digital>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Digital>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Digital>::static_kind(),
            kernel: None,
            netlist: None,
            hdl: Some(self.hdl(&name)?),
            _phantom: std::marker::PhantomData,
        }
        .with_netlist_black_box()
    }
}

impl<W: Domain, R: Domain> Sync1Bit<W, R> {
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let module_ident = format_ident!("{}", module_name);
        let i_kind = <<Self as CircuitIO>::I as Digital>::static_kind();
        let i = <Self as CircuitIO>::I::dont_care();
        let reset_index = bit_range(i_kind, &path!(i.cr.val().reset))?;
        let reset_index = syn::Index::from(reset_index.0.start);
        let clock_index = bit_range(i_kind, &path!(i.cr.val().clock))?;
        let clock_index = syn::Index::from(clock_index.0.start);
        let data_index = bit_range(i_kind, &path!(i.data))?;
        let data_index = syn::Index::from(data_index.0.start);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident(input wire [2:0] i, output wire [0:0] o);
                wire [0:0] data;
                wire [0:0] clock;
                wire [0:0] reset;
                reg  [0:0] reg1;
                reg  [0:0] reg2;
                assign data = i[#data_index];
                assign clock = i[#clock_index];
                assign reset = i[#reset_index];
                assign o = reg2;
                always @(posedge clock) begin
                    if (reset) begin
                        reg1 <= 1'b0;
                    end else begin
                        reg1 <= data;
                    end
                    if (reset) begin
                        reg2 <= 1'b0;
                    end else begin
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
    use expect_test::expect;
    use rand::{Rng, SeedableRng};
    use rhdl::prelude::vlog::Pretty;

    use super::*;

    fn sync_stream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
        // Assume the red stuff comes on the edges of a clock
        let red = (0..)
            .map(move |_| rng.random::<bool>())
            .take(100)
            .with_reset(1)
            .clock_pos_edge(100);
        let blue = std::iter::repeat(false).with_reset(1).clock_pos_edge(79);
        red.merge_map(blue, |r, g| In {
            data: signal(r.1),
            cr: signal(g.0),
        })
    }

    #[test]
    fn test_sync_stream_makes_sense() -> miette::Result<()> {
        let stream = sync_stream();
        let session = Session::default();
        let stream = stream.map(|x| session.untraced(x, ()));
        for (ndx, val) in stream
            .take(150)
            .edge_time(|p| p.input.cr.val().clock)
            .filter(|x| x.input.cr.val().clock.raw())
            .enumerate()
        {
            let pred = 39 + 78 * ndx;
            assert!(pred == val.time as usize);
        }
        let stream = sync_stream();
        let stream = stream.map(|x| session.untraced(x, ()));
        for (ndx, val) in stream
            .take(150)
            .edge_time(|p| p.input.cr.val().clock)
            .filter(|x| !x.input.cr.val().clock.raw())
            .enumerate()
        {
            let pred = 78 + 78 * ndx;
            assert!(pred == val.time as usize);
        }
        Ok(())
    }

    #[test]
    fn test_hdl_generation() -> miette::Result<()> {
        let uut = Sync1Bit::<Red, Blue>::default();
        let hdl = uut.hdl("top")?.modules.pretty();
        let expect = expect_test::expect![[r#"
            module top(input wire [2:0] i, output wire [0:0] o);
               wire [0:0] data;
               wire [0:0] clock;
               wire [0:0] reset;
               reg [0:0] reg1;
               reg [0:0] reg2;
               assign data = i[0];
               assign clock = i[1];
               assign reset = i[2];
               assign o = reg2;
               always @(posedge clock) begin
                  if (reset) begin
                     reg1 <= 1'b0;
                  end else begin
                     reg1 <= data;
                  end
                  if (reset) begin
                     reg2 <= 1'b0;
                  end else begin
                     reg2 <= reg1;
                  end
               end
            endmodule
        "#]];
        expect.assert_eq(&hdl);
        let stream = sync_stream();
        let test_bench = uut.run(stream).collect::<TestBench<_, _>>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("synchronizer");
        std::fs::create_dir_all(&root).unwrap();
        let test_mod = test_bench.rtl(
            &uut,
            &TestBenchOptions::default()
                .vcd(&root.join("hdl.vcd").to_string_lossy())
                .skip(!0),
        )?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_synchronizer_performance() -> miette::Result<()> {
        let uut = Sync1Bit::<Red, Blue>::default();
        // Assume the Blue stuff comes on the edges of a clock
        let input = sync_stream();
        let _ = uut
            .run(input)
            .glitch_check(|i| (i.input.cr.val().clock, i.output.val()))
            .last();
        Ok(())
    }

    #[test]
    fn test_synchronizer_function() -> miette::Result<()> {
        let uut = Sync1Bit::<Red, Blue>::default();
        let input = sync_stream();
        let vcd = uut.run(input).collect::<VcdFile>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("synchronizer");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["5560c2b70d377be2927fe3d326a260f2743604b81b747445faa6ebbaff1aab90"];
        let digest = vcd.dump_to_file(root.join("synchronizer.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
