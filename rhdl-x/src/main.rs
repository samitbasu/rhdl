use rhdl::core::circuit::chain::Chain;
use rhdl::core::flow_graph::passes::check_for_unconnected_clock_reset::CheckForUnconnectedClockReset;
use rhdl::core::flow_graph::passes::pass::Pass;
use rhdl::core::sim::verilog_testbench::write_testbench_module;
use rhdl::core::types::timed;
use rhdl::prelude::*;
use rhdl_fpga::core::{constant, dff};
use std::io::Write;
use std::iter::repeat;
use std::{io, iter};
mod trizsnd;
use trizsnd::Cmd;
mod bit4;
mod bitvector;
use anyhow::ensure;
/* use rhdl_core::as_verilog_literal;
use rhdl_core::codegen::verilog::as_verilog_decl;
use rhdl_core::prelude::*;
use rhdl_core::root_descriptor;
use rhdl_core::types::domain::Red;
use rhdl_macro::Digital;
use rhdl_macro::Timed;
 */
//use translator::Translator;
//use verilog::VerilogTranslator;

//mod backend;
//mod circuit;
//mod clock;
mod auto_counter;
mod counter;
mod doubler;
mod strobe;
mod zdriver;
//mod descriptions;
pub mod inverter;
pub mod logic_loop;
pub mod single_bit;
pub mod state_cycler;
//mod push_pull;
//mod strobe;
//mod tristate;
//mod traitx;
//mod translator;
//mod verilog;
//mod dfg;
//mod trace;
//mod case;
//mod check;
//mod signal;
//mod timeset;
//mod visit;
mod async_counter;
mod busz;
mod trizrcv;
//mod demo;

//#[cfg(test)]
//mod tests;

// Let's start with the DFF.  For now, we will assume a reset is present.

// Template:
/*
    module top;
    reg clk;
    reg enable;
    wire out;

    initial begin
       enable = 1;
       clk = 1;
       forever #10 clk = ~clk;
    end

    Strobe_748a98de03e4aa30 dut(.i({enable, clk}), .o(out) );

    initial begin
    $dumpfile("strobe_v.vcd");
    $dumpvars(0);
    #1000;
    $finish;
    end

  endmodule

*/

#[test]
fn test_static_kind_function() {
    #[derive(Copy, Clone, PartialEq)]
    struct Foo {}

    impl Digital for Foo {
        const BITS: usize = 0;

        fn static_kind() -> Kind {
            use std::sync::LazyLock;
            static KIND: LazyLock<Kind> = LazyLock::new(|| Kind::make_struct("Foo", vec![]));
            KIND.clone()
        }

        fn static_trace_type() -> rtt::TraceType {
            todo!()
        }

        fn bin(self) -> Vec<bool> {
            vec![]
        }

        fn init() -> Self {
            Self {}
        }
    }

    fn sk() -> &'static Kind {
        use std::sync::LazyLock;
        static KIND: LazyLock<Kind> = LazyLock::new(|| Kind::make_struct("Foo", vec![]));
        &KIND
    }

    let k = Foo::static_kind();
    let l = sk();
    assert_eq!(k, *l);
}

#[test]
fn test_struct_vcd() {
    use rhdl::prelude::*;

    #[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
    pub enum State {
        #[default]
        Idle,
        Write,
        Read,
        Done,
    }

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    pub struct I {
        pub enable: bool,
        pub data: [b4; 3],
        pub tuple: (b3, b3),
        pub state: State,
        pub inum: Option<b6>,
    }

    #[derive(Copy, Clone, PartialEq, Debug, Digital)]
    pub struct O {
        pub enable: bool,
        pub data: [b4; 3],
        pub tuple: (b3, b3),
        pub state: State,
        pub inum: Option<b6>,
    }

    #[kernel]
    pub fn pass_through(cr: ClockReset, i: I) -> O {
        O {
            enable: !i.enable,
            data: [i.data[2], i.data[1], i.data[0]],
            tuple: (i.tuple.1, i.tuple.0),
            state: match i.state {
                State::Idle => State::Write,
                State::Write => State::Read,
                State::Read => State::Done,
                State::Done => State::Idle,
            },
            inum: match i.inum {
                Some(x) => Some(x + 1),
                None => None,
            },
        }
    }

    let circuit = Func::new::<pass_through>().unwrap();
    let inputs = (0..100).map(|x| I {
        enable: x % 2 == 0,
        data: [bits(x % 16), bits((x + 1) % 16), bits((x + 2) % 16)],
        tuple: (bits((x + 3) % 8), bits((x + 4) % 8)),
        state: match x % 4 {
            0 => State::Idle,
            1 => State::Write,
            2 => State::Read,
            3 => State::Done,
            _ => unreachable!(),
        },
        inum: if x % 8 == 0 { None } else { Some(bits(x % 64)) },
    });
    let inputs = test_stream(inputs);
    simple_traced_synchronous_run(&circuit, inputs, "pass_through.vcd");
}

