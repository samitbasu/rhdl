//! An asynchornous block RAM
//!
//! A simple block ram that stores 2^N values of type T.
//! It has two interfaces for read and writing, and supports
//! two different clocks.  This RAM is meant primarily for
//! FPGAs, as you can specify the initial contents of the
//! RAM.  For ASICs, you should probably assume the contents
//! of the RAM are random on initialization and implement
//! reset mechanism.
//!
//! This block ram is not "combinatorial" but is rather
//! "fully registered".  That means that the read address
//! is sampled on the positive edge of the read clock, and the
//! data is presented prior to the positive edge of the _next_ clock.
//!
//! There are block rams that don't have this limitation (i.e., they
//! provide the read output on the same clock as the read address).  But
//! they are generally not portable.  If you need one of those, you should
//! create a custom model for it.
//!
//! Here is the schematic symbol:
#![doc = badascii_doc::badascii_formal!(r"
      +----+AsyncRAM+-------------+     
 B<N> |                           | T   
+---->| read.addr          output +---->
 clk  |                           |     
+---->| read.clock      R domain  |     
      |               ^           |     
      |          +----+-----+     |     
 B<N> |               v           |     
+---->| write.addr      W domain  |     
  T   |                           |     
+---->| write.data                |     
 bool |                           |     
+---->| write.enable              |     
 clk  |                           |     
+---->| write.clock               |     
      |                           |     
      +---------------------------+         
")]
//!
//!# Timing
//!
//! From a timing perspective, the asynchronous RAM operates
//! with two clocks, a read clock and a write clock.  The output
//! is synchronous with the read clock, as indicated by the
//! type signature.  The write inputs are synchronous to their own
//! write clock.
//!
//! It is considered undefined behavior to read and write simultaneously
//! to the same address in the block RAM.  With multiple clocks, it
//! becomes even more nebulous to define what "simultaneous" means.
//! In general, you should avoid accessing the same address/row of the RAM
//! from both the read and write sides of the interface at the same
//! time.
//!
//! Here is a simple timing diagram:
#![doc = badascii_doc::badascii!(r"
                    +----+    +----+    +----+    +----+
read.clock     +----+    +----+    +----+    +----+     
                    :         :         :               
               +---+A1+--+---+A2+--+---+A1+--+-----     
read.addr      +---------+---------+---------+-----     
                                                        
                         +---+D1+--+---+D2+--+---+42+--+
output         +---------+---------+---------+---------+
                                                        
                   +---+   +---+   +---+   +---+        
write.clock    +---+   +---+   +---+   +---+   +---+    
                                                        
                       +-+42+--+                        
write.data     +-------+-------+-----------------------+
                                                        
                       +-+A1+--+                        
write.addr     +-------+-------+-----------------------+
                                                        
                       +-------+                        
write.enable   +-------+       +-----------------------+
")]
//!
//! In general, I don't recommend using an [AsyncBRAM].
//! It's easier and more idiomatic to use an [OptionAsyncBRAM]
//! or a [PipeAsyncBRAM] which provide [Option]-based interfaces.
//!
//!# Example
//!
//! Here is an example running the timing pattern shown
//! above.
//!```
#![doc = include_str!("../../../examples/async_bram.rs")]
//!```
//! With a simulation trace:
#![doc = include_str!("../../../doc/async_bram.md")]

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use quote::{format_ident, quote};
use rhdl::prelude::*;
use syn::parse_quote;

#[derive(PartialEq, Debug, Clone, Default)]
/// The [AsyncBRAM] core.  It stores elements of
/// type `T`.  The write side is on the `W` clock domain,
/// and the read side is on the `R` clock domain.
/// The `N` parameter indicates the number of address bits
/// which determines the size of the BRAM.
pub struct AsyncBRAM<T: Digital, W: Domain, R: Domain, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    initial: BTreeMap<Bits<N>, T>,
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> AsyncBRAM<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// Create a new [AsyncBRAM] with the given initialization values.
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        let len = (1 << N) as usize;
        Self {
            initial: initial.into_iter().take(len).collect(),
            _w: Default::default(),
            _r: Default::default(),
        }
    }
}

