// A Synchronous design consists of the following pieces:
//
//  Input   - a Digital type that describes the input to the design
// Output   - a Digital type that describes the output of the design
//  State   - a Digital type that describes the state of the design
//  Clock   - the clock signal for the design (may be implicit)
//  Initial - the initial state of the design
//  Update  - the update function for the design.
//  Params  - the parameters for the design (held constant)

use std::path::PathBuf;

use anyhow::Result;
use rhdl_bits::alias::b4;
use rhdl_bits::{bits, Bits};
use rhdl_core::{
    compile_design, generate_verilog, note, note_init_db, note_take, note_time,
    test_module::TestModule, Digital, DigitalFn,
};
use rhdl_core::{Synchronous, UpdateFn};
use rhdl_fpga::{make_constrained_verilog, Constraint, PinConstraint};
use rhdl_macro::{kernel, Digital};

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
    type Update = pulser_update<{ N }>;

    const INITIAL_STATE: Self::State = PulserState::<N> {
        one_shot: OneShot::<N>::INITIAL_STATE,
        strobe: Strobe::<N>::INITIAL_STATE,
    };

    const UPDATE: UpdateFn<Self> = pulser_update::<{ N }>;
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
    type Update = pulse_update;

    const INITIAL_STATE: Self::State = false;
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = pulse_update;
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
    type Update = one_shot_update<{ N }>;

    const INITIAL_STATE: Self::State = OneShotState::<{ N }> {
        counter: bits::<{ N }>(0),
        running: false,
    };
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        one_shot_update::<{ N }>;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct OneShotState<const N: usize> {
    counter: Bits<N>,
    running: bool,
}

