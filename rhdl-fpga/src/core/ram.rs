use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use rhdl::{
    core::hdl::ast::{index, memory_index, Declaration},
    prelude::*,
};

///
/// A simple block ram that stores 2^N values of type T.
/// It has two interfaces for read and writing, and supports
/// two different clocks.  This RAM is meant primarily for
/// FPGAs, as you can specify the initial contents of the
/// RAM.  For ASICs, you should probably assume the contents
/// of the RAM are random on initialization and implement
/// reset mechanism.
///
/// This block ram is not "combinatorial" but is rather
/// "fully registered".  That means that the read address
/// is sampled on the positive edge of the read clock, and the
/// data is presented on the positive edge of the _next_ clock.
///
/// There are block rams that don't have this limitation (i.e., they
/// provide the read output on the same clock as the read address).  But
/// they are generally not portable.  If you need one of those, you should
/// create a custom model for it.
#[derive(Debug, Clone)]
pub struct U<T: Digital, W: Domain, R: Domain, const N: usize> {
    initial: BTreeMap<Bits<N>, T>,
    _w: std::marker::PhantomData<W>,
    _r: std::marker::PhantomData<R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> U<T, W, R, N> {
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
#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct ReadI<const N: usize> {
    pub addr: Bits<N>,
    pub clock: Clock,
}

/// The write input lines control the write side of the RAM.
/// It contains the address to write to, the data, and the
/// enable and clock signal.
#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct WriteI<T: Digital, const N: usize> {
    pub addr: Bits<N>,
    pub data: T,
    pub enable: bool,
    pub clock: Clock,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital, Timed)]
