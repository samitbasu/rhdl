use rhdl::prelude::*;

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Digital)]
pub enum State {
    #[default]
    Boot,
    Idle,
    Run,
    Done,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Digital)]
pub enum O {
    #[default]
    Idle,
    Busy,
    Done,
    Error,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
#[rhdl(auto_dq)]
pub struct U {
    state: crate::dff::U<State>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            state: crate::dff::U::new(State::Boot),
        }
    }
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = state_cycler;
}

#[kernel]
pub fn state_cycler(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::init();
    let mut o = O::default();
    let mut state = q.state;
    note("current_state", state);
    match state {
        State::Boot => {
            o = O::Error;
            if !cr.reset.any() {
                state = State::Idle;
            }
        }
        State::Idle => {
            o = O::Idle;
            if i.enable {
                state = State::Run;
            }
        }
        State::Run => {
            o = O::Busy;
            if !i.enable {
                state = State::Done;
            }
        }
        State::Done => {
            o = O::Done;
            state = State::Idle;
        }
    }
    note("next_state", state);
    d.state = state;
    if cr.reset.any() {
        d.state = State::Boot;
    }
    (o, d)
}
