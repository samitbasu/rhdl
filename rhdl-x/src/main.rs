use std::iter::repeat;
use std::time::Instant;

use rhdl_bits::alias::*;
use rhdl_bits::bits;
use rhdl_core::note_db::note_time;
use rhdl_core::note_init_db;
use rhdl_core::note_take;
use rhdl_core::DigitalFn;
use rhdl_core::{note, Digital};
use rhdl_macro::kernel;
use rhdl_macro::Digital;
use rhdl_x::Foo;

use rhdl_bits::Bits;

// Consider this example:
/*

#[derive(LogicBlock)]
pub struct Pulser {
    pub clock: Signal<In, Clock>,
    pub enable: Signal<In, Bit>,
    pub pulse: Signal<Out, Bit>,
    strobe: Strobe<32>,
    shot: Shot<32>,
}

The update function for this looked like:

fn update(&mut self) {

}

To make it functional, we want instead to do something like

fn update(params, inputs, (internal_outputs)) -> (outputs, (internal_inputs)) {}


In simulation, we then want to do something like:

let i = next_input;
let p = params;
let q = obj.q();
let (o,d) = T::Update(p, i, q);
obj.d(d);


We can eliminate parameters by re-introducing the notion of a Constant object
which does not change.


// Assuming that we thus eliminate the clock and input/output signals, we get
// down to something like

#[derive(LogicBlock)]
pub struct Pulser {
   params: Constant<T>,
   strobe: Strobe<32>,
   shot: Shot<32>,
}

This solves the problem of design parameters, since they are simply injected into
the list of inputs.

Now the update function looks like:

fn update(inputs, (internal_outputs)) -> (outputs, (internal_inputs)) {}


We need

trait LogicBlock {
    type I: Digital;
    type O: Digital;
    type Q: Digital;
    type D: Digital;
    type S: Digital;
}

fn update(inputs: I, internal_outputs: D) -> (outputs: O, internal_inputs: Q) {
    // This is the update function
    // user defined...
}


fn sim(&mut self, inputs: I, internal_state: S, internal_outputs: Q) -> O {
    loop {
        let (outputs, internal_inputs) = Self::Update(inputs, internal_outputs);
        let o0 = chil0.sim(internal_inputs.0, internal_state.0,)
    }
}


Can we then do something like:

fn d(&mut self, inputs) -> outputs {
    loop {
        let internal_outputs = self.q();
        let (outputs, internal_inputs) = Self::Update(inputs, internal_outputs);
        child0.d(internal_inputs.0);
        child1.d(internal_inputs.1);
        if self.q() == internal_outputs {
            return outputs;
        }
    }
}

not quite.  self.q() is not as described below.

fn q(&self) -> outputs {
    (child0.q(), child1.q())
}

Somewhere, we must store the internal outputs of the child blocks.

fn update(&mut self, inputs, internal_outputs) -> (outputs, internal_outputs) {
    loop {
        let (outputs, internal_inputs) = Self::Update(inputs, internal_outputs);
        ??
    }
}



In turn, we have something like

fn d(&self, inputs) {

}

*/

trait Circuit {
    // Input type - not auto derived
    type I: Digital;
    // Output type - not auto derived
    type O: Digital;

    // Outputs of internal circuitry - auto derived
    type Q: Digital;
    // Inputs of internal circuitry - auto derived
    type D: Digital;

    type Update: DigitalFn;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D);

    // State for simulation - auto derived
    type S: Default;

    // Simulation update - auto derived
    fn sim(input: Self::I, state: &mut Self::S) -> Self::O;
}

// New type for the clock
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct Clock(pub bool);

// First a DFF

pub struct DFF<T: Digital> {
    phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct DFFIn<T: Digital> {
    pub clock: Clock,
    pub data: T,
}

impl<T: Digital> Circuit for DFF<T> {
    type I = DFFIn<T>;

    type O = T;

    type Q = ();

    type D = ();

    type Update = dff<T>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = dff::<T>;

    type S = DFFIn<T>;

    fn sim(input: Self::I, state: &mut Self::S) -> Self::O {
        let output = if input.clock.0 && !state.clock.0 {
            input.data
        } else {
            state.data
        };
        state.clock = input.clock;
        state.data = output;
        output
    }
}

#[kernel]
fn dff<T: Digital>(i: DFFIn<T>, q: ()) -> (T, ()) {
    (i.data, ())
}

fn main() {
    let clock_cycle = std::iter::once(Clock(true)).chain(std::iter::once(Clock(false)));
    let clock = clock_cycle.cycle();
    let data = (0..10).cycle();
    let inputs = clock.zip(data).map(|(clock, data)| DFFIn { clock, data });
    note_init_db();
    note_time(0);
    let mut state = <DFF<u8> as Circuit>::S::default();
    for (time, input) in inputs.enumerate().take(1000) {
        note_time(time as u64 * 1_000);
        note("input", input);
        let output = <DFF<u8> as Circuit>::sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("dff.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}
