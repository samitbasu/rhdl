use crate::core::dff;

use super::*;

#[derive(PartialEq, Debug, Default, Digital, Clone, Copy)]
pub enum State {
    #[default]
    Idle,
    Write,
    Read,
}

#[derive(PartialEq, Debug, Digital, Clone, Copy)]
pub struct I {
    pub bitz: BitZ<8>,
    pub state: Option<LineState>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    state: dff::DFF<State>,
    reg: dff::DFF<b8>,
}

impl SynchronousIO for U {
    type I = I;
    type O = BitZ<8>;
    type Kernel = trizrcv;
}

#[kernel]
pub fn trizrcv(_cr: ClockReset, i: I, q: Q) -> (BitZ<8>, D) {
    let mut d = D::dont_care();
    d.reg = q.reg;
    let mut state = q.state;
    let mut o = BitZ::<8>::default();
    match state {
        State::Idle => {
            if let Some(i_state) = i.state {
                match i_state {
                    LineState::Write => {
                        state = State::Write;
                        d.reg = i.bitz.value;
                    }
                    LineState::Read => {
                        state = State::Read;
                    }
                }
            }
        }
        State::Write => {
            state = State::Idle;
        }
        State::Read => {
            o.mask = b8(0xff);
            o.value = q.reg;
            state = State::Idle;
        }
    }
    d.state = state;
    (o, d)
}
