use rhdl::prelude::*;

pub mod step_1 {
    use super::*;
    // ANCHOR: step_1
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum OpCode {
        Sum(b8, b8),
        Diff(b8, b8),
        And(b8, b8),
        Or(b8, b8),
        Xor(b8, b8),
        #[default]
        Noop,
    }

    #[kernel]
    fn alu(op: OpCode) -> Option<b8> {
        trace("op", &op);
        let output = match op {
            OpCode::Sum(a, b) => Some(a + b),
            OpCode::Diff(a, b) => Some(a - b),
            OpCode::And(a, b) => Some(a & b),
            OpCode::Or(a, b) => Some(a | b),
            OpCode::Xor(a, b) => Some(a ^ b),
            OpCode::Noop => None,
        };
        trace("output", &output);
        output
    }
    // ANCHOR_END: step_1

    // ANCHOR: step_1_test
    #[test]
    fn test_alu() {
        let ops = [
            OpCode::Sum(b8::from(10), b8::from(5)),
            OpCode::Diff(b8::from(20), b8::from(8)),
            OpCode::And(b8::from(0b11001100), b8::from(0b01101111)),
            OpCode::Or(b8::from(0b11001111), b8::from(0b10111010)),
            OpCode::Xor(b8::from(0b11111100), b8::from(0b10101110)),
            OpCode::Noop,
        ];
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, op) in ops.iter().enumerate() {
            let traced = session.traced_at_time(ndx as u64 * 100, || {
                alu(*op);
            });
            trace::container::TraceContainer::record(&mut svg, &traced).unwrap()
        }
        if !std::path::PathBuf::from("trace_enums.svg").exists() {
            let mut file = std::fs::File::create("trace_enums.svg").unwrap();
            svg.finalize(&SvgOptions::default(), &mut file).unwrap();
        }
    }
    // ANCHOR_END: step_1_test
}

pub mod step_2 {
    use super::*;

    // ANCHOR: step_2_state
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum State {
        #[default]
        Idle,
        Running,
        Paused,
        Stopped,
        Broken,
    }
    // ANCHOR_END: step_2_state

    // ANCHOR: step_2_event
    #[derive(Copy, Clone, PartialEq, Digital, Default)]
    pub enum Event {
        #[default]
        Start,
        Stop,
    }
    // ANCHOR_END: step_2_event

    // ANCHOR: step_2_kernel
    #[kernel]
    fn fsm(state: State, event: Event) -> State {
        trace("state", &state);
        trace("event", &event);
        let next_state = match (state, event) {
            (State::Idle, Event::Start) => State::Running,
            (State::Running, Event::Stop) => State::Paused,
            (State::Paused, Event::Start) => State::Running,
            (State::Paused, Event::Stop) => State::Stopped,
            (State::Stopped, Event::Stop) => State::Idle,
            _ => State::Broken,
        };
        trace("next_state", &next_state);
        next_state
    }
    // ANCHOR_END: step_2_kernel

    // ANCHOR: step_2_test
    #[test]
    fn test_fsm() {
        let events = [
            Event::Start,
            Event::Stop,
            Event::Start,
            Event::Stop,
            Event::Stop,
            Event::Stop,
            Event::Stop,
        ];
        let mut state = State::Idle;
        let session = Session::default();
        let mut svg = SvgFile::default();
        for (ndx, event) in events.iter().enumerate() {
            let sample = session.traced_at_time(ndx as u64 * 100, || {
                state = fsm(state, *event);
            });
            trace::container::TraceContainer::record(&mut svg, &sample).unwrap();
        }
        let mut file = std::fs::File::create("tracing_demo_fsm.svg").unwrap();
        svg.finalize(&SvgOptions::default(), &mut file).unwrap();
    }
    // ANCHOR_END: step_2_test

    #[test]
    fn test_vcd() {
        let events = [
            Event::Start,
            Event::Stop,
            Event::Start,
            Event::Stop,
            Event::Stop,
            Event::Stop,
            Event::Stop,
        ];
        let mut state = State::Idle;
        // ANCHOR: step_2_vcd
        let session = Session::default();
        let mut vcd = VcdFile::default();
        for (ndx, event) in events.iter().enumerate() {
            let sample = session.traced_at_time(ndx as u64 * 100, || {
                state = fsm(state, *event);
            });
            trace::container::TraceContainer::record(&mut vcd, &sample).unwrap();
        }
        let mut file = std::fs::File::create("tracing_demo_fsm.vcd").unwrap();
        vcd.finalize(&VcdOptions::default(), &mut file).unwrap();
        // ANCHOR_END: step_2_vcd
    }
}
