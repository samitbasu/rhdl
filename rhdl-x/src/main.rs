use std::iter::repeat;
use std::time::Instant;

use rhdl_bits::alias::*;
use rhdl_bits::bits;
use rhdl_core::note_db::note_time;
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

fn q(&self) -> outputs {
    (child0.q(), child1.q())
}




In turn, we have something like

fn d(&self, inputs) {

}

*/

// The input and output types
// cannot be auto derived.  Nor
// can the parameters type.
trait ClockedLogic {
    // Input type
    type I: Digital;
    // Output type
    type O: Digital;
    // Parameters type
    type P: Digital;
    // Update function
    type Update: DigitalFn;

    fn i(&mut self, i: Self::I);
    fn o(&self) -> Self::O;
    fn pos_edge(&mut self);
    fn neg_edge(&mut self);
    fn verilog() -> String;
    fn module_name() -> String;

    const UPDATE: LogicUpdateFn<Self>;
}

type LogicUpdateFn<T> = fn(
    <T as ClockedLogic>::P,
    <T as ClockedLogic>::I,
    <T as ClockedLogic>::Q,
) -> (<T as ClockedLogic>::O, <T as ClockedLogic>::D);

// Start with a DFF implementation
#[derive(Default)]
struct DFF<T: Digital> {
    d: T,
    q: T,
}

impl<T: Digital> ClockedLogic for DFF<T> {
    type D = ();
    type Q = ();
    type I = T;
    type O = T;
    type P = ();
    type Update = dff_update<T>;

    fn i(&mut self, i: Self::I) {
        self.d = i;
    }

    fn o(&self) -> Self::O {
        self.q
    }

    fn pos_edge(&mut self) {
        self.q = self.d;
    }

    fn neg_edge(&mut self) {}

    fn module_name() -> String {
        format!("dff_{}", T::bits())
    }

    fn verilog() -> String {
        let bits = T::bits();
        format!(
            "
module {name}(input wire clock, input wire[{BITS}:0] i, output reg[{BITS}:0] o);

    always @(posedge clock) begin
        o <= i;
    end

endmodule

        ",
            name = Self::module_name(),
            BITS = bits.saturating_sub(1)
        )
    }

    const UPDATE: LogicUpdateFn<Self> = dff_update::<T>;
}

#[kernel]
fn dff_update<T: Digital>(p: (), i: T, q: ()) -> (T, ()) {
    (i, ())
}

// Assuming that the DFF above is correct.  We can now
// implement a counter
#[derive(Default)]
struct Counter<const N: usize> {
    count: DFF<Bits<N>>,
}

impl<const N: usize> ClockedLogic for Counter<N> {
    type I = bool;
    type O = Bits<N>;
    type P = ();
    type Update = counter_update<N>;
    const UPDATE: LogicUpdateFn<Self> = counter_update::<N>;
    type Q = Bits<N>;
    type D = Bits<N>;
    fn d(&mut self, d: Self::D) {
        self.count.d(d);
    }
    fn q(&self) -> Self::Q {
        self.count.q()
    }
    fn pos_edge(&mut self) {
        self.count.pos_edge();
    }
    fn neg_edge(&mut self) {
        self.count.neg_edge();
    }
    fn module_name() -> String {
        format!("counter_{}", N)
    }
    fn verilog() -> String {
        "TBD".into()
    }
}

#[kernel]
fn counter_update<const N: usize>(
    params: (),
    enable: bool,
    prev_count: Bits<N>,
) -> (Bits<N>, Bits<N>) {
    let next_count = if enable { prev_count + 1 } else { prev_count };
    (next_count, prev_count)
}

fn simulate<T: ClockedLogic>(
    mut obj: T,
    params: T::P,
    inputs: impl Iterator<Item = T::I>,
) -> impl Iterator<Item = T::O> {
    inputs.map(move |input| {
        let q = obj.q();
        let (o, d) = (T::UPDATE)(params, input, q);
        obj.d(d);
        obj.pos_edge();
        o
    })
}

#[derive(Default)]
struct OneShot<const N: usize> {
    counter: Counter<N>,
    active: DFF<bool>,
}

#[derive(Digital, Default, PartialEq, Clone, Copy)]
struct OneShotParameters<const N: usize> {
    on_duration: Bits<N>,
}

#[kernel]
fn oneshot_update<const N: usize>(
    params: OneShotParameters<N>,
    enable: bool,
    //    (count, active): (Bits<N>, bool),
    count_active_q: (Bits<N>, bool),
) -> (bool, (Bits<N>, bool)) {
    let (count, active) = count_active_q;
    let next_active = if enable {
        if count == params.on_duration {
            false
        } else {
            true
        }
    } else {
        false
    };
    let next_count = if next_active {
        count + 1
    } else {
        bits::<{ N }>(0)
    };

    // HOW DOES INTERNAL STATE GET UPDATED?
    (active, (next_count, next_active))
}

impl<const N: usize> ClockedLogic for OneShot<N> {
    type I = bool;
    type O = bool;
    type P = OneShotParameters<N>;
    // auto derived
    type Q = (
        <Counter<N> as ClockedLogic>::Q,
        <DFF<bool> as ClockedLogic>::Q,
    );
    // auto derived
    type D = (
        <Counter<N> as ClockedLogic>::D,
        <DFF<bool> as ClockedLogic>::D,
    );

    type Update = oneshot_update<N>;
    const UPDATE: LogicUpdateFn<Self> = oneshot_update::<N>;

    // auto derived
    fn d(&mut self, d: Self::D) {
        self.counter.d(d.0);
        self.active.d(d.1);
    }

    // auto derived
    fn q(&self) -> Self::Q {
        (self.counter.q(), self.active.q())
    }

    // auto derived
    fn pos_edge(&mut self) {
        self.counter.pos_edge();
        self.active.pos_edge();
    }

    // auto derived
    fn neg_edge(&mut self) {
        self.counter.neg_edge();
        self.active.neg_edge();
    }

    fn module_name() -> String {
        format!("oneshot_{}", N)
    }

    // auto derived
    fn verilog() -> String {
        "TBD".into()
    }
}

fn main() {
    let input = repeat(false)
        .take(10)
        .chain(repeat(true).take(1))
        .chain(repeat(false).take(50));
    let one_shot_param = OneShotParameters::<8> {
        on_duration: bits(10),
    };
    let mut one_shot = OneShot::<8>::default();
    let mut one_shot_sim = simulate(one_shot, one_shot_param, input);
    one_shot_sim.for_each(|o| println!("one_shot: {}", o));
}
