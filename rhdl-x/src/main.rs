use std::iter::repeat;

use anyhow::ensure;
use rhdl_core::as_verilog_literal;
use rhdl_core::codegen::verilog::as_verilog_decl;
use rhdl_core::prelude::*;
use rhdl_core::root_descriptor;
use rhdl_core::types::domain::Red;
use rhdl_macro::Digital;
use rhdl_macro::Timed;

//use translator::Translator;
//use verilog::VerilogTranslator;

//mod backend;
//mod circuit;
//mod clock;
//mod constant;
//mod counter;
//mod descriptions;
//mod dff;
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

//#[cfg(test)]
//mod tests;

// Let's start with the DFF.  For now, we will assume a reset is present.

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct DFFI<T: Digital> {
    pub data: T,
    pub clock: Clock,
    pub reset: Reset,
}

// The DFF itself has a reset value, and a clock domain.
#[derive(Default, Clone)]
pub struct DFF<T: Digital, C: Domain> {
    reset: T,
    clock: std::marker::PhantomData<C>,
}

impl<T: Digital, C: Domain> CircuitIO for DFF<T, C> {
    type I = Signal<DFFI<T>, C>;
    type O = Signal<T, C>;
}

impl<T: Digital, C: Domain> Circuit for DFF<T, C> {
    type Q = ();
    type D = ();
    type Z = ();

    type Update = Self;

    type S = DFFI<T>;

    fn sim(&self, input: Self::I, state: &mut Self::S, _io: &mut Self::Z) -> Self::O {
        note("input", input);
        let output = if input.val().clock.raw() && !state.clock.raw() {
            input.val().data
        } else {
            state.data
        };
        state.data = output;
        state.clock = input.val().clock;
        let output = if input.val().reset.raw() {
            state.data = self.reset;
            self.reset
        } else {
            output
        };
        note("output", output);
        signal(output)
    }

    fn name(&self) -> &'static str {
        "DFF"
    }

    fn as_hdl(&self, kind: HDLKind) -> anyhow::Result<HDLDescriptor> {
        ensure!(kind == HDLKind::Verilog);
        Ok(self.as_verilog())
    }

    fn descriptor(&self) -> CircuitDescriptor {
        root_descriptor(self)
    }
}

impl<T: Digital, C: Domain> DFF<T, C> {
    fn as_verilog(&self) -> HDLDescriptor {
        let module_name = self.descriptor().unique_name;
        let input_bits = DFFI::<T>::bits();
        let output_bits = T::bits();
        let init = as_verilog_literal(&self.reset.typed_bits());
        let input_wire = as_verilog_decl("wire", input_bits, "i");
        let output_reg = as_verilog_decl("reg", output_bits, "o");
        let d = as_verilog_decl("wire", T::bits(), "d");
        let code = format!(
            "
module {module_name}(input {input_wire}, output {output_reg});
   wire clk;
   wire rst;
   wire {d};
   assign {{rst, clk, d}} = i;
   initial begin
        o = {init};
   end
   always @(posedge clk) begin 
        if (rst) begin
            o <= {init};
        end else begin
            o <= {d};
        end
    end
endmodule
            "
        );
        HDLDescriptor {
            name: module_name,
            body: code,
            children: Default::default(),
        }
    }
}

impl<T: Digital, C: Domain> DigitalFn for DFF<T, C> {}

pub fn sim_reset() -> impl Iterator<Item = Reset> {
    repeat(reset(true)).take(4).chain(repeat(reset(false)))
}

pub fn sim_clock() -> impl Iterator<Item = Clock> {
    std::iter::once(clock(true))
        .chain(std::iter::once(clock(false)))
        .cycle()
}

pub fn sim_samples<T: Digital>() -> impl Iterator<Item = Signal<DFFI<T>, Red>> {
    sim_clock()
        .zip(sim_reset())
        .zip(std::iter::repeat(T::default()))
        .map(|((clock, reset), data)| DFFI { data, clock, reset })
}

#[test]
fn test_dff() {
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

fn main() -> anyhow::Result<()> {
    let dff = DFF::<b4, Red> {
        reset: b4::from(0b1010),
        clock: Default::default(),
    };
    let hdl = dff.as_hdl(HDLKind::Verilog)?;
    println!("{}", hdl.body);
    Ok(())
}
