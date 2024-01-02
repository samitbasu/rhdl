// A Synchronous design consists of the following pieces:
//
//  Input   - a Digital type that describes the input to the design
// Output   - a Digital type that describes the output of the design
//  State   - a Digital type that describes the state of the design
//  Clock   - the clock signal for the design (may be implicit)
//  Initial - the initial state of the design
//  Update  - the update function for the design.
//  Params  - the parameters for the design (held constant)

use anyhow::Result;
use rhdl_bits::{bits, Bits};
use rhdl_core::{note, note_init_db, note_take, note_time, Digital};
use rhdl_macro::{kernel, Digital};

pub trait Synchronous: Sized {
    type Input: Digital;
    type Output: Digital;
    type State: Digital;
    type Params: Digital;

    const INITIAL_STATE: Self::State;

    fn params(&self) -> Self::Params;
}

// Maybe we don't need a trait.

// Let's try a simple strobe

pub struct Strobe<const N: usize> {
    pub period: Bits<N>,
}

impl<const N: usize> Synchronous for Strobe<N> {
    type Input = StrobeInput;
    type Output = StrobeOutput;
    type State = StrobeState<N>;
    type Params = StrobeParams<N>;

    const INITIAL_STATE: Self::State = StrobeState::<{ N }> {
        count: bits::<{ N }>(0),
    };

    fn params(&self) -> Self::Params {
        StrobeParams::<{ N }> {
            period: self.period,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StrobeInput {
    pub enable: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StrobeOutput {
    pub active: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StrobeState<const N: usize> {
    pub count: Bits<N>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StrobeParams<const N: usize> {
    pub period: Bits<N>,
}

#[kernel]
fn update<const N: usize>(
    params: StrobeParams<N>,
    state: StrobeState<N>,
    input: StrobeInput,
) -> (StrobeState<N>, StrobeOutput) {
    let mut count = state.count;
    let mut active = false;

    if input.enable {
        count += 1;
        if count == params.period {
            count = bits::<{ N }>(0);
            active = true;
        }
    }
    note("active", active);

    (StrobeState::<{ N }> { count }, StrobeOutput { active })
}

pub fn simulate<M: Synchronous, F>(
    obj: &M,
    inputs: impl Iterator<Item = M::Input>,
    update: F,
) -> impl Iterator<Item = M::Output>
where
    F: Fn(M::Params, M::State, M::Input) -> (M::State, M::Output),
{
    let params = obj.params();
    let mut state = M::State::default();
    note_time(0);
    let mut time = 0;
    inputs.map(move |input| {
        let (new_state, output) = update(params, state, input);
        state = new_state;
        time += 1_000;
        note_time(time);
        output
    })
}

#[test]
fn test_strobe_simulation() {
    let enable = std::iter::repeat(StrobeInput { enable: true }).take(1_000_000);
    let strobe = Strobe::<16> { period: bits(100) };
    let now = std::time::Instant::now();
    note_init_db();
    let outputs = simulate(&strobe, enable, update)
        .filter(|x| x.active)
        .count();
    eprintln!("outputs: {}, elapsed {:?}", outputs, now.elapsed());
    let mut vcd_file = std::fs::File::create("strobe.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}