/// For the input interface, we have write and read parts.  
/// These are on different clock domains, so we need to split
/// them out.
/// The read input lines contain the current address and the
/// clock signal.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct ReadI<const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The address to read from
    pub addr: Bits<N>,
    /// The read clock
    pub clock: Clock,
}

/// The write input lines control the write side of the RAM.
/// It contains the address to write to, the data, and the
/// enable and clock signal.
#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct WriteI<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The address to write to
    pub addr: Bits<N>,
    /// The data to write to the BRAM
    pub data: T,
    /// The enable flag to enable a write operation
    pub enable: bool,
    /// The write clock
    pub clock: Clock,
}

#[derive(PartialEq, Debug, Digital, Copy, Timed, Clone)]
/// The inputs for the [AsyncBRAM] core
pub struct In<T: Digital, W: Domain, R: Domain, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    /// The write interface
    pub write: Signal<WriteI<T, N>, W>,
    /// The read interface
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitDQ for AsyncBRAM<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type D = ();
    type Q = ();
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for AsyncBRAM<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type I = In<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Clone)]
#[doc(hidden)]
pub struct S<T: Digital, const N: usize>
where
    rhdl::bits::W<N>: BitWidth,
{
    write_prev: WriteI<T, N>,
    contents: BTreeMap<Bits<N>, T>,
    read_clock: Clock,
    output_current: T,
    output_next: T,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> Circuit for AsyncBRAM<T, W, R, N>
where
    rhdl::bits::W<N>: BitWidth,
{
    type S = Rc<RefCell<S<T, N>>>;

    fn init(&self) -> Self::S {
        Rc::new(RefCell::new(S {
            write_prev: WriteI::dont_care(),
            contents: self.initial.clone(),
            read_clock: Clock::default(),
            output_current: T::dont_care(),
            output_next: T::dont_care(),
        }))
    }

    fn description(&self) -> String {
        format!(
            "Block RAM with {} entries of type {}",
            1 << N,
            std::any::type_name::<T>()
        )
    }

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        trace("input", &input);
        // Borrow the state mutably.
        let state = &mut state.borrow_mut();
        // We implement write-before-read semantics, but relying on this
        // is UB
        let write_if = input.write.val();
        if !write_if.clock.raw() {
            state.write_prev = write_if;
        }
        if write_if.clock.raw() && !state.write_prev.clock.raw() && state.write_prev.enable {
            let addr = state.write_prev.addr;
            let data = state.write_prev.data;
            state.contents.insert(addr, data);
        }
        // We need to handle the clock domain crossing stuff carefully
        // here.
        let read_if = input.read.val();
        // We sample the address whenever the read clock is low.
        // We also update the read out value of the BRAM whenever the
        // read clock is low.
        if !read_if.clock.raw() {
            state.output_next = state
                .contents
                .get(&read_if.addr)
                .copied()
                .unwrap_or_else(|| T::dont_care());
        }
        // On the positive edge of the read clock, we update the
        // current address and output values
        if read_if.clock.raw() && !state.read_clock.raw() {
            state.output_current = state.output_next;
        }
        state.read_clock = read_if.clock;
        trace("output", &state.output_current);
        signal(state.output_current)
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
            ntl: circuit_black_box(self, name)?,
        })
    }

    /*

    module uut(input wire [18:0] i, output reg [7:0] o);
        wire [3:0] read_addr;
        wire [0:0] read_clk;
        wire [3:0] write_addr;
        wire [7:0] write_data;
        wire [0:0] write_enable;
        wire [0:0] write_clk;
        reg [7:0] mem[15:0];
        initial begin
            mem[0] = 8'b00001111;
            mem[1] = 8'b00001110;
            mem[2] = 8'b00001101;
            mem[3] = 8'b00001100;
            mem[4] = 8'b00001011;
            mem[5] = 8'b00001010;
            mem[6] = 8'b00001001;
            mem[7] = 8'b00001000;
            mem[8] = 8'b00000111;
            mem[9] = 8'b00000110;
            mem[10] = 8'b00000101;
            mem[11] = 8'b00000100;
            mem[12] = 8'b00000011;
            mem[13] = 8'b00000010;
            mem[14] = 8'b00000001;
            mem[15] = 8'b00000000;
        end
        assign read_addr = i[17:14];
        assign read_clk = i[18];
        assign write_addr = i[3:0];
        assign write_data = i[11:4];
        assign write_enable = i[12];
        assign write_clk = i[13];
        always @(posedge read_clk) begin
            o <= mem[read_addr];
        end
        always @(posedge write_clk) begin
            if (write_enable)
            begin
                mem[write_addr] <= write_data;
            end
        end
    endmodule
         */

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let module_ident = format_ident!("{}", module_name);
        let input_bits: vlog::BitRange = (0..(<Self as CircuitIO>::I::bits())).into();
        let address_bits: vlog::BitRange = (0..N).into();
        let data_bits: vlog::BitRange = (0..T::BITS).into();
        let memory_size: vlog::BitRange = (0..(1 << N)).into();
        let initial_values = self.initial.iter().map(|(addr, val)| {
            let val: vlog::LitVerilog = val.typed_bits().into();
            let addr = syn::Index::from(addr.raw() as usize);
            quote! {
                mem[#addr] = #val;
            }
        });
        let i_kind = <<Self as CircuitIO>::I as Digital>::static_kind();
        let read_addr_range: vlog::BitRange = bit_range(i_kind, &path!(.read.val().addr))?.0.into();
        let read_clk_range: vlog::BitRange = bit_range(i_kind, &path!(.read.val().clock))?.0.into();
        let write_addr_range: vlog::BitRange =
            bit_range(i_kind, &path!(.write.val().addr))?.0.into();
        let write_data_range: vlog::BitRange =
            bit_range(i_kind, &path!(.write.val().data))?.0.into();
        let write_enable_range: vlog::BitRange =
            bit_range(i_kind, &path!(.write.val().enable))?.0.into();
        let write_clk_range: vlog::BitRange =
            bit_range(i_kind, &path!(.write.val().clock))?.0.into();
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident(input wire [#input_bits] i, output reg [#data_bits] o);
                wire [#address_bits] read_addr;
                wire [0:0] read_clk;
                wire [#address_bits] write_addr;
                wire [#data_bits] write_data;
                wire [0:0] write_enable;
                wire [0:0] write_clk;
                reg [#data_bits] mem[#memory_size];
                initial begin
                    #(#initial_values)*
                end
                assign read_addr = i[#read_addr_range];
                assign read_clk = i[#read_clk_range];
                assign write_addr = i[#write_addr_range];
                assign write_data = i[#write_data_range];
                assign write_enable = i[#write_enable_range];
                assign write_clk = i[#write_clk_range];
                always @(posedge read_clk) begin
                    o <= mem[read_addr];
                end
                always @(posedge write_clk) begin
                    if (write_enable)
                    begin
                        mem[write_addr] <= write_data;
                    end
                end
            endmodule
        };
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use expect_test::{expect, expect_file};
    use rhdl::prelude::vlog::Pretty;

    use super::*;

    fn get_scan_out_stream<const N: usize>(
        read_clock: u64,
        count: usize,
    ) -> impl Iterator<Item = TimedSample<ReadI<N>>> + Clone
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let scan_addr = (0..(1 << N)).map(bits::<N>).cycle().take(count);
        let stream_read = scan_addr.without_reset().clock_pos_edge(read_clock);
        stream_read.map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        })
    }

    fn get_write_stream<T: Digital, const N: usize>(
        write_clock: u64,
        write_data: impl Iterator<Item = Option<(Bits<N>, T)>> + Clone,
    ) -> impl Iterator<Item = TimedSample<WriteI<T, N>>> + Clone
    where
        rhdl::bits::W<N>: BitWidth,
    {
        let stream_write = write_data.without_reset().clock_pos_edge(write_clock);
        stream_write.map(|t| {
            t.map(|(cr, val)| WriteI {
                addr: val.map(|(a, _)| a).unwrap_or_else(|| bits(0)),
                data: val.map(|(_, d)| d).unwrap_or_else(|| T::dont_care()),
                enable: val.is_some(),
                clock: cr.clock,
            })
        })
    }

    #[test]
    fn test_ram_netlist() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let hdl = uut.netlist_hdl("uut")?;
        let expect = expect_file!["ram_fg.v.expect"];
        expect.assert_eq(&hdl.to_string());
        Ok(())
    }

    #[test]
    fn test_ram_as_verilog() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let expect = expect_test::expect![[r#"
            module uut(input wire [18:0] i, output reg [7:0] o);
               wire [3:0] read_addr;
               wire [0:0] read_clk;
               wire [3:0] write_addr;
               wire [7:0] write_data;
               wire [0:0] write_enable;
               wire [0:0] write_clk;
               reg [7:0] mem[15:0];
               initial begin
                  mem[0] = 8'b00001111;
                  mem[1] = 8'b00001110;
                  mem[2] = 8'b00001101;
                  mem[3] = 8'b00001100;
                  mem[4] = 8'b00001011;
                  mem[5] = 8'b00001010;
                  mem[6] = 8'b00001001;
                  mem[7] = 8'b00001000;
                  mem[8] = 8'b00000111;
                  mem[9] = 8'b00000110;
                  mem[10] = 8'b00000101;
                  mem[11] = 8'b00000100;
                  mem[12] = 8'b00000011;
                  mem[13] = 8'b00000010;
                  mem[14] = 8'b00000001;
                  mem[15] = 8'b00000000;
               end
               assign read_addr = i[17:14];
               assign read_clk = i[18:18];
               assign write_addr = i[3:0];
               assign write_data = i[11:4];
               assign write_enable = i[12:12];
               assign write_clk = i[13:13];
               always @(posedge read_clk) begin
                  o <= mem[read_addr];
               end
               always @(posedge write_clk) begin
                  if (write_enable) begin
                     mem[write_addr] <= write_data;
                  end
               end
            endmodule
        "#]];
        let module = uut.hdl("uut")?.as_module().pretty();
        expect.assert_eq(&module.to_string());
        let stream_read = get_scan_out_stream(100, 34);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat_n(None, 50));
        // Stitch the two streams together
        let stream = stream_read.merge(stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let test_bench = uut.run(stream).collect::<TestBench<_, _>>();
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_behavior() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits(0))),
        );
        let writes = vec![
            Some((bits(0), bits(142))),
            Some((bits(5), bits(89))),
            Some((bits(2), bits(100))),
            None,
            Some((bits(15), bits(23))),
        ];
        let stream_read = get_scan_out_stream(100, 32);
        let stream_write = get_write_stream(70, writes.into_iter().chain(std::iter::repeat(None)));
        let stream = stream_read.merge(stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let expected = vec![142, 0, 100, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23]
            .into_iter()
            .map(|x| signal(bits(x)));
        let vcd = uut.run(stream.clone()).collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("asynchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["ae24c8e9d3f0f61dc55d368a1777b2a5af60e7f6e770856d2cb6ef9bc8d39d8c"];
        let digest = vcd.dump_to_file(root.join("ram_write.vcd")).unwrap();
        expect.assert_eq(&digest);
        let output = uut
            .run(stream)
            .glitch_check(|x| (x.value.0.read.val().clock, x.value.1.val()))
            .sample_at_pos_edge(|x| x.value.0.read.val().clock)
            .skip(17)
            .map(|x| x.value.1);
        let expected = expected.collect::<Vec<_>>();
        let output = output.collect::<Vec<_>>();
        assert_eq!(expected, output);
        Ok(())
    }

    #[test]
    fn test_ram_read_only_behavior() -> miette::Result<()> {
        // Let's start with a simple test where the RAM is pre-initialized,
        // and we just want to read it.
        let uut = AsyncBRAM::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 32);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat_n(None, 50));
        // Stitch the two streams together
        let stream = merge(stream_read, stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let values = (0..16).map(|x| bits(15 - x)).cycle().take(32);
        let samples = uut
            .run(stream)
            .sample_at_pos_edge(|i| i.value.0.read.val().clock)
            .skip(1);
        let output = samples.map(|x| x.value.1.val());
        assert!(values.eq(output));
        Ok(())
    }
}
