use rhdl::prelude::*;

use crate::core::dff;

/// Implement a Carloni Relay station.  As described in the
/// paper "From Latency-Insensitive Design to Communication-Based
/// System-Level Design" by Carloni. Proceedings of the IEEE, 2015.
///
/// This is an implementation of the relay station as shown in Figure 4.
///
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<T: Digital> {
    // The main FF
    main_ff: dff::U<T>,
    // The aux FF
    aux_ff: dff::U<T>,
    // The void FF
    void_ff: dff::U<bool>,
    // The state FF
    state_ff: dff::U<State>,
}

// The state is either Run or Stall
#[derive(Copy, Clone, PartialEq, Debug, Digital, Default)]
pub enum State {
    #[default]
    Run,
    Stall,
}

impl<T: Digital> Default for U<T> {
    fn default() -> Self {
        Self {
            main_ff: dff::U::new(T::init()),
            aux_ff: dff::U::new(T::init()),
            void_ff: dff::U::new(true),
            state_ff: dff::U::new(State::Run),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct I<T: Digital> {
    pub data_in: T,
    pub void_in: bool,
    pub stop_in: bool,
}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct O<T: Digital> {
    pub data_out: T,
    pub void_out: bool,
    pub stop_out: bool,
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = I<T>;
    type O = O<T>;
    type Kernel = carloni_kernel<T>;
}

#[kernel]
pub fn carloni_kernel<T: Digital>(_cr: ClockReset, i: I<T>, q: Q<T>) -> (O<T>, D<T>) {
    let mut d = D::<T>::init();
    let mut o = O::<T>::init();
    // There are 4 control signals
    let mut sel = false;
    let mut main_en = false;
    let mut aux_en = false;
    let mut stop_out = false;
    // These are just renames for signals to match the diagram
    let void_out = q.void_ff;
    // Calculate the next state and update the control signals
    let will_stall = i.stop_in & (!i.void_in & !void_out);
    d.state_ff = q.state_ff;
    match q.state_ff {
        State::Run => {
            if !i.stop_in | (!i.void_in & void_out) {
                main_en = true;
            } else if will_stall {
                d.state_ff = State::Stall;
                aux_en = true;
            }
        }
        State::Stall => {
            if i.stop_in {
                stop_out = true;
            } else {
                sel = true;
                main_en = true;
                stop_out = true;
                d.state_ff = State::Run;
            }
        }
    }
    // Assemble the aux fifo
    d.aux_ff = if aux_en { i.data_in } else { q.aux_ff };
    let d_mux = if sel { q.aux_ff } else { i.data_in };
    d.main_ff = if main_en { d_mux } else { q.main_ff };
    let v_mux = if sel { false } else { i.void_in };
    d.void_ff = if main_en { v_mux } else { q.void_ff };
    o.data_out = q.main_ff;
    o.void_out = q.void_ff;
    o.stop_out = stop_out;
    (o, d)
}