pub struct I<T: Digital, W: Domain, R: Domain, const N: usize> {
    pub write: Signal<WriteI<T, N>, W>,
    pub read: Signal<ReadI<N>, R>,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitDQ for U<T, W, R, N> {
    type D = ();
    type Q = ();
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> CircuitIO for U<T, W, R, N> {
    type I = I<T, W, R, N>;
    type O = Signal<T, R>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

// TODO - maybe put this into a mutex?
#[derive(Debug, Clone, PartialEq)]
pub struct S<T: Digital, const N: usize> {
    write_prev: WriteI<T, N>,
    contents: BTreeMap<Bits<N>, T>,
    read_clock: Clock,
    output_current: T,
    output_next: T,
}

impl<T: Digital, W: Domain, R: Domain, const N: usize> Circuit for U<T, W, R, N> {
    type S = Rc<RefCell<S<T, N>>>;

    fn init(&self) -> Self::S {
        Rc::new(RefCell::new(S {
            write_prev: WriteI::init(),
            contents: self.initial.clone(),
            read_clock: Clock::default(),
            output_current: T::init(),
            output_next: T::init(),
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
        // Borrow the state mutably.
        let state = &mut state.borrow_mut();
        // We implement write-before-read semantics, but relying on this
        // is UB
        let write_if = input.write.val();
        if !write_if.clock.raw() {
            state.write_prev = write_if;
        }
        if write_if.clock.raw() && !state.write_prev.clock.raw() && write_if.enable {
            state.contents.insert(write_if.addr, write_if.data);
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
                .unwrap_or_else(|| T::init());
        }
        // On the positive edge of the read clock, we update the
        // current address and output values
        if read_if.clock.raw() && !state.read_clock.raw() {
            state.output_current = state.output_next;
        }
        state.read_clock = read_if.clock;
        signal(state.output_current)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(name)?;
        let (input, output) = flow_graph.circuit_black_box::<Self>(hdl);
        flow_graph.inputs = vec![input, vec![]];
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
        let wire_decl = |name: &str, width| Declaration {
            kind: HDLKind::Wire,
            name: name.into(),
            width: unsigned_width(width),
            alias: None,
        };
        module.declarations.extend([
            wire_decl("read_addr", N),
            wire_decl("read_clk", 1),
            wire_decl("write_addr", N),
            wire_decl("write_data", T::bits()),
            wire_decl("write_enable", 1),
            wire_decl("write_clk", 1),
            Declaration {
                kind: HDLKind::Reg,
                name: format!("mem[{}:0]", (1 << N) - 1),
                width: output_bits,
                alias: None,
            },
        ]);
        module.statements.push(initial(
            self.initial
                .iter()
                .map(|(a, d)| {
                    let d: BitString = d.typed_bits().into();
                    assign(&format!("mem[{}]", a.0), bit_string(&d))
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
    use rhdl::core::testbench::asynchronous::TestModuleOptions;

    use super::*;
    use std::iter::repeat;

    fn get_scan_out_stream<const N: usize>(
        read_clock: u64,
        count: usize,
    ) -> impl Iterator<Item = TimedSample<ReadI<N>>> {
        let scan_addr = (0..(1 << N)).map(bits::<N>).cycle().take(count);
        let stream_read = stream(scan_addr);
        let stream_read = clock_pos_edge(stream_read, read_clock);
        stream_read.map(|t| {
            t.map(|(cr, val)| ReadI {
                addr: val,
                clock: cr.clock,
            })
        })
    }

    fn get_write_stream<T: Digital, const N: usize>(
        write_clock: u64,
        write_data: impl Iterator<Item = Option<(Bits<N>, T)>>,
    ) -> impl Iterator<Item = TimedSample<WriteI<T, N>>> {
        let stream_write = stream(write_data);
        let stream_write = clock_pos_edge(stream_write, write_clock);
        stream_write.map(|t| {
            t.map(|(cr, val)| WriteI {
                addr: val.map(|(a, _)| a).unwrap_or_else(|| bits(0)),
                data: val.map(|(_, d)| d).unwrap_or_else(|| T::init()),
                enable: val.is_some(),
                clock: cr.clock,
            })
        })
    }

    #[test]
    fn test_ram_as_verilog() -> miette::Result<()> {
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 34);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, repeat(None).take(50));
        // Stitch the two streams together
        let stream = merge(stream_read, stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });

        //write_testbench(&uut, stream, "ram_tb_2.v")?;
        //        test_asynchronous_hdl(&uut, stream)?;
        let options = TestModuleOptions {
            skip_first_cases: 2,
            vcd_file: Some("ram.vcd".into()),
            hold_time: 1,
            ..Default::default()
        };
        let test_mod = build_rtl_testmodule(&uut, stream, options)?;
        std::fs::write("ram_tb.v", &test_mod.testbench).unwrap();
        test_mod.run_iverilog()?;

        Ok(())
    }

    #[test]
    fn test_ram_write_behavior() {
        let uut = U::<Bits<8>, Red, Green, 4>::new(
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
        let stream_write = get_write_stream(70, writes.into_iter());
        let stream = merge(stream_read, stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });
        let expected = repeat(None).take(16).chain(
            vec![142, 0, 100, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 0]
                .into_iter()
                .map(|x| Some(signal(bits(x)))),
        );

        type UC = U<Bits<8>, Red, Green, 4>;
        validate(
            &uut,
            stream,
            &mut [
                glitch_check::<UC>(|i| i.value.read.val().clock),
                value_check::<UC>(|i| i.value.read.val().clock, expected),
            ],
            Default::default(),
        )
    }

    #[test]
    fn test_ram_read_only_behavior() {
        // Let's start with a simple test where the RAM is pre-initialized,
        // and we just want to read it.
        let uut = U::<Bits<8>, Red, Green, 4>::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let stream_read = get_scan_out_stream(100, 32);
        // The write interface will be dormant
        let stream_write = get_write_stream(70, repeat(None).take(50));
        // Stitch the two streams together
        let stream = merge(stream_read, stream_write, |r, w| I {
            read: signal(r),
            write: signal(w),
        });
        type UC = U<Bits<8>, Red, Green, 4>;
        let values = (0..16).map(|x| Some(signal(bits(15 - x)))).cycle();
        validate(
            &uut,
            stream,
            &mut [
                glitch_check::<UC>(|i| i.value.read.val().clock),
                value_check::<UC>(|i| i.value.read.val().clock, values),
            ],
            Default::default(),
        )
    }
}
