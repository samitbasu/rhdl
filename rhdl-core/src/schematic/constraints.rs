use super::schematic_impl::PinPath;

pub struct Constraint {
    pub kind: ConstraintKind,
    pub help: String,
    pub name: String,
}

pub fn constraint_must_clock(pin_path: PinPath) -> Constraint {
    Constraint {
        kind: ConstraintKind::MustClock(MustClockConstraint { pin_path }),
        help: "This pin must be clocked".to_string(),
        name: "MustClock".to_string(),
    }
}

pub fn constraint_not_constant_valued(pin_path: PinPath) -> Constraint {
    Constraint {
        kind: ConstraintKind::NotConstantValued(NotConstantValuedConstraint { pin_path }),
        help: "This pin must not be constant valued".to_string(),
        name: "NotConstantValued".to_string(),
    }
}

pub fn constraint_output_synchronous(
    output: PinPath,
    clock: PinPath,
    edge: EdgeType,
) -> Constraint {
    Constraint {
        kind: ConstraintKind::OutputSynchronous(OutputSynchronousConstraint {
            output,
            clock,
            edge,
        }),
        help: "This output must be synchronous to the clock".to_string(),
        name: "OutputSynchronous".to_string(),
    }
}

pub fn constraint_input_synchronous(input: PinPath, clock: PinPath, edge: EdgeType) -> Constraint {
    Constraint {
        kind: ConstraintKind::InputSynchronous(InputSynchronousConstraint { input, clock, edge }),
        help: "This input must be synchronous to the clock".to_string(),
        name: "InputSynchronous".to_string(),
    }
}

pub enum ConstraintKind {
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
}

pub struct NotConstantValuedConstraint {
    pub pin_path: PinPath,
}

pub struct OutputSynchronousConstraint {
    pub output: PinPath,
    pub clock: PinPath,
    pub edge: EdgeType,
}

pub struct InputSynchronousConstraint {
    pub input: PinPath,
    pub clock: PinPath,
    pub edge: EdgeType,
}
