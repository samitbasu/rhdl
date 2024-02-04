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

/*

The following macro_rules takes a circuit definition and generates a set of structs
and impls for it.  Given an invokation of the form:

circuit! {
    name: MyCircuit

    inputs: (a: u8, b: u8)

    outputs: (c: u8, d: u8)

    elements: (child0: Child0, child1: Child1)
}

It generates the following structs:

pub struct MyCircuit {
    child0: Child0,
    child1: Child1,
}

#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyCircuitI {
    a: u8,
    b: u8,
}

#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyCircuitO {
    c: u8,
    d: u8,
}

#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyCircuitQ {
    child0: Child0O,
    child1: Child1O,
}

#[derive(Copy, Clone, PartialEq, Digital)]
pub struct MyCircuitD {
    child0: Child0I,
    child1: Child1I,
}

 */

use paste::paste;
macro_rules! circuit {
    (
        name: $name:ident

        inputs: ($($input:ident: $input_type:ty),*)

        outputs: ($($output:ident: $output_type:ty),*)

        elements: ($($element:ident: $element_type:ty),*)
    ) => {
        paste!{
        pub struct $name {
            $($element: $element_type,)*
        }

        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        pub struct [<$name I>] {
            $($input: $input_type,)*
        }

        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        pub struct [<$name O>] {
            $($output: $output_type,)*
        }

        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        pub struct [<$name Q>] {
            $($element: <$element_type as Circuit>::O,)*
        }

        #[derive(Copy, Clone, PartialEq, Default, Digital)]
        pub struct [<$name D>]{
            $($element: <$element_type as Circuit>::I,)*
        }
    }
    }
}

circuit! {
    name: MyStrobe

    inputs: (clock: Clock, enable: bool)

    outputs: (strobe: bool)

    elements: (threshold: Constant<Bits<8>>, counter: DFF<Bits<8>>)
}

pub trait Circuit {
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
    type S: Default + PartialEq + Clone;

    // Simulation update - auto derived
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;

    fn init_state(&self) -> Self::S {
        Default::default()
    }
}

// New type for the clock
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct Clock(pub bool);

// Constant block
pub struct Constant<T: Digital> {
    value: T,
}

impl<T: Digital> From<T> for Constant<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> DigitalFn for Constant<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

impl<T: Digital> Circuit for Constant<T> {
    type I = ();

    type O = T;

    type Q = ();

    type D = ();

    type S = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |_, _| (T::default(), ());

    fn sim(&self, _: Self::I, _: &mut Self::S) -> Self::O {
        self.value
    }
}

// First a DFF

#[derive(Default)]
pub struct DFF<T: Digital> {
    phantom: std::marker::PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct DFFI<T: Digital> {
    pub clock: Clock,
    pub data: T,
}

impl<T: Digital> Circuit for DFF<T> {
    type I = DFFI<T>;

    type O = T;

    type Q = ();

    type D = ();

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| (i.data, ());

