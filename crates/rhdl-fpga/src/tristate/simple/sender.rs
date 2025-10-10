use crate::core::dff;

use super::*;

#[derive(PartialEq, Debug, Default, Digital, Clone)]
pub enum Cmd {
    Write(b8),
    #[default]
    Read,
}

#[derive(PartialEq, Debug, Default, Digital, Clone)]
pub enum State {
    #[default]
    Idle,
    Write,
    ReadReq,
    ReadRcv,
    ValueEmit,
}

// The input struct includes the tristate bus
#[derive(PartialEq, Debug, Digital, Clone)]
pub struct I {
    pub bitz: BitZ<U8>,
    pub cmd: Option<Cmd>,
}

#[derive(PartialEq, Debug, Digital, Clone, Default)]
pub struct O {
    pub bitz: BitZ<U8>,
    pub control: Option<LineState>,
    pub data: Option<b8>,
}

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U {
    state: dff::DFF<State>,
    reg: dff::DFF<b8>,
}

impl SynchronousIO for U {
    type I = I;
    type O = O;
    type Kernel = trizsnd;
}

#[kernel]
pub fn trizsnd(_cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::dont_care();
    // Latching behavior
    d.reg = q.reg;
    let mut state = q.state;
    let mut o = O::default();
    // Set outputs based on the current state
    match state {
        State::Idle => {
            if let Some(cmd) = i.cmd {
                match cmd {
                    Cmd::Write(data) => {
                        state = State::Write;
                        d.reg = data;
                    }
                    Cmd::Read => {
                        state = State::ReadReq;
                    }
                }
            }
        }
        State::Write => {
            o.bitz.value = q.reg;
            o.bitz.mask = b8(0xff);
            o.control = Some(LineState::Write);
            state = State::Idle;
        }
        State::ReadReq => {
            o.control = Some(LineState::Read);
            state = State::ReadRcv;
        }
        State::ReadRcv => {
            d.reg = i.bitz.value;
            state = State::ValueEmit;
        }
        State::ValueEmit => {
            state = State::Idle;
            o.data = Some(q.reg);
        }
    }
    o.bitz.mask |= i.bitz.mask;
    o.bitz.value |= i.bitz.value & i.bitz.mask;
    d.state = state;
    (o, d)
}
