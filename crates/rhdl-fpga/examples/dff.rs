use rhdl::prelude::*;
use rhdl_fpga::{core::dff::DFF, doc::write_svg_as_markdown};

// We create a state enum to track the number
// of zeros we have seen.  This could just be
// counter, but it's a bit easier to read this way.
#[derive(PartialEq, Digital, Debug, Default)]
pub enum State {
    // Enums need to have a default
    // value.  In this case, it's
    // safe to assume that the default
    // is no zeros.
    #[default]
    NoZeros,
    OneZero,
    TwoZeroes,
}

// This is the core itself.  We derive these
// 4 traits to provide the needed functions
#[derive(Synchronous, Clone, Debug, SynchronousDQ)]
pub struct Recognizer {
    state: DFF<State>,
}

impl Default for Recognizer {
    fn default() -> Self {
        Self {
            state: DFF::new(State::NoZeros),
        }
    }
}

// The SynchronousIO trait tells RHDL what the
// input and output types are (both bools in this case),
// as well as where the kernel can be found.
impl SynchronousIO for Recognizer {
    type I = bool;
    type O = bool;
    type Kernel = kernel;
}

#[kernel]
// This is the kernel.  Hopefully self evident.
pub fn kernel(_cr: ClockReset, i: bool, q: Q) -> (bool, D) {
    let (o, d) = match (q.state, i) {
        (State::NoZeros, false) => (false, State::OneZero),
        (State::NoZeros, true) => (false, State::NoZeros),
        (State::OneZero, false) => (false, State::TwoZeroes),
        (State::OneZero, true) => (false, State::NoZeros),
        (State::TwoZeroes, false) => (false, State::TwoZeroes),
        (State::TwoZeroes, true) => (true, State::NoZeros),
    };
    (o, D { state: d })
}

fn main() -> Result<(), RHDLError> {
    // Recognize the sequence `0 0 1`
    let input = [false, true, false, true, true, false, false, true, false];
    let input = input.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = Recognizer::default();
    let vcd = uut.run(input)?.collect::<Vcd>();
    let options = SvgOptions::default().with_label_width(20);
    write_svg_as_markdown(vcd, "dff.md", options)?;
    Ok(())
}
