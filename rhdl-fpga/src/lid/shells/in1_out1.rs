use rhdl::prelude::*;

use crate::core::{dff, option::unpack};

/// Implement a Carloni Shell with a single input channel and a
/// single output channel.  As described in the paper "Coping with
/// Latency in SOC Design".  This is an implementation of the shell
/// as shown in Figure 2.
///
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<
    X: Digital + Default, // The input type
    Y: Digital + Default,
> {
    // The FF to hold the input data - combined with
    // the void_in flag
    data_in_ff: dff::U<Option<X>>,
    // The FF to hold the stop out flag
    ready_out_ff: dff::U<bool>,
    // The FF to hold the output data - combined with
    // the void_out flag
    data_out_ff: dff::U<Option<Y>>,
    // The FF to hold the stop in flag
    ready_in_ff: dff::U<bool>,
}

#[derive(Debug, Digital)]
pub struct I<X: Digital, Y: Digital> {
    pub data_in: Option<X>,
    pub ready_in: bool,
    pub pearl_out: Y,
}

#[derive(Debug, Digital)]
pub struct O<X: Digital, Y: Digital> {
    pub data_out: Option<Y>,
    pub ready_out: bool,
    pub pearl_strobe: bool,
    pub pearl_in: X,
}

impl<X: Digital + Default, Y: Digital + Default> SynchronousIO for U<X, Y> {
    type I = I<X, Y>;
    type O = O<X, Y>;
    type Kernel = kernel<X, Y>;
}

#[kernel]
pub fn kernel<X: Digital + Default, Y: Digital + Default>(
    _cr: ClockReset,
    i: I<X, Y>,
    q: Q<X, Y>,
) -> (O<X, Y>, D<X, Y>) {
    // Connect the data in FF
    let mut d = D::<X, Y>::dont_care();
    let mut o = O::<X, Y>::dont_care();
    d.data_in_ff = i.data_in;
    d.ready_in_ff = i.ready_in;
    let (data_in_valid, data_in) = unpack::<X>(q.data_in_ff);
    let stop_in = !q.ready_in_ff;
    let halt_input = stop_in || !data_in_valid;
    d.ready_out_ff = !halt_input;
    d.data_out_ff = if halt_input { None } else { Some(i.pearl_out) };
    o.pearl_in = data_in;
    o.pearl_strobe = !halt_input;
    o.data_out = q.data_out_ff;
    o.ready_out = q.ready_out_ff;
    (o, d)
}
