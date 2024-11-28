use rhdl::prelude::*;
use rhdl_fpga::core::dff;

#[derive(PartialEq, Clone, Copy, Debug, Default, Digital)]
pub enum State {
    #[default]
    Idle,
    Write,
    ReadReq,
    ReadRcv,
    ValueEmit,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Digital)]
pub enum Cmd {
    Write(b8),
    #[default]
    Read,
}

#[derive(PartialEq, Clone, Copy, Debug, Digital, Default)]
pub enum LineState {
    Write,
    #[default]
    Read,
}

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub bitz: BitZ<8>,
    pub cmd: Option<Cmd>,
}

#[derive(PartialEq, Clone, Copy, Debug, Digital, Default)]
pub struct O {
    pub bitz: BitZ<8>,
    pub control: Option<LineState>,
    pub data: Option<b8>,
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
    type O = O;
    type Kernel = trizsnd;
}

#[kernel]
pub fn trizsnd(cr: ClockReset, i: I, q: Q) -> (O, D) {
    let mut d = D::maybe_init();
    d.reg = q.reg;
    let mut state = q.state;
    let mut o = O::default();
    trace("input", &i);
    trace("current_state", &state);
    if state == State::Write {
        o.bitz.value = q.reg;
        o.bitz.mask = bits::<8>(0xff);
        o.control = Some(LineState::Write);
    }
    if state == State::ReadReq {
        o.control = Some(LineState::Read);
    }
    if state == State::ReadRcv {
        d.reg = i.bitz.value;
    }
    if state == State::ValueEmit {
        o.data = Some(q.reg);
    }
    match state {
        State::Idle => match i.cmd {
            Some(Cmd::Write(data)) => {
                state = State::Write;
                d.reg = data;
            }
            Some(Cmd::Read) => {
                state = State::ReadReq;
            }
            None => {}
        },
        State::Write => {
            state = State::Idle;
        }
        State::ReadReq => {
            state = State::ReadRcv;
        }
        State::ReadRcv => {
            state = State::ValueEmit;
        }
        State::ValueEmit => {
            state = State::Idle;
        }
    }
    o.bitz.mask |= i.bitz.mask;
    o.bitz.value |= i.bitz.value & i.bitz.mask;
    d.state = state;
    trace("output", &o);
    (o, d)
}
