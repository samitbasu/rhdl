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
+---->| read.clock                |     
      |                           |     
      |                           |     
 B<N> |                           |     
+---->| write.addr                |     
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

use rhdl::{
    core::hdl::ast::{index, memory_index, unsigned_wire_decl, Declaration},
    prelude::*,
};

#[derive(PartialEq, Debug, Clone, Default)]
pub struct AsyncBRAM<T: Digital, W: Domain, R: Domain, N: BitWidth> {
    initial: BTreeMap<Bits<N>, T>,
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

impl<T: Digital, W: Domain, R: Domain, N: BitWidth> AsyncBRAM<T, W, R, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        let len = (1 << N::BITS) as usize;
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
#[derive(PartialEq, Debug, Digital)]
pub struct ReadI<N: BitWidth> {
    pub addr: Bits<N>,
    pub clock: Clock,
}

/// The write input lines control the write side of the RAM.
/// It contains the address to write to, the data, and the
/// enable and clock signal.
#[derive(PartialEq, Debug, Digital)]
pub struct WriteI<T: Digital, N: BitWidth> {
    pub addr: Bits<N>,
    pub data: T,
    pub enable: bool,
    pub clock: Clock,
}

#[derive(PartialEq, Debug, Digital, Timed)]
pub struct In<T: Digital, W: Domain, R: Domain, N: BitWidth> {
    pub write: Signal<WriteI<T, N>, W>,
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital, W: Domain, R: Domain, N: BitWidth> CircuitDQ for AsyncBRAM<T, W, R, N> {
    type D = ();
    type Q = ();
}

impl<T: Digital, W: Domain, R: Domain, N: BitWidth> CircuitIO for AsyncBRAM<T, W, R, N> {
    type I = In<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

#[derive(PartialEq, Debug, Clone)]
pub struct S<T: Digital, N: BitWidth> {
    write_prev: WriteI<T, N>,
    contents: BTreeMap<Bits<N>, T>,
    read_clock: Clock,
    output_current: T,
    output_next: T,
}

impl<T: Digital, W: Domain, R: Domain, N: BitWidth> Circuit for AsyncBRAM<T, W, R, N> {
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
            1 << N::BITS,
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
        let output_bits = unsigned_width(T::bits());
        let input_bits = unsigned_width(<Self as CircuitIO>::I::bits());
        module.ports = vec![
            port("i", Direction::Input, HDLKind::Wire, input_bits),
            port("o", Direction::Output, HDLKind::Reg, output_bits),
        ];
        module.declarations.extend([
            unsigned_wire_decl("read_addr", N::BITS),
            unsigned_wire_decl("read_clk", 1),
            unsigned_wire_decl("write_addr", N::BITS),
            unsigned_wire_decl("write_data", T::bits()),
            unsigned_wire_decl("write_enable", 1),
            unsigned_wire_decl("write_clk", 1),
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
                .map(|(a, d)| {
                    let d: BitString = d.typed_bits().into();
                    assign(&format!("mem[{}]", a.raw()), bit_string(&d))
                })
                .collect(),
        ));
        let i_kind = <<Self as CircuitIO>::I as Timed>::static_kind();
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign(
                "read_addr",
                Path::default().field("read").signal_value().field("addr"),
            ),
            reassign(
                "read_clk",
                Path::default().field("read").signal_value().field("clock"),
            ),
            reassign(
                "write_addr",
                Path::default().field("write").signal_value().field("addr"),
            ),
            reassign(
                "write_data",
                Path::default().field("write").signal_value().field("data"),
            ),
            reassign(
                "write_enable",
                Path::default()
                    .field("write")
                    .signal_value()
                    .field("enable"),
            ),
            reassign(
                "write_clk",
                Path::default().field("write").signal_value().field("clock"),
            ),
        ]);
        module.statements.push(always(
            vec![Events::Posedge("read_clk".into())],
            vec![non_blocking_assignment(
                "o",
                memory_index("mem", id("read_addr")),
            )],
        ));
        module.statements.push(always(
            vec![Events::Posedge("write_clk".into())],
            vec![if_statement(
                id("write_enable"),
                vec![non_blocking_assignment("mem[write_addr]", id("write_data"))],
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
    use std::path::PathBuf;

    use expect_test::{expect, expect_file};

    use super::*;

    fn get_scan_out_stream<N: BitWidth>(
        read_clock: u64,
        count: usize,
    ) -> impl Iterator<Item = TimedSample<ReadI<N>>> + Clone {
        let scan_addr = (0..(1 << N::BITS)).map(bits::<N>).cycle().take(count);
        let stream_read = scan_addr.stream().clock_pos_edge(read_clock);
        stream_read.map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        })
    }

    fn get_write_stream<T: Digital, N: BitWidth>(
        write_clock: u64,
        write_data: impl Iterator<Item = Option<(Bits<N>, T)>> + Clone,
    ) -> impl Iterator<Item = TimedSample<WriteI<T, N>>> + Clone {
        let stream_write = write_data.stream().clock_pos_edge(write_clock);
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
    fn test_ram_flow_graph() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<U8>, Red, Green, U4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let fg = uut.flow_graph("uut")?;
        let hdl = fg.hdl("top")?;
        let expect = expect_file!["ram_fg.v.expect"];
        expect.assert_eq(&hdl.to_string());
        Ok(())
    }

    #[test]
    fn test_ram_as_verilog() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<U8>, Red, Green, U4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 34);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat(None).take(50));
        // Stitch the two streams together
        let stream = stream_read.merge(stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let test_bench = uut.run(stream)?.collect::<TestBench<_, _>>();
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(10))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_behavior() -> miette::Result<()> {
        let uut = AsyncBRAM::<Bits<U8>, Red, Green, U4>::new(
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
        let vcd = uut.run(stream.clone())?.collect::<Vcd>();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vcd")
            .join("ram")
            .join("asynchronous");
        std::fs::create_dir_all(&root).unwrap();
        let expect = expect!["f90130a11fa63b43b6595c5b5f33201d2caab089d7aebfc849779db2176eb190"];
        let digest = vcd.dump_to_file(&root.join("ram_write.vcd")).unwrap();
        expect.assert_eq(&digest);
        let output = uut
            .run(stream)?
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
        let uut = AsyncBRAM::<Bits<U8>, Red, Green, U4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 32);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, std::iter::repeat(None).take(50));
        // Stitch the two streams together
        let stream = merge(stream_read, stream_write, |r, w| In {
            read: signal(r),
            write: signal(w),
        });
        let values = (0..16).map(|x| bits(15 - x)).cycle().take(32);
        let samples = uut
            .run(stream)?
            .sample_at_pos_edge(|i| i.value.0.read.val().clock)
            .skip(1);
        let output = samples.map(|x| x.value.1.val());
        assert!(values.eq(output));
        Ok(())
    }
}
