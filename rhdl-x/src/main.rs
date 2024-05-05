use counter::Counter;
use counter::CounterI;
use rhdl_core::note;
use rhdl_core::note_db::note_time;
use rhdl_core::note_init_db;
use rhdl_core::note_take;
use rhdl_core::Circuit;

//use translator::Translator;
//use verilog::VerilogTranslator;

//mod backend;
//mod circuit;
mod clock;
mod constant;
mod counter;
mod descriptions;
mod dff;
mod push_pull;
mod strobe;
mod tristate;
//mod traitx;
//mod translator;
//mod verilog;
//mod dfg;
//mod trace;
mod chalk;
mod check;
mod signal;
mod timeset;
mod visit;

fn main() {
    let clock = clock::clock();
    let enable = std::iter::repeat(false)
        .take(20)
        .chain(std::iter::repeat(true));
    let inputs = clock
        .zip(enable)
        .map(|(clock, enable)| CounterI { clock, enable });
    note_init_db();
    note_time(0);
    let counter = Counter::<8>::default();
    let mut state = counter.init_state();
    let mut io = <Counter<8> as Circuit>::Z::default();
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = counter.sim(input, &mut state, &mut io);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("counter.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::{
        dff::{DFF, DFFI},
        strobe::{Strobe, StrobeI},
    };
    use rhdl_bits::alias::*;
    use rhdl_core::{compile_design, note_pop_path, note_push_path, Digital, HDLKind};
    use rhdl_macro::{kernel, Digital};
    use rhdl_std::slice;

    use super::*;

    #[derive(Debug, Clone, PartialEq, Copy, Default, Digital)]
    struct Bar(u8, u8, bool);

    #[derive(Debug, Clone, PartialEq, Copy, Digital, Default)]
    struct Foo {
        a: u8,
        b: u8,
        c: bool,
    }

    #[derive(Debug, Clone, PartialEq, Copy, Digital)]
    enum Baz {
        A(Bar),
        B { foo: Foo },
        C(u8),
    }

    #[test]
    fn test_static_kind_from_adt() {
        let kind = Baz::static_kind();
        eprintln!("{:?}", kind);
    }

    #[kernel]
    fn debaz(a: Baz) -> u8 {
        match a {
            Baz::A(b) => b.0,
            Baz::B { foo } => foo.a,
            Baz::C(c) => c,
        }
    }

    #[test]
    fn test_dff() {
        let clock = clock::clock();
        let data = (0..10).cycle();
        let inputs = clock.zip(data).map(|(clock, data)| DFFI { clock, data });
        note_init_db();
        note_time(0);
        let dff = DFF::<u8>::default();
        let mut state = dff.init_state();
        let mut io = <DFF<u8> as Circuit>::Z::default();
        for (time, input) in inputs.enumerate().take(1000) {
            note_time(time as u64 * 1_000);
            note("input", input);
            let output = dff.sim(input, &mut state, &mut io);
            note("output", output);
        }
        let db = note_take().unwrap();
        let dff = std::fs::File::create("dff.vcd").unwrap();
        db.dump_vcd(&[], dff).unwrap();
    }

    #[test]
    fn test_strobe() {
        let clock = clock::clock();
        let enable = std::iter::repeat(true);
        let inputs = clock
            .zip(enable)
            .map(|(clock, enable)| StrobeI { clock, enable });
        note_init_db();
        note_time(0);
        let strobe = Strobe::<8>::new(b8(5));
        let mut state = strobe.init_state();
        let mut io = <Strobe<8> as Circuit>::Z::default();
        for (time, input) in inputs.enumerate().take(2000) {
            note_time(time as u64 * 100);
            note("input", input);
            let output = strobe.sim(input, &mut state, &mut io);
            note("output", output);
        }
        let db = note_take().unwrap();
        let strobe = std::fs::File::create("strobe.vcd").unwrap();
        db.dump_vcd(&[], strobe).unwrap();
    }

    #[test]
    fn test_strobe_verilog() {
        let strobe = Strobe::<8>::new(b8(5));
        let top = strobe.as_hdl(HDLKind::Verilog).unwrap();
        let verilog = format!(
            "
    module top;
    reg clk;
    reg enable;
    wire out;
  
    initial begin
       enable = 1;
       clk = 1;
       forever #10 clk = ~clk;
    end
  
    Strobe_748a98de03e4aa30 dut(.i({{enable, clk}}), .o(out) );
  
    initial begin
    $dumpfile(\"strobe_v.vcd\");
    $dumpvars(0);
    #1000;
    $finish;
    end
  
  
  endmodule
    
    {}",
            top
        );
        std::fs::write("strobe.v", verilog).unwrap();
    }

    #[test]
    fn test_timing_note() {
        #[derive(Copy, Clone, PartialEq, Digital, Default)]
        pub enum State {
            #[default]
            A,
            B,
            C,
        };
        note_init_db();
        note_time(0);
        let tic = Instant::now();
        for i in 0..1_000_000 {
            note_time(i);
            note_push_path("a");
            note_push_path("b");
            note_push_path("c");
            note("i", b8(4));
            note_pop_path();
            note_pop_path();
            note_pop_path();
            note_push_path("a");
            note_push_path("b");
            note_push_path("d");
            note("name", b16(0x1234));
            note_pop_path();
            note_pop_path();
            note_pop_path();
            note_push_path("b");
            note_push_path("c");
            note_push_path("e");
            note("color", State::B);
            note_pop_path();
            note_pop_path();
            note_pop_path();
        }
        let toc = Instant::now();
        eprintln!("Time: {:?}", toc - tic);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Digital, Copy, Clone, PartialEq, Default)]
        pub struct Foo(b8, b8);

        let kind = Foo::static_kind();
        eprintln!("{:?}", kind);
    }

    #[test]
    fn test_dfg_analysis_of_kernel() {
        use rhdl_std::UnsignedMethods;
        #[derive(Copy, Clone, PartialEq, Digital, Default)]
        pub struct Foo {
            a: bool,
            b: b4,
            c: b4,
        }

        #[derive(Copy, Clone, PartialEq, Digital)]
        pub enum Bar {
            A(b8),
            B(Foo),
            C { x: b4, y: b4 },
        }

        impl Default for Bar {
            fn default() -> Self {
                Bar::A(b8(0))
            }
        }

        #[kernel]
        fn concatenate_bits(x: b4, y: b4) -> (b4, b4) {
            let d = Foo {
                a: true,
                b: x,
                c: y,
            };
            let e = Foo { a: false, ..d };
            let f = Bar::B(e);
            let g = Bar::C { x, y };
            let z = match f {
                Bar::B(b) => b.b,
                _ => b4(0),
            };
            (y - x, y + 3 + z)
        }

        #[kernel]
        fn add_stuff(x: b4, y: b4, z: Foo) -> b4 {
            let h = [b4(5); 16];
            let c = slice::<4, 4>(h[0], 1);
            let q = x + concatenate_bits(x, y).0 + z.b;
            match z.c.xor() {
                true => q + z.c,
                false => q - z.c + h[z.b] - b4(5) + c,
            }
        }

        let design = compile_design::<add_stuff>().unwrap();
    }
}
