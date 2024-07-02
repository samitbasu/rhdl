use rhdl_core::prelude::*;
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

    fn init_state(&self) -> Self::S {
        Self::S::random()
    }

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

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor> {
        ensure!(kind == HDLKind::Verilog);
        Ok(self.as_verilog())
    }
}

fn main() {
    println!("Hello world!");
}
