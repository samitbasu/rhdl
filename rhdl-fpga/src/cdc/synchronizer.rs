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
//! For more detail, see [this doc]!  It has lots of great detail.
//! (http://cva.stanford.edu/people/davidbbs/classes/ee108a/winter0607%20labs/lect.9.Metastability-blackschaffer.ppt)
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

use rhdl::{
    core::hdl::ast::{index, unsigned_reg_decl, unsigned_wire_decl},
    prelude::*,
};

/// A simple two-register synchronizer for crossing
/// a single bit from the W domain to the R domain
#[derive(PartialEq, Debug, Clone, Default)]
pub struct Sync1Bit<W: Domain, R: Domain> {
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

#[derive(PartialEq, Debug, Digital, Timed)]
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
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Digital)]
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

    fn description(&self) -> String {
        format!("Synchronizer from {:?}->{:?}", W::color(), R::color())
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

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: <Self::I as Timed>::static_kind(),
            output_kind: <Self::O as Timed>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: rhdl::core::ntl::builder::circuit_black_box(self, name)?,
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
            port("i", Direction::Input, HDLKind::Wire, unsigned_width(3)),
            port("o", Direction::Output, HDLKind::Wire, unsigned_width(1)),
        ];
        module.declarations.extend([
            unsigned_wire_decl("data", 1),
            unsigned_wire_decl("clock", 1),
            unsigned_wire_decl("reset", 1),
            unsigned_reg_decl("reg1", 1),
            unsigned_reg_decl("reg2", 1),
        ]);
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign("data", Path::default().field("data").signal_value()),
            reassign(
                "clock",
                Path::default().field("cr").signal_value().field("clock"),
            ),
            reassign(
                "reset",
                Path::default().field("cr").signal_value().field("reset"),
            ),
            continuous_assignment("o", id("reg2")),
        ]);
        let init = false.typed_bits().into();
        let reg1 = if_statement(
            id("reset"),
            vec![non_blocking_assignment("reg1", bit_string(&init))],
            vec![non_blocking_assignment("reg1", id("data"))],
        );
        let reg2 = if_statement(
            id("reset"),
            vec![non_blocking_assignment("reg2", bit_string(&init))],
            vec![non_blocking_assignment("reg2", id("reg1"))],
        );
        let events = vec![Events::Posedge("clock".into())];
        module.statements.push(always(events, vec![reg1, reg2]));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rand::{Rng, SeedableRng};
    use rhdl::core::sim::vcd;

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
        red.merge(blue, |r, g| In {
            data: signal(r.1),
            cr: signal(g.0),
        })
    }

    #[test]
    fn test_sync_stream_makes_sense() -> miette::Result<()> {
        let stream = sync_stream();
        for (ndx, val) in stream
            .take(150)
            .edge_time(|p| p.value.cr.val().clock)
            .filter(|x| x.value.cr.val().clock.raw())
            .enumerate()
        {
            let pred = 39 + 78 * ndx;
            assert!(pred == val.time as usize);
        }
        let stream = sync_stream();
        for (ndx, val) in stream
            .take(150)
            .edge_time(|p| p.value.cr.val().clock)
            .filter(|x| !x.value.cr.val().clock.raw())
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
        let stream = sync_stream();
        let test_bench = uut.run(stream)?.collect::<TestBench<_, _>>();
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
            .run(input)?
            .glitch_check(|i| (i.value.0.cr.val().clock, i.value.1.val()))
            .last();
        Ok(())
    }

    #[test]
    fn test_synchronizer_function() -> miette::Result<()> {
        let uut = Sync1Bit::<Red, Blue>::default();
        let input = sync_stream();
        let vcd = uut.run(input)?.collect::<vcd::Vcd>();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("synchronizer");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["6447afec090e6d976ed27898d22bfef13361bff6b78b6dbc7db1ada3bcd29252"];
        let digest = vcd.dump_to_file(root.join("synchronizer.vcd")).unwrap();
        expect.assert_eq(&digest);
        Ok(())
    }
}
