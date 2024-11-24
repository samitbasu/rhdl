use rhdl::prelude::*;

use crate::core::dff;

use super::{ChannelMToS, ChannelSToM};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    // Register to hold the data
    sample: dff::U<T>,
    // Register to hold the valid signal
    state: dff::U<State>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
pub enum State {
    #[default]
    Idle,
    Valid,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    pub bus: ChannelSToM,
    pub to_send: Option<T>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    pub bus: ChannelMToS<T>,
    pub full: bool,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = sender_kernel<T>;
}

#[kernel]
pub fn unpack<T: Digital>(opt: Option<T>) -> (bool, T) {
    match opt {
        None => (false, T::init()),
        Some(t) => (true, t),
    }
}

#[kernel]
pub fn sender_kernel<T: Digital>(cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::init();
    let mut o = O::<T>::init();
    //  unpack the input
    let (input_valid, input_data) = unpack::<T>(i.to_send);
    // By default, the state will not change.
    d.state = q.state;
    match q.state {
        State::Idle => {
            if input_valid {
                d.state = State::Valid;
                d.sample = input_data;
            }
        }
        State::Valid => {
            if i.bus.ready {
                if input_valid {
                    d.state = State::Valid;
                    d.sample = input_data;
                } else {
                    d.state = State::Idle;
                    d.sample = T::init();
                }
            }
        }
    }
    o.bus.data = q.sample;
    o.bus.valid = q.state == State::Valid;
    o.full = q.state != State::Idle;
    // Reset logic
    if cr.reset.any() {
        o.full = false;
    }
    (o, d)
}
