//! A simple synchronous block ram.  
//!
//! The contents are generic over
//! a type T, and the address is assumed to be N bits wide.
//!
//! Here is the schematic symbol
#![doc = badascii_doc::badascii_formal!(r#"
      +--+SyncBRAM+---------+    
B<N>  |                     | T  
+---->|read_addr        out +--->
B<N>  |                     |    
+---->|write.addr           |    
 T    |                     |    
+---->|write.value          |    
bool  |                     |    
+---->|write.enable         |    
      |                     |    
      +---------------------+    
"#)]
//!# Timing
//! From a timing perspective, it is assumed that the block ram
//! implements a single cycle delay on both the read and write
//! interfaces.  Reading and writing to the same address cell at
//! the same clock cycle does not result in defined behavior.  The
//! underlying primitive may provide some guarantees (like write before read),
//! but these are not enforced, so use with caution.
//!
//! The following diagram illustrates the basic timing.  The `Data@A1`
//! indicates the nominal contents of the BRAM cell at address `A1`:
#![doc = badascii_doc::badascii!(r#"
+------+Timing+----------------------------------------------------+
|                                                                  |
|             +----+    +----+    +----+    +----+    +            |
|      clk  +-+    +----+    +----+    +----+    +----+            |
|             :         :         :         :         :            |
| read_addr   XXXX +---+A1+--+---+A2+--+---+A1+--+  ...            |
|           +------+---------+---------+---------+-----+           |
|             :         :         :         :         :            |
|       out           XXXX   +---+D1+--+---+D2+--+---+D3+--+  ...  |
|           +----------------+---------+---------+---------+-----+ |
|             :         :         :         :         :            |
| write.addr                 +---+A1+--+                           |
|           +----------------+---------+--------------------+      |
|             :         :         :         :         :            |
| write.value                +---+D3+--+                           |
|             +--------------+---------+--------------------+      |
|             :         :         :         :         :            |
| write.enable               +---------+                           |
|              ++------------+    : δt +--------------------+      |
|             :         :         +-->      :         :            |
| Data@A1      +-----+D1+-------------+-+D3+---------------+       |
|              +----------------------+--------------------+       |
|                                                                  |
+------------------------------------------------------------------+
"#)]
//!
//! In general, I don't recommend using a [SyncBRAM].  It's easier
//! and more idiomatic to use either [OptionSyncBRAM](super::option_sync::OptionSyncBRAM)
//! or [PipeSyncBRAM](super::pipe_sync::PipeSyncBRAM),
//! which provide [Option] interfaces.
//!
//!# Example
//!
//! Here we encode the example above.  
//! ```
#![doc = include_str!("../../../examples/sync_bram.rs")]
//! ```
//! The trace below demonstrates the result.
#![doc = include_str!("../../../doc/sync_bram.md")]

use rhdl::{
    core::hdl::ast::{index, index_bit, memory_index, Declaration},
    prelude::*,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// The synchronous version of the block ram.  
///
/// This one assumes a clock
/// for both the read and write interfaces, and since the clock and reset
/// lines are implied with Synchronous circuits, they do not appear in the
/// interface.
#[derive(PartialEq, Debug, Clone, Default)]
pub struct SyncBRAM<T: Digital, N: BitWidth> {
    initial: BTreeMap<Bits<N>, T>,
}

impl<T: Digital, N: BitWidth> SyncBRAM<T, N> {
    /// Create a new [SyncBRAM] with the provided initial contents.
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        let len = (1 << N::BITS) as usize;
        Self {
            initial: initial.into_iter().take(len).collect(),
        }
    }
}

#[derive(PartialEq, Debug, Digital)]
/// A collection of signals for a raw write interface
pub struct Write<T: Digital, N: BitWidth> {
    /// The address for the write operation
    pub addr: Bits<N>,
    /// The value to write in the write operation
    pub value: T,
    /// Set this to `true` to enable a write
    pub enable: bool,
}

