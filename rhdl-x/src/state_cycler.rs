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

#[derive(Clone, Debug, Synchronous, SynchronousDQZ)]
pub struct U {
    state: crate::dff::U<State>,
    driver: crate::zdriver::ZDriver<8>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            state: crate::dff::U::new(State::Boot),
            driver: Default::default(),
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
    trace("current_state", &state);
    d.driver.data = bits::<8>(0);
    d.driver.mask = bits::<8>(0);
    match state {
        State::Boot => {
            o = O::Error;
            if !cr.reset.any() {
                state = State::Idle;
            }
        }
        State::Idle => {
            d.driver.mask = b8(0b1111_1111);
            d.driver.data = b8(0b1010_1010);
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
    trace("next_state", &state);
    d.state = state;
    if cr.reset.any() {
        d.state = State::Boot;
    }
    (o, d)
}
