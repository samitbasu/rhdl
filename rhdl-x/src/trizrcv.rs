use crate::trizsnd::LineState;
use rhdl::prelude::*;
use rhdl_fpga::core::dff;

#[derive(PartialEq, Clone, Copy, Debug, Default, Digital)]
pub enum State {
    #[default]
    Idle,
    Write,
    Read,
}

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub bitz: BitZ<8>,
    pub state: Option<LineState>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U {
    state: dff::U<State>,
    reg: dff::U<b8>,
}

impl Default for U {
    fn default() -> Self {
        Self {
            state: dff::U::new(State::Idle),
            reg: dff::U::new(b8::default()),
        }
    }
}

impl SynchronousIO for U {
    type I = I;
    type O = BitZ<8>;
    type Kernel = trizrcv;
}

#[kernel]
pub fn trizrcv(cr: ClockReset, i: I, q: Q) -> (BitZ<8>, D) {
    let mut d = D::maybe_init();
    d.reg = q.reg;
    let mut state = q.state;
    let mut o = BitZ::<8>::default();
    trace("current_state", &state);
    match state {
        State::Idle => match i.state {
            Some(LineState::Write) => {
                state = State::Write;
                d.reg = i.bitz.value + 1;
            }
            Some(LineState::Read) => {
                state = State::Read;
            }
            None => {}
        },
        State::Write => {
            state = State::Idle;
        }
        State::Read => {
            o.mask = bits::<8>(0xff);
            o.value = q.reg;
            state = State::Idle;
        }
    }
    d.state = state;
    (o, d)
}