#[test]
fn test_triz_pair() {
    #[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
    pub struct Fixture {
        pub snd: trizsnd::U,
        pub rcv: trizrcv::U,
    }

    type I = Option<trizsnd::Cmd>;
    type O = Option<Bits<8>>;

    impl SynchronousIO for Fixture {
        type I = I;
        type O = O;
        type Kernel = fixture;
    }

    #[kernel]
    pub fn fixture(cr: ClockReset, i: I, q: Q) -> (O, D) {
        let mut d = D::init();
        d.rcv.bitz = q.snd.bitz;
        d.rcv.state = q.snd.control;
        d.snd.bitz = q.rcv;
        d.snd.cmd = i;
        (q.snd.data, d)
    }

    let cmd = [
        None,
        None,
        Some(Cmd::Write(bits(0x15))),
        None,
        None,
        Some(Cmd::Read),
        None,
        None,
        None,
        None,
    ];
    /*     let inputs = cmd.iter().map(|x| trizsnd::I {
           bitz: Default::default(),
           cmd: *x,
       });
    */
    let inputs = cmd.into_iter();
    let inputs = test_stream(inputs);
    let uut = Fixture::default();
    simple_traced_synchronous_run(&uut, inputs, "trizsnd.vcd");
}

#[test]
fn test_empty_mut() {
    let mut a = ();
    let b = &mut a;
    *b = ();
}

#[test]
#[allow(clippy::assign_op_pattern)]
fn test_adt_shadow() {
    #[derive(PartialEq, Copy, Clone, Digital, Default)]
    pub enum NooState {
        #[default]
        Init,
        Run(u8, u8, u8),
        Walk {
            foo: u8,
        },
        Boom,
    }

    #[kernel]
    fn do_stuff<C: Domain>(s: Signal<NooState, C>) -> Signal<(u8, NooState), C> {
        let y = bits::<12>(72);
        let foo = bits::<14>(32);
        let mut a: u8 = 0;
        let d = match s.val() {
            NooState::Init => {
                a = 1;
                NooState::Run(1, 2, 3)
            }
            NooState::Walk { foo: x } => {
                a = x;
                NooState::Boom
            }
            NooState::Run(x, _, y) => {
                a = x + y;
                NooState::Walk { foo: 7 }
            }
            NooState::Boom => {
                a = a + 3;
                NooState::Init
            }
        };
        signal((a, d))
    }
}

#[test]
fn test_async_counter() {
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = test_stream(inputs);
    let inputs = inputs.map(|x| {
        timed_sample(
            x.time,
            async_counter::I {
                clock_reset: signal(x.value.0),
                enable: signal(x.value.1),
            },
        )
    });
    let uut: async_counter::U = async_counter::U::default();
    validate(
        &uut,
        inputs,
        &mut [],
        ValidateOptions {
            vcd_filename: Some("async_counter.vcd".into()),
            rtt_filename: Some("async_counter.vcd.rtt".into()),
        },
    )
}

#[test]
fn test_async_counter_fg() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let hdl = uut.hdl("top")?;
    eprintln!("{}", hdl.as_verilog());
    let fg = uut.flow_graph("top")?;
    let mut dot = std::fs::File::create("async_counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_dff_name() -> miette::Result<()> {
    let uut = dff::U::new(b4::from(0b1010));
    eprintln!("name: {}", uut.description());
    Ok(())
}

// TO check with yosys:
// yosys -p "read -vlog95 async_counter.v; hierarchy -check -top rhdl_x_async_counter_U_fb5e6b876dbb9038; proc; write -vlog95 async_counter_yosys.v"
#[test]
fn test_async_counter_hdl() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let hdl = uut.hdl("top")?;
    eprintln!("{}", hdl.as_verilog());
    std::fs::write("async_counter.v", hdl.as_verilog()).unwrap();
    Ok(())
}

#[test]
fn test_async_counter_tb() -> miette::Result<()> {
    let uut = async_counter::U::default();
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = test_stream(inputs);
    let inputs = inputs.map(|x| {
        timed_sample(
            x.time,
            async_counter::I {
                clock_reset: signal(x.value.0),
                enable: signal(x.value.1),
            },
        )
    });
    write_testbench(&uut, inputs, "async_counter_tb.v")?;
    Ok(())
}

#[test]
fn test_adapter_fg() -> miette::Result<()> {
    let counter = counter::U::default();
    let uut = Adapter::<counter::U<2>, Red>::new(counter);
    let fg = &uut.flow_graph("top")?;
    let mut dot = std::fs::File::create("adapter.dot").unwrap();
    write_dot(fg, &mut dot).unwrap();
    Ok(())
}

#[cfg(test)]
fn test_stream<T: Digital>(
    inputs: impl Iterator<Item = T>,
) -> impl Iterator<Item = TimedSample<(ClockReset, T)>> {
    stream::clock_pos_edge(stream::reset_pulse(4).chain(stream::stream(inputs)), 100)
}

#[test]
fn test_dff() {
    let inputs = (0..).map(|_| Bits::init()).take(1000);
    let uut: dff::U<b4> = dff::U::new(b4::from(0b0000));
    simple_traced_synchronous_run(&uut, test_stream(inputs), "dff.vcd");
}