#[kernel]
pub fn one_shot_update<const N: usize>(
    params: OneShot<N>,
    q: OneShotState<N>,
    trigger: bool,
) -> (OneShotState<N>, bool) {
    note("trigger", trigger);
    note("state", q.running);
    note("counter", q.counter);
    let mut d = q;
    if q.running {
        d.counter += 1;
    }
    if q.running && (q.counter == params.duration) {
        d.running = false;
    }
    let active = q.running;
    if trigger {
        d.running = true;
        d.counter = bits::<{ N }>(0);
    }
    note("active", active);
    (d, active)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Strobe<const N: usize> {
    pub period: Bits<N>,
}

impl<const N: usize> Synchronous for Strobe<N> {
    type Input = bool;
    type Output = bool;
    type State = StrobeState<N>;
    type Update = strobe_update<{ N }>;

    const INITIAL_STATE: Self::State = StrobeState::<{ N }> {
        count: bits::<{ N }>(0),
    };
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        strobe_update::<{ N }>;
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
    let mut q_count = state.count;
    if input {
        q_count += 1;
    }
    let active = input & (state.count == params.period);
    if active {
        q_count = bits::<{ N }>(0);
    }
    note("active", active);

    (StrobeState::<{ N }> { count: q_count }, active)
}

pub fn simulate<M: Synchronous>(
    obj: M,
    inputs: impl Iterator<Item = M::Input>,
) -> impl Iterator<Item = M::Output> {
    let mut state = M::State::default();
    note_time(0);
    let mut time = 0;
    inputs.map(move |input| {
        let (new_state, output) = M::UPDATE(obj, state, input);
        state = new_state;
        time += 1_000;
        note_time(time);
        output
    })
}

pub fn make_verilog_testbench<M: Synchronous>(
    obj: M,
    inputs: impl Iterator<Item = M::Input>,
) -> Result<TestModule> {
    // Given a synchronous object and an iterator of inputs, generate a Verilog testbench
    // that will simulate the object and print the results to the console.
    let verilog = generate_verilog(&compile_design(M::Update::kernel_fn().try_into()?)?)?;
    let module_code = format!("{}", verilog);
    let inputs = inputs.collect::<Vec<_>>();
    let outputs = simulate(obj, inputs.iter().copied()).collect::<Vec<_>>();
    let test_loop = inputs
        .iter()
        .zip(outputs.iter())
        .map(|(input, output)| {
            format!(
                "input_value = {}; #501; $display(\"0x%0h 0x%0h\", {}, output_reg); #499;",
                rhdl_core::as_verilog_literal(&input.typed_bits()),
                rhdl_core::as_verilog_literal(&output.typed_bits())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let testbench = format!(
        "

module testbench();

    reg clk;
    localparam config_value = {config};
    reg [{STATE_BITS}:0] state;
    wire [{STATE_AND_OUTPUT_BITS}:0] update_result;
    reg [{INPUT_BITS}:0] input_value;
    wire [{OUTPUT_BITS}:0] output_value;
    reg [{OUTPUT_BITS}:0] output_reg;


    {module_code}

    initial begin
        clk = 1'b0;
        forever #500 clk = ~clk;
    end

    assign update_result = {update_fn}(config_value, state, input_value);
    assign output_value = update_result[{OUTPUT_END}:{OUTPUT_START}];

    always @(posedge clk) begin
        state <= update_result[{STATE_BITS}:0];
        output_reg <= output_value;
    end

    initial begin
        #0
        state = {initial_state};
        {test_loop}
        $finish;
    end
endmodule
",
        STATE_BITS = M::State::bits() - 1,
        STATE_AND_OUTPUT_BITS = M::State::bits() + M::Output::bits() - 1,
        INPUT_BITS = M::Input::bits() - 1,
        OUTPUT_BITS = M::Output::bits() - 1,
        update_fn = verilog.name,
        config = rhdl_core::as_verilog_literal(&obj.typed_bits()),
        initial_state = rhdl_core::as_verilog_literal(&M::INITIAL_STATE.typed_bits()),
        OUTPUT_START = M::State::bits(),
        OUTPUT_END = M::State::bits() + M::Output::bits() - 1,
    );

    Ok(TestModule {
        testbench,
        num_cases: inputs.len(),
    })
}

#[test]
fn test_strobe_simulation() {
    let enable = std::iter::repeat(true).take(1_000_000);
    let strobe = Strobe::<16> { period: bits(100) };
    let now = std::time::Instant::now();
    note_init_db();
    let outputs = simulate(strobe, enable).filter(|x| *x).count();
    eprintln!("outputs: {}, elapsed {:?}", outputs, now.elapsed());
    let mut vcd_file = std::fs::File::create("strobe.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_start_pulse_simulation() {
    let input = std::iter::repeat(()).take(100);
    let pulse = StartPulse {};
    note_init_db();
    let outputs = simulate(pulse, input).filter(|x| *x).count();
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
    let outputs = simulate(one_shot, input).filter(|x| *x).count();
    let mut vcd_file = std::fs::File::create("one_shot.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn test_pulser_simulation() {
    let input = std::iter::repeat(true).take(1_000);
    let pulser = Pulser::<16> {
        one_shot: OneShot::<16> { duration: bits(20) },
        strobe: Strobe::<16> { period: bits(100) },
    };
    note_init_db();
    let outputs = simulate(pulser, input).filter(|x| *x).count();
    let mut vcd_file = std::fs::File::create("pulser.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

#[test]
fn get_pulser_verilog() -> Result<()> {
    let design = compile_design(pulser_update::<16>::kernel_fn().try_into()?)?;
    let verilog = generate_verilog(&design)?;
    eprintln!("Verilog {}", verilog);
    std::fs::write("pulser.v", format!("{}", verilog))?;
    let input = std::iter::repeat(true).take(10_000);
    let pulser = Pulser::<16> {
        one_shot: OneShot::<16> { duration: bits(10) },
        strobe: Strobe::<16> { period: bits(100) },
    };
    let tb = make_verilog_testbench(pulser, input)?;
    tb.run_iverilog()
}

// To make a blinker, we want to blink at a rate of 1 Hz. The clock is 100 MHz, so we want to
// toggle the output every 50 million clock cycles. We can use a Strobe with a period of 50
// million to do this.  We want the LED to be on for 1/5th of a second, which is 10 million
// clock cycles. We can use a OneShot with a duration of 10 million to do this.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Blinker {
    pub pulser: Pulser<26>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct BlinkerState {
    pub pulser: PulserState<26>,
}

impl Synchronous for Blinker {
    type Input = ();
    type Output = b4;
    type State = BlinkerState;
    type Update = blinker_update;

    const INITIAL_STATE: Self::State = BlinkerState {
        pulser: Pulser::<26>::INITIAL_STATE,
    };
    const UPDATE: UpdateFn<Self> = blinker_update;
}

#[kernel]
pub fn blinker_update(params: Blinker, state: BlinkerState, _input: ()) -> (BlinkerState, b4) {
    let (q_pulser, pulser_output) = pulser_update::<26>(params.pulser, state.pulser, true);
    let blinker_output = if pulser_output {
        b4(0b1111)
    } else {
        b4(0b0000)
    };
    (BlinkerState { pulser: q_pulser }, blinker_output)
}

#[test]
fn get_blinker_data_flow_graph() {
    let blinker = Blinker {
        pulser: Pulser::<26> {
            one_shot: OneShot::<26> {
                duration: bits(10_000_000),
            },
            strobe: Strobe::<26> {
                period: bits(50_000_000),
            },
        },
    };
    let graph = blinker.data_flow_graph();
    eprintln!("{}", graph);
}

#[test]
fn get_blinker_synth() -> Result<()> {
    let blinker = Blinker {
        pulser: Pulser::<26> {
            one_shot: OneShot::<26> {
                duration: bits(10_000_000),
            },
            strobe: Strobe::<26> {
                period: bits(50_000_000),
            },
        },
    };
    // Make pin constraints for the outputs
    let mut constraints = (0..4)
        .map(|i| PinConstraint {
            kind: rhdl_fpga::PinConstraintKind::Output,
            index: i,
            constraint: Constraint::Location(rhdl_fpga::bsp::alchitry::cu::LED_ARRAY_LOCATIONS[i]),
        })
        .collect::<Vec<_>>();
    constraints.push(PinConstraint {
        kind: rhdl_fpga::PinConstraintKind::Input,
        index: 0,
        constraint: Constraint::Unused,
    });
    let top = make_constrained_verilog(
        blinker,
        constraints,
        Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
    )?;
    let pcf = top.pcf()?;
    std::fs::write("blink.v", &top.module)?;
    std::fs::write("blink.pcf", &pcf)?;
    eprintln!("{}", top.module);
    rhdl_fpga::bsp::alchitry::cu::synth_yosys_nextpnr_icepack(&top, &PathBuf::from("blink"))?;
    Ok(())
}
