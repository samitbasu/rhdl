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

use quote::{format_ident, quote};
use rhdl::prelude::*;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use syn::parse_quote;

/// The synchronous version of the block ram.  
///
/// This one assumes a clock
/// for both the read and write interfaces, and since the clock and reset
/// lines are implied with Synchronous circuits, they do not appear in the
/// interface.
#[derive(PartialEq, Debug, Clone)]
pub struct SyncBRAM<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    initial: BTreeMap<Bits<N>, T>,
}

impl<T: Digital, const N: usize> Default for SyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    fn default() -> Self {
        Self {
            initial: BTreeMap::default(),
        }
    }
}

impl<T: Digital, const N: usize> SyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Create a new [SyncBRAM] with the provided initial contents.
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        let len = (1 << N) as usize;
        Self {
            initial: initial.into_iter().take(len).collect(),
        }
    }
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// A collection of signals for a raw write interface
pub struct Write<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The address for the write operation
    pub addr: Bits<N>,
    /// The value to write in the write operation
    pub value: T,
    /// Set this to `true` to enable a write
    pub enable: bool,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
/// Core inputs
pub struct In<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The read address to provide to the [SyncBRAM]
    pub read_addr: Bits<N>,
    /// The write parameters as a [Write] struct.
    pub write: Write<T, N>,
}

impl<T: Digital, const N: usize> SynchronousDQ for SyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type D = ();
    type Q = ();
}

impl<T: Digital, const N: usize> SynchronousIO for SyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, N>;
    type O = T;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Clone)]
#[doc(hidden)]
pub struct S<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    clock: Clock,
    contents: BTreeMap<Bits<N>, T>,
    output_current: T,
    output_next: T,
    write_prev: Write<T, N>,
}

impl<T: Digital, const N: usize> Synchronous for SyncBRAM<T, N>
where
    rhdl::bits::W<N>: BitWidth,
{
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
            1 << N,
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
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: synchronous_black_box(self, name)?,
            circuit_type: CircuitType::Synchronous,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let module = format_ident!("{}", module_name);
        let input_bits: vlog::BitRange = (0..(<Self::I as Digital>::BITS)).into();
        let address_bits: vlog::BitRange = (0..N).into();
        let data_bits: vlog::BitRange = (0..(T::BITS)).into();
        let memory_size: vlog::BitRange = (0..(1 << N)).into();
        let initial_values = self.initial.iter().map(|(addr, val)| {
            let val: vlog::LitVerilog = val.typed_bits().into();
            let addr = syn::Index::from(addr.raw() as usize);
            quote! {mem[#addr] = #val;}
        });
        let i_kind = <Self::I as Digital>::static_kind();
        let read_addr_index: vlog::BitRange = bit_range(i_kind, &path!(.read_addr))?.0.into();
        let write_addr_index: vlog::BitRange = bit_range(i_kind, &path!(.write.addr))?.0.into();
        let write_value_index: vlog::BitRange = bit_range(i_kind, &path!(.write.value))?.0.into();
        let write_enable_index: vlog::BitRange = bit_range(i_kind, &path!(.write.enable))?.0.into();
        let clock_index: vlog::BitRange = bit_range(ClockReset::static_kind(), &path!(.clock))?
            .0
            .into();
        let module: vlog::ModuleDef = parse_quote! {
            module #module(
                input wire [1:0] clock_reset,
                input wire [#input_bits] i,
                output reg [#data_bits] o
            );
                wire [#address_bits] read_addr;
                wire [#address_bits] write_addr;
                wire [#data_bits] write_value;
                wire [0:0] write_enable;
                wire [0:0] clock;
                reg [#data_bits] mem[#memory_size];
                initial begin
                    #(#initial_values)*
                end
                assign read_addr = i[#read_addr_index];
                assign write_addr = i[#write_addr_index];
                assign write_value = i[#write_value_index];
                assign write_enable = i[#write_enable_index];
                assign clock = clock_reset[#clock_index];
                always @(posedge clock) begin
                    o <= mem[read_addr];
                end
                always @(posedge clock) begin
                    if (write_enable)
                    begin
                        mem[write_addr] <= write_value;
                    end
                end
            endmodule
        };
        Ok(HDLDescriptor {
            name: module_name,
            modules: module.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use rhdl::prelude::{vlog::Pretty, *};

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

    impl From<Cmd> for In<b8, 4> {
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
        type UC = SyncBRAM<b8, 4>;
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
        let sim = uut.run(stream);
        let vcd = sim.clone().collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("synchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["5def5c54395ba2862fc22ba74776d05afd9b013a1600fab6a7b0d78a6da9ba72"];
        let digest = vcd
            .dump_to_file(root.join("test_scan_out_ram.vcd"))
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
    ) -> impl Iterator<Item = TimedSample<(ClockReset, In<b8, 4>)>> {
        let inputs = (0..).map(|_| rand_cmd().into()).take(len);
        inputs.with_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        type UC = SyncBRAM<b8, 4>;
        let uut: UC = SyncBRAM::new((0..).map(|ndx| (bits(ndx), bits(0))));
        let expect = expect_test::expect![[r#"
            module top(input wire [1:0] clock_reset, input wire [16:0] i, output reg [7:0] o);
               wire [3:0] read_addr;
               wire [3:0] write_addr;
               wire [7:0] write_value;
               wire [0:0] write_enable;
               wire [0:0] clock;
               reg [7:0] mem[15:0];
               initial begin
                  mem[0] = 8'b00000000;
                  mem[1] = 8'b00000000;
                  mem[2] = 8'b00000000;
                  mem[3] = 8'b00000000;
                  mem[4] = 8'b00000000;
                  mem[5] = 8'b00000000;
                  mem[6] = 8'b00000000;
                  mem[7] = 8'b00000000;
                  mem[8] = 8'b00000000;
                  mem[9] = 8'b00000000;
                  mem[10] = 8'b00000000;
                  mem[11] = 8'b00000000;
                  mem[12] = 8'b00000000;
                  mem[13] = 8'b00000000;
                  mem[14] = 8'b00000000;
                  mem[15] = 8'b00000000;
               end
               assign read_addr = i[3:0];
               assign write_addr = i[7:4];
               assign write_value = i[15:8];
               assign write_enable = i[16:16];
               assign clock = clock_reset[0:0];
               always @(posedge clock) begin
                  o <= mem[read_addr];
               end
               always @(posedge clock) begin
                  if (write_enable) begin
                     mem[write_addr] <= write_value;
                  end
               end
            endmodule
        "#]];
        let hdl = uut.hdl("top")?.modules.pretty();
        expect.assert_eq(&hdl);
        let stream = random_command_stream(1000);
        let test_bench = uut.run(stream).collect::<SynchronousTestBench<_, _>>();
        let test_mod = test_bench.ntl(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_then_read() -> miette::Result<()> {
        type UC = SyncBRAM<b8, 4>;
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
        let sim = uut.run(inputs);
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