#[test]
fn test_constant() {
    let inputs = (0..).map(|_| ()).take(100);
    let uut: constant::U<b4> = constant::U::new(b4::from(0b1010));
    simple_traced_synchronous_run(&uut, test_stream(inputs), "constant.vcd");
}

#[test]
fn test_strobe() {
    let inputs = (0..).map(|_| strobe::I { enable: true }).take(1000);
    let uut: strobe::U<16> = strobe::U::new(bits(100));
    simple_traced_synchronous_run(&uut, test_stream(inputs), "strobe.vcd");
}

#[test]
fn test_strobe_fg() -> miette::Result<()> {
    let uut: strobe::U<8> = strobe::U::new(bits(100));
    let fg = &uut.flow_graph("uut")?;
    let mut dot = std::fs::File::create("strobe.dot").unwrap();
    write_dot(fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_counter_simulation() {
    let inputs = (0..5000)
        .map(|x| x > 1000 && x < 10000)
        .map(|x| counter::I { enable: x });
    let uut: counter::U<4> = counter::U::default();
    simple_traced_synchronous_run(&uut, test_stream(inputs), "counter.vcd");
}

#[test]
fn test_counter_testbench() -> miette::Result<()> {
    let inputs = (0..1000)
        .map(|x| x > 100 && x < 900)
        .map(|x| counter::I { enable: x });
    let inputs = stream::reset_pulse(1).chain(stream::stream(inputs));
    let uut: counter::U<4> = counter::U::default();
    write_synchronous_testbench(&uut, inputs, 100, "counter_tb.v")?;
    Ok(())
}

#[test]
fn test_state_cycler_vcd() -> miette::Result<()> {
    let uut: state_cycler::U = state_cycler::U::default();
    let disable = state_cycler::I { enable: false };
    let enable = state_cycler::I { enable: true };
    let inputs = repeat(repeat(disable).take(10).chain(repeat(enable).take(10))).take(3);
    let inputs = inputs.flatten();
    let inputs = test_stream(inputs);
    simple_traced_synchronous_run(&uut, inputs, "state_cycler.vcd");
    Ok(())
}

#[test]
fn test_autocounter_vcd() -> miette::Result<()> {
    let uut: auto_counter::U<4> = auto_counter::U::default();
    let inputs = repeat(()).take(1000);
    let stream = test_stream(inputs);
    simple_traced_synchronous_run(&uut, stream, "autocounter.vcd");
    Ok(())
}

#[test]
fn test_autocounter() -> miette::Result<()> {
    let uut: auto_counter::U<4> = auto_counter::U::default();
    let fg = uut.flow_graph("uut")?;
    let inputs = repeat(()).take(1000);
    let stream = test_stream(inputs);
    write_testbench_module(&fg.hdl("autocounter")?, stream, "autocounter_fg_tb.v", 4)?;
    let vg = fg.hdl("top")?;
    std::fs::write("auto_counter.v", vg.as_verilog()).unwrap();
    let mut dot = std::fs::File::create("auto_counter.dot").unwrap();
    write_dot(&fg, &mut dot).unwrap();
    Ok(())
}

#[test]
fn test_auto_doubler() -> miette::Result<()> {
    let c1: auto_counter::U<4> = auto_counter::U::default();
    let c2 = Func::new::<doubler::doubler<4>>()?;
    let uut = Chain::new(c1, c2);
    let inputs = repeat(()).take(1000);
    let stream = test_stream(inputs);
    simple_traced_synchronous_run(&uut, stream, "auto_double.vcd");
    Ok(())
}

#[test]
fn test_auto_doubler_hdl() -> miette::Result<()> {
    let c1: auto_counter::U<4> = auto_counter::U::default();
    let c2 = Func::new::<doubler::doubler<4>>()?;
    let c3: dff::U<b4> = dff::U::new(b4::from(0b0000));
    let c4 = c2.clone();
    let uut = Chain::new(c1, Chain::new(c2, Chain::new(c3, c4)));
    let inputs = repeat(()).take(1000);
    let stream = test_stream(inputs);
    test_synchronous_hdl(
        &uut,
        stream,
        TraceOptions {
            vcd: Some("jnk.vcd".into()),
            assertions_enabled: true,
            ..Default::default()
        },
    )?;
    //    write_testbench_module(&fg.hdl("autodoubler")?, stream, "autodoubler_tb.v", 4)?;
    Ok(())
}

fn main() -> miette::Result<()> {
    let counter: counter::U<4> = counter::U::default();
    let hdl = counter.hdl("uut")?;
    println!("{}", hdl.as_verilog());
    for (child, descriptor) in hdl.children {
        println!("{child} {}", descriptor.as_verilog());
    }
    /*
       let strobe: strobe::U<16> = strobe::U::new(bits(100));
       let hdl = strobe.as_hdl(HDLKind::Verilog)?;
       println!("{}", hdl.body);

       let dff: dff::U<b4> = dff::U::new(b4::from(0b1010));
       let hdl = dff.as_hdl(HDLKind::Verilog)?;
       println!("{}", hdl.body);
    */
    Ok(())
}