#[derive(PartialEq, Debug, Digital)]
/// Core inputs
pub struct In<T: Digital, N: BitWidth> {
    /// The read address to provide to the [SyncBRAM]
    pub read_addr: Bits<N>,
    /// The write parameters as a [Write] struct.
    pub write: Write<T, N>,
}

impl<T: Digital, N: BitWidth> SynchronousDQ for SyncBRAM<T, N> {
    type D = ();
    type Q = ();
}

impl<T: Digital, N: BitWidth> SynchronousIO for SyncBRAM<T, N> {
    type I = In<T, N>;
    type O = T;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Clone)]
#[doc(hidden)]
pub struct S<T: Digital, N: BitWidth> {
    clock: Clock,
    contents: BTreeMap<Bits<N>, T>,
    output_current: T,
    output_next: T,
    write_prev: Write<T, N>,
}

impl<T: Digital, N: BitWidth> Synchronous for SyncBRAM<T, N> {
    type S = Rc<RefCell<S<T, N>>>;

    fn init(&self) -> Self::S {
        Rc::new(RefCell::new(S {
            clock: Clock::default(),
            contents: self.initial.clone(),
            output_current: T::dont_care(),
            output_next: T::dont_care(),
            write_prev: Write::dont_care(),
        }))
    }

    fn description(&self) -> String {
        format!(
            "Synchronous RAM with {} entries of type {}",
            1 << N::BITS,
            std::any::type_name::<T>()
        )
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("synchronous_ram");
        trace("input", &input);
        let state = &mut state.borrow_mut();
        let clock = clock_reset.clock;
        if !clock.raw() {
            state.output_next = state
                .contents
                .get(&input.read_addr)
                .copied()
                .unwrap_or(T::dont_care());
            state.write_prev = input.write;
        }
        if clock.raw() && !state.clock.raw() {
            if state.write_prev.enable {
                let addr = state.write_prev.addr;
                let data = state.write_prev.value;
                state.contents.insert(addr, data);
            }
            state.output_current = state.output_next;
        }
        state.clock = clock;
        trace("output", &state.output_current);
        trace_pop_path();
        state.output_current
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(&format!("{name}_inner"))?;
        let (clock_reset, input, output) = flow_graph.synchronous_black_box::<Self>(hdl);
        flow_graph.inputs = vec![clock_reset, input];
        flow_graph.output = output;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            flow_graph,
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
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
        let output_bits = unsigned_width(T::BITS);
        let input_bits = unsigned_width(<Self::I as Digital>::BITS);
        module.ports = vec![
            port(
                "clock_reset",
                Direction::Input,
                HDLKind::Wire,
                unsigned_width(2),
            ),
            port("i", Direction::Input, HDLKind::Wire, input_bits),
            port("o", Direction::Output, HDLKind::Reg, output_bits),
        ];
        let wire_decl = |name: &str, width| Declaration {
            kind: HDLKind::Wire,
            name: name.into(),
            width: unsigned_width(width),
            alias: None,
        };
        module.declarations.extend([
            wire_decl("read_addr", N::BITS),
            wire_decl("write_addr", N::BITS),
            wire_decl("write_value", T::BITS),
            wire_decl("write_enable", 1),
            wire_decl("clock", 1),
            Declaration {
                kind: HDLKind::Reg,
                name: format!("mem[{}:0]", (1 << N::BITS) - 1),
                width: output_bits,
                alias: None,
            },
        ]);
        module.statements.push(initial(
            self.initial
                .iter()
                .map(|(addr, val)| {
                    let val: BitString = val.typed_bits().into();
                    assign(&format!("mem[{}]", addr.raw()), bit_string(&val))
                })
                .collect(),
        ));
        let i_kind = <Self::I as Digital>::static_kind();
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign("read_addr", Path::default().field("read_addr")),
            reassign("write_addr", Path::default().field("write").field("addr")),
            reassign("write_value", Path::default().field("write").field("value")),
            reassign(
                "write_enable",
                Path::default().field("write").field("enable"),
            ),
            continuous_assignment("clock", index_bit("clock_reset", 0)),
        ]);
        module.statements.push(always(
            vec![Events::Posedge("clock".into())],
            vec![non_blocking_assignment(
                "o",
                memory_index("mem", id("read_addr")),
            )],
        ));
        module.statements.push(always(
            vec![Events::Posedge("clock".into())],
            vec![if_statement(
                id("write_enable"),
                vec![non_blocking_assignment(
                    "mem[write_addr]",
                    id("write_value"),
                )],
                vec![],
            )],
        ));
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rhdl::prelude::*;

    use super::*;
    use std::path::PathBuf;

    #[derive(PartialEq, Debug, Clone, Copy)]
    enum Cmd {
        Write(b4, b8),
        Read(b4),
    }

    fn rand_cmd() -> Cmd {
        match rand::random::<u8>() % 2 {
            0 => Cmd::Write(
                bits(rand::random::<u128>() % 16),
                bits(rand::random::<u128>() % 256),
            ),
            1 => Cmd::Read(bits(rand::random::<u128>() % 16)),
            _ => unreachable!(),
        }
    }

    struct TestItem(Cmd, b8);

    impl From<Cmd> for In<b8, U4> {
        fn from(cmd: Cmd) -> Self {
            match cmd {
                Cmd::Write(addr, value) => In {
                    read_addr: bits(0),
                    write: Write {
                        addr,
                        value,
                        enable: true,
                    },
                },
                Cmd::Read(addr) => In {
                    read_addr: addr,
                    write: Write::dont_care(),
                },
            }
        }
    }

    #[test]
    fn test_scan_out_ram() -> miette::Result<()> {
        type UC = SyncBRAM<b8, U4>;
        let uut: UC = SyncBRAM::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let test = (0..16)
            .cycle()
            .map(|ndx| TestItem(Cmd::Read(bits(ndx)), bits(15 - ndx)))
            .take(17);
        let inputs = test.clone().map(|item| item.0.into());
        let expected = test.map(|item| item.1).take(16);
        let stream = inputs.with_reset(1).clock_pos_edge(100);
        let sim = uut.run(stream)?;
        let vcd = sim.clone().collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("synchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["1308d1e201408d4630039df66282029c8ca0c49d914fd0baa60f1dbe4f0e135a"];
        let digest = vcd
            .dump_to_file(&root.join("test_scan_out_ram.vcd"))
            .unwrap();
        expect.assert_eq(&digest);
        let values = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .synchronous_sample()
            .skip(2)
            .map(|x| x.value.2);
        assert!(values.eq(expected));
        Ok(())
    }

    fn random_command_stream(
        len: usize,
    ) -> impl Iterator<Item = TimedSample<(ClockReset, In<b8, U4>)>> {
        let inputs = (0..).map(|_| rand_cmd().into()).take(len);
        inputs.with_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        type UC = SyncBRAM<b8, U4>;
        let uut: UC = SyncBRAM::new((0..).map(|ndx| (bits(ndx), bits(0))));
        let stream = random_command_stream(1000);
        let test_bench = uut.run(stream)?.collect::<SynchronousTestBench<_, _>>();
        let test_mod = test_bench.flow_graph(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_then_read() -> miette::Result<()> {
        type UC = SyncBRAM<b8, U4>;
        let uut: UC = SyncBRAM::new(std::iter::repeat_n((bits(0), b8::from(0)), 16));
        let test = vec![
            Cmd::Write(bits(0), bits(72)),
            Cmd::Write(bits(1), bits(99)),
            Cmd::Write(bits(2), bits(255)),
            Cmd::Read(bits(0)),
            Cmd::Read(bits(1)),
            Cmd::Read(bits(2)),
            Cmd::Read(bits(3)),
        ];
        let inputs = test
            .into_iter()
            .map(|x| x.into())
            .with_reset(1)
            .clock_pos_edge(100);
        let sim = uut.run(inputs)?;
        let outputs = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .synchronous_sample()
            .skip(5)
            .take(3)
            .map(|x| x.value.2)
            .collect::<Vec<_>>();
        assert_eq!(outputs, vec![b8::from(72), b8::from(99), b8::from(255)]);
        Ok(())
    }
}
