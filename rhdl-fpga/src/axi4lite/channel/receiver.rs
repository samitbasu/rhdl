use rhdl::prelude::*;

use crate::core::dff;

use super::{ChannelMToS, ChannelSToM};

#[derive(Clone, Debug, Synchronous, SynchronousDQ, Default)]
pub struct U<T: Digital> {
    // Register to hold the data
    sample: dff::U<T>,
    // Register to hold the state
    state: dff::U<State>,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
pub enum State {
    #[default]
    Idle,
    Valid,
    Wait,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct I<T: Digital> {
    // Connection to the bus
    pub bus: ChannelMToS<T>,
    // Signal to accept the data.
    pub next: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Digital)]
pub struct O<T: Digital> {
    // Data from the bus - None if there is no data
    pub data: Option<T>,
    // Output connection to the bus
    pub bus: ChannelSToM,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = receiver_kernel<T>;
}

#[kernel]
pub fn pack<T: Digital>(valid: bool, data: T) -> Option<T> {
    if valid {
        Some(data)
    } else {
        None
    }
}

#[kernel]
pub fn receiver_kernel<T: Digital>(cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::init();
    let mut o = O::<T>::init();
    d.sample = q.sample;
    d.state = q.state;
    o.bus.ready = false;
    match q.state {
        State::Idle => {
            if i.bus.valid {
                d.state = State::Valid;
                d.sample = i.bus.data;
            }
        }
        State::Valid => {
            if i.next {
                d.state = State::Wait;
                d.sample = T::init();
                o.bus.ready = true;
            }
        }
        State::Wait => {
            if !i.bus.valid {
                d.state = State::Idle;
            }
        }
    }
    o.data = pack::<T>(q.state == State::Valid, q.sample);
    (o, d)
}
