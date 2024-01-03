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
use rhdl_core::{
    compile_design, generate_verilog, note, note_init_db, note_take, note_time, Digital, DigitalFn,
};
use rhdl_macro::{kernel, Digital};

pub trait Synchronous: Digital {
    type Input: Digital;
    type Output: Digital;
    type State: Digital;

    const INITIAL_STATE: Self::State;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Pulser<const N: usize> {
    pub one_shot: OneShot<N>,
    pub strobe: Strobe<N>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct PulserState<const N: usize> {
    pub one_shot: OneShotState<N>,
    pub strobe: StrobeState<N>,
}

impl<const N: usize> Synchronous for Pulser<N> {
    type Input = bool;
    type Output = bool;
    type State = PulserState<N>;

    const INITIAL_STATE: Self::State = PulserState::<N> {
        one_shot: OneShot::<N>::INITIAL_STATE,
        strobe: Strobe::<N>::INITIAL_STATE,
    };
}

#[kernel]
pub fn pulser_update<const N: usize>(
    params: Pulser<N>,
    state: PulserState<N>,
    input: bool,
) -> (PulserState<N>, bool) {
    let (q_strobe, strobe_output) = strobe_update::<{ N }>(params.strobe, state.strobe, input);
    let (q_one_shot, one_shot_output) =
        one_shot_update::<{ N }>(params.one_shot, state.one_shot, strobe_output);
    (
        PulserState::<{ N }> {
            one_shot: q_one_shot,
            strobe: q_strobe,
        },
        one_shot_output,
    )
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StartPulse {}

impl Synchronous for StartPulse {
    type Input = ();
    type Output = bool;
    type State = bool;

    const INITIAL_STATE: Self::State = false;
}

#[kernel]
pub fn pulse_update(_params: StartPulse, state: bool, _input: ()) -> (bool, bool) {
    note("state", state);
    note("output", !state);
    (true, !state)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct OneShot<const N: usize> {
    pub duration: Bits<N>,
}

impl<const N: usize> Synchronous for OneShot<N> {
    type Input = bool;
    type Output = bool;
    type State = OneShotState<N>;

    const INITIAL_STATE: Self::State = OneShotState::<{ N }> {
        count: bits::<{ N }>(0),
        active: false,
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct OneShotState<const N: usize> {
    count: Bits<N>,
    active: bool,
}

#[kernel]
pub fn one_shot_update<const N: usize>(
    params: OneShot<N>,
    state: OneShotState<N>,
    input: bool,
) -> (OneShotState<N>, bool) {
    let mut q_state = state;

    if input {
        q_state.active = true;
        q_state.count = bits::<{ N }>(0);
    }

    if state.active {
        q_state.count += 1;
        if q_state.count == params.duration {
            q_state.active = false;
        }
    }

    note("enable", input);
    note("q_state", q_state);
    let output = state.active;

    (q_state, output)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Strobe<const N: usize> {
    pub period: Bits<N>,
}

impl<const N: usize> Synchronous for Strobe<N> {
    type Input = bool;
    type Output = bool;
    type State = StrobeState<N>;

    const INITIAL_STATE: Self::State = StrobeState::<{ N }> {
        count: bits::<{ N }>(0),
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StrobeState<const N: usize> {
    count: Bits<N>,
}

#[kernel]
pub fn strobe_update<const N: usize>(
    params: Strobe<N>,
    state: StrobeState<N>,
    input: bool,
) -> (StrobeState<N>, bool) {
    let mut count = state.count;
    let mut active = false;

    if input {
        count += 1;
        if count == params.period {
            count = bits::<{ N }>(0);
            active = true;
        }
    }
    note("active", active);

    (StrobeState::<{ N }> { count }, active)
}

pub fn sim<F, I, O, S, P>(
    obj: P,
    inputs: impl Iterator<Item = I>,
    update: F,
) -> impl Iterator<Item = O>
where
    F: Fn(P, S, I) -> (S, O),
    P: Digital,
    I: Digital,
    O: Digital,
    S: Digital,
{
    let mut state = S::default();
    note_time(0);
    let mut time = 0;
    inputs.map(move |input| {
        let (new_state, output) = update(obj, state, input);
        state = new_state;
        time += 1_000;
        note_time(time);
        output
    })
}

pub fn simulate<M: Synchronous, F>(
    obj: M,
    inputs: impl Iterator<Item = M::Input>,
    update: F,
) -> impl Iterator<Item = M::Output>
where
    F: Fn(M, M::State, M::Input) -> (M::State, M::Output),
{
    let mut state = M::State::default();
    note_time(0);
    let mut time = 0;
    inputs.map(move |input| {
        let (new_state, output) = update(obj, state, input);
        state = new_state;
        time += 1_000;
        note_time(time);
        output
    })
}

#[test]
fn test_strobe_simulation() {
    let enable = std::iter::repeat(true).take(1_000_000);
    let strobe = Strobe::<16> { period: bits(100) };
    let now = std::time::Instant::now();
    note_init_db();
    let outputs = sim(strobe, enable, strobe_update).filter(|x| *x).count();
    eprintln!("outputs: {}, elapsed {:?}", outputs, now.elapsed());
    let mut vcd_file = std::fs::File::create("strobe.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_start_pulse_simulation() {
    let input = std::iter::repeat(()).take(100);
    let pulse = StartPulse {};
    note_init_db();
    let outputs = sim(pulse, input, pulse_update).filter(|x| *x).count();
    assert_eq!(outputs, 1);
    let mut vcd_file = std::fs::File::create("start_pulse.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_one_shot_simulation() {
    let input = std::iter::once(true)
        .chain(std::iter::repeat(false).take(100))
        .cycle()
        .take(1000);
    let one_shot = OneShot::<16> { duration: bits(10) };
    note_init_db();
    let outputs = sim(one_shot, input, one_shot_update).filter(|x| *x).count();
    let mut vcd_file = std::fs::File::create("one_shot.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_pulser_simulation() {
    let input = std::iter::repeat(true).take(1_000);
    let pulser = Pulser::<16> {
        one_shot: OneShot::<16> { duration: bits(10) },
        strobe: Strobe::<16> { period: bits(100) },
    };
    note_init_db();
    let outputs = sim(pulser, input, pulser_update).filter(|x| *x).count();
    let mut vcd_file = std::fs::File::create("pulser.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn get_pulser_verilog() -> Result<()> {
    let design = compile_design(pulser_update::<16>::kernel_fn().try_into()?)?;
    let verilog = generate_verilog(&design)?;
    eprintln!("Verilog {}", verilog);
    Ok(())
}
