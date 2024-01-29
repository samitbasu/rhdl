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

trait ClockedLogic: Default {
    type D: Digital;
    type Q: Digital;
    fn d(&mut self, d: Self::D);
    fn q(&self) -> Self::Q;
    fn pos_edge(&mut self);
    fn neg_edge(&mut self);
    fn verilog() -> String;
    fn module_name() -> String;
}

trait Synch: Digital {
    type I: Digital;
    type O: Digital;
    type L: ClockedLogic;
    type Update: DigitalFn;

    const UPDATE: SynchUpdateFn<Self>;
}

type SynchUpdateFn<T> = fn(
    T,
    <T as Synch>::I,
    <<T as Synch>::L as ClockedLogic>::Q,
) -> (<<T as Synch>::L as ClockedLogic>::D, <T as Synch>::O);

#[derive(Digital, Default, PartialEq, Clone, Copy)]
struct Counter<const N: usize> {}

impl<const N: usize> Synch for Counter<N> {
    type I = bool;
    type O = Bits<N>;
    type L = DFF<Bits<N>>;
    type Update = counter_update<N>;

    const UPDATE: SynchUpdateFn<Self> = counter_update::<N>;
}

#[kernel]
fn counter_update<const N: usize>(
    params: Counter<N>,
    enable: bool,
    prev_count: Bits<N>,
) -> (Bits<N>, Bits<N>) {
    let next_count = if enable { prev_count + 1 } else { prev_count };
    (next_count, prev_count)
}

fn simulate<T: Synch>(obj: T, inputs: impl Iterator<Item = T::I>) -> impl Iterator<Item = T::O> {
    let mut state = T::L::default();
    inputs.map(move |input| {
        let q = state.q();
        let (d, o) = (T::UPDATE)(obj, input, q);
        state.d(d);
        state.pos_edge();
        o
    })
}

struct OneShot<const N: usize> {
    counter: Counter<N>,
    active: DFF<bool>,
}

struct OneShotParameters<const N: usize> {
    on_duration: Bits<N>,
}

impl Synch for OneShot<N> {
    type I = bool;
    type O = bool;
    type P = OneShotParameters<N>;
}

#[derive(Default)]
struct DFF<T: Digital> {
    d: T,
    q: T,
}

impl<T: Digital> ClockedLogic for DFF<T> {
    type D = T;
    type Q = T;

    fn d(&mut self, d: Self::D) {
        self.d = d;
    }

    fn q(&self) -> Self::Q {
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
module {name}(input wire clock, input wire[{BITS}:0] d, output reg[{BITS}:0] q);

    always @(posedge clock) begin
        q <= d;
    end

endmodule

        ",
            name = Self::module_name(),
            BITS = bits.saturating_sub(1)
        )
    }
}

// Example of aggregation
#[derive(Default)]
struct FooClocked {
    c1: DFF<b8>,
    c2: DFF<bool>,
}

impl ClockedLogic for FooClocked {
    type D = (b8, bool);
    type Q = (b8, bool);

    fn d(&mut self, d: Self::D) {
        self.c1.d(d.0);
        self.c2.d(d.1);
    }

    fn q(&self) -> Self::Q {
        (self.c1.q(), self.c2.q())
    }

    fn pos_edge(&mut self) {
        self.c1.pos_edge();
        self.c2.pos_edge();
    }

    fn neg_edge(&mut self) {
        self.c1.neg_edge();
        self.c2.neg_edge();
    }

    fn module_name() -> String {
        "foo_clocked".into()
    }

    fn verilog() -> String {
        format!(
            "
module {name}(input wire clock, input wire[{D_BITS}:0] d, output reg[{Q_BITS}:0] q);
        {c1_mod} {c1_mod}_inst(.clock(clock), .d(d[{C1_END_D}:{C1_START_D}]), .q(q[{C1_END_Q}:{C1_START_Q}]);
        {c2_mod} {c2_mod}_inst(.clock(clock), .d(d[{C2_END_D}:{C2_START_D}]), .q(q[{C2_END_Q}:{C2_START_Q}]);
endmodule
",
            name = Self::module_name(),
            D_BITS = <Self::D as Digital>::bits().saturating_sub(1),
            Q_BITS = <Self::Q as Digital>::bits().saturating_sub(1),
            C1_START_D = 0,
            C1_END_D = 7,
            C2_START_D = 8,
            C2_END_D = 8,
            C1_START_Q = 0,
            C1_END_Q = 7,
            C2_START_Q = 8,
            C2_END_Q = 8,
            c1_mod = <DFF<b8> as ClockedLogic>::module_name(),
            c2_mod = <DFF<bool> as ClockedLogic>::module_name(),
        )
    }
}

trait ClockedModule {
    type Input: Digital;
    type Output: Digital;
    fn name() -> String;
    fn as_verilog() -> String;
}

fn make_verilog<T: ClockedModule>() -> String {
    format!("
module {name}(input wire clock, input wire[{INPUT_BITS}:0] {name}_in, output reg[{OUTPUT_BITS}:0] {name}_out);

    {body}

endmodule ", name = T::name(), INPUT_BITS=T::Input::bits().saturating_sub(1), 
OUTPUT_BITS=T::Output::bits().saturating_sub(1), body=T::as_verilog())
}

struct Adder {}

impl ClockedModule for Adder {
    type Input = (b8, b8);
    type Output = b8;
    fn name() -> String {
        "adder_8".into()
    }
    fn as_verilog() -> String {
        "
        always @(posedge clock) begin 
            adder_8_out <= adder_8_in[7:0] + adder_8_in[15:8];
        end
        "
        .into()
    }
}

struct Doubler {}

impl ClockedModule for Doubler {
    type Input = b8;
    type Output = b8;
    fn name() -> String {
        "doubler_8".into()
    }
    fn as_verilog() -> String {
        "
        always @(posedge clock) begin
           doubler_8_out <= (doubler_8_in << 1);
        end
        "
        .into()
    }
}

struct Pair<A: ClockedModule, B: ClockedModule> {
    a: std::marker::PhantomData<A>,
    b: std::marker::PhantomData<B>,
}

impl<A: ClockedModule, B: ClockedModule> ClockedModule for Pair<A, B> {
    type Input = b8;
    type Output = b8;

    fn name() -> String {
        "two_x_and_1".into()
    }
    fn as_verilog() -> String {
        "
        wire [{A_INPUT_BITS}:0] a_in;
        wire [{A_OUTPUT_BITS}:0] a_out;
        {A} mod_a(.clock(clock), .input(a_in), .output(a_out));
        wire [{B_INPUT_BITS}:0] b_in;
        wire [{B_OUTPUT_BITS}:0] b_out;
        {B} mod_b(.clock(clock), .input(b_in), .output(b_out));

        assign a_in = {self_in, b_out};
        assign b_in = self_in;
        assign c_out = a_out;

        "
        .into()
    }
}

struct Widget {}

fn main() {}
