use super::schematic_impl::PinPath;

pub fn constraint_must_clock(pin_path: PinPath) -> Constraint {
    Constraint::MustClock(MustClockConstraint {
        pin_path,
        help: "This pin must be clocked".to_string(),
    })
}

pub fn constraint_not_constant_valued(pin_path: PinPath) -> Constraint {
    Constraint::NotConstantValued(NotConstantValuedConstraint {
        pin_path,
        help: "This pin must not be constant valued".to_string(),
    })
}

pub fn constraint_output_synchronous(
    output: PinPath,
    clock: PinPath,
    edge: EdgeType,
) -> Constraint {
    Constraint::OutputSynchronous(OutputSynchronousConstraint {
        output,
        clock,
        edge,
        help: "This output must be synchronous to the input clock".to_string(),
    })
}

pub fn constraint_input_synchronous(input: PinPath, clock: PinPath, edge: EdgeType) -> Constraint {
    Constraint::InputSynchronous(InputSynchronousConstraint {
        input,
        clock,
        edge,
        help: "This input must be synchronous to the clock".to_string(),
    })
}

pub enum Constraint {
    MustClock(MustClockConstraint),
    NotConstantValued(NotConstantValuedConstraint),
    OutputSynchronous(OutputSynchronousConstraint),
    InputSynchronous(InputSynchronousConstraint),
}

pub enum EdgeType {
    Positive,
    Negative,
}

pub struct MustClockConstraint {
    pub pin_path: PinPath,
    pub help: String,
}

pub struct NotConstantValuedConstraint {
    pub pin_path: PinPath,
    pub help: String,
}

pub struct OutputSynchronousConstraint {
    pub output: PinPath,
    pub clock: PinPath,
    pub edge: EdgeType,
    pub help: String,
}

pub struct InputSynchronousConstraint {
    pub input: PinPath,
    pub clock: PinPath,
    pub edge: EdgeType,
    pub help: String,
}