    type S = DFFI<T>;

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
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

impl<T: Digital> DigitalFn for DFF<T> {
    fn kernel_fn() -> rhdl_core::KernelFnKind {
        todo!()
    }
}

// Next a counter with an enable signal
#[derive(Default)]
pub struct Counter<const N: usize> {
    count: DFF<Bits<N>>,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct CounterI<const N: usize> {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> Circuit for Counter<N> {
    type I = CounterI<N>;

    type O = Bits<N>;

    type Q = (Bits<N>,);

    type D = (DFFI<Bits<N>>,);

    type Update = counter<N>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = counter::<N>;

    type S = (Self::Q, <DFF<Bits<N>> as Circuit>::S);

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        loop {
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            let o0 = self.count.sim(internal_inputs.0, &mut state.1);
            state.0 = (o0,);
            if state == &prev_state {
                return outputs;
            }
        }
    }
}

#[kernel]
pub fn counter<const N: usize>(
    i: CounterI<N>,
    (count_q,): (Bits<N>,),
) -> (Bits<N>, (DFFI<Bits<N>>,)) {
    let count_next = if i.enable { count_q + 1 } else { count_q };
    (
        count_q,
        (DFFI::<Bits<{ N }>> {
            clock: i.clock,
            data: count_next,
        },),
    )
}

// Build a strobe
pub struct Strobe<const N: usize> {
    threshold: Constant<Bits<N>>,
    counter: DFF<Bits<N>>,
}

impl<const N: usize> Strobe<N> {
    pub fn new(param: Bits<N>) -> Self {
        Self {
            threshold: param.into(),
            counter: DFF::default(),
        }
    }
}

// Can we autoderive something like:
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeQ<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::O,
    counter: <DFF<Bits<N>> as Circuit>::O,
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeD<const N: usize> {
    threshold: <Constant<Bits<N>> as Circuit>::I,
    counter: <DFF<Bits<N>> as Circuit>::I,
}

impl<const N: usize>
    From<(
        <Constant<Bits<N>> as Circuit>::O,
        <DFF<Bits<N>> as Circuit>::O,
    )> for StrobeQ<N>
{
    fn from(
        (threshold, counter): (
            <Constant<Bits<N>> as Circuit>::O,
            <DFF<Bits<N>> as Circuit>::O,
        ),
    ) -> Self {
        Self { threshold, counter }
    }
}

#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct StrobeI<const N: usize> {
    pub clock: Clock,
    pub enable: bool,
}

impl<const N: usize> Circuit for Strobe<N> {
    type I = StrobeI<N>;

    type O = bool;

    type Q = StrobeQ<N>;

    type D = StrobeD<N>;

    type S = (
        Self::Q,
        <Constant<Bits<N>> as Circuit>::S,
        <DFF<Bits<N>> as Circuit>::S,
    );

    type Update = strobe<N>;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = strobe::<N>;

    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O {
        loop {
            let prev_state = state.clone();
            let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
            state.0.threshold = self.threshold.sim(internal_inputs.threshold, &mut state.1);
            state.0.counter = self.counter.sim(internal_inputs.counter, &mut state.2);
            if state == &prev_state {
                return outputs;
            }
        }
    }
}

#[kernel]
pub fn strobe<const N: usize>(i: StrobeI<N>, q: StrobeQ<N>) -> (bool, StrobeD<N>) {
    let counter_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let counter_next = if strobe {
        bits::<{ N }>(1)
    } else {
        counter_next
    };
    (
        strobe,
        StrobeD::<{ N }> {
            threshold: (),
            counter: DFFI::<Bits<{ N }>> {
                clock: i.clock,
                data: counter_next,
            },
        },
    )
}

fn clock() -> impl Iterator<Item = Clock> {
    std::iter::once(Clock(true))
        .chain(std::iter::once(Clock(false)))
        .cycle()
}

#[test]
fn test_dff() {
    let clock = clock();
    let data = (0..10).cycle();
    let inputs = clock.zip(data).map(|(clock, data)| DFFI { clock, data });
    note_init_db();
    note_time(0);
    let dff = DFF::<u8>::default();
    let mut state = dff.init_state();
    for (time, input) in inputs.enumerate().take(1000) {
        note_time(time as u64 * 1_000);
        note("input", input);
        let output = dff.sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("dff.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();
}

#[test]
fn test_strobe() {
    let clock = clock();
    let enable = std::iter::repeat(true);
    let inputs = clock
        .zip(enable)
        .map(|(clock, enable)| StrobeI { clock, enable });
    note_init_db();
    note_time(0);
    let strobe = Strobe::<8>::new(b8(5));
    let mut state = strobe.init_state();
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = strobe.sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let strobe = std::fs::File::create("strobe.vcd").unwrap();
    db.dump_vcd(&[], strobe).unwrap();
}

fn main() {
    let clock = clock();
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
    for (time, input) in inputs.enumerate().take(2000) {
        note_time(time as u64 * 100);
        note("input", input);
        let output = counter.sim(input, &mut state);
        note("output", output);
    }
    let db = note_take().unwrap();
    let dff = std::fs::File::create("counter.vcd").unwrap();
    db.dump_vcd(&[], dff).unwrap();

    let foo = MyStrobe {
        threshold: b8(5).into(),
        counter: DFF::default(),
    };
    let boo = MyStrobeI {
        clock: Clock(true),
        enable: true,
    };
}
