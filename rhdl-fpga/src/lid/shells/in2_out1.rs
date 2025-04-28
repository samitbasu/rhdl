use crate::core::{dff, option::unpack};
use rhdl::prelude::*;

/// Implement a Carloni Shell with two input channels and a
/// single output channel.  As described in the paper "Coping with
/// Latency in SOC Design".  This is an implementation of the shell
/// as show in Figure 2.
///
#[derive(Clone, Debug, Synchronous, SynchronousDQ)]
pub struct U<W: Digital + Default, X: Digital + Default, Y: Digital + Default> {
    // The FF to hold the input data for first channel
    data_1_in_ff: dff::DFF<Option<W>>,
    // The FF to hold the input data for second channel
    data_2_in_ff: dff::DFF<Option<X>>,
    // The FF to hold the stop out flag
    ready_out_ff: dff::DFF<bool>,
    // The FF to hold the output data - combined with
    // the void_out flag
    data_out_ff: dff::DFF<Option<Y>>,
    // The FF to hold the stop in flag
    ready_in_ff: dff::DFF<bool>,
}

#[derive(PartialEq, Debug, Digital)]
pub struct I<W: Digital, X: Digital, Y: Digital> {
    pub data_1_in: Option<W>,
    pub data_2_in: Option<X>,
    pub ready_in: bool,
    pub pearl_out: Y,
}

#[derive(PartialEq, Debug, Digital)]
pub struct O<W: Digital, X: Digital, Y: Digital> {
    pub data_out: Option<Y>,
    pub ready_out: bool,
    pub pearl_strobe: bool,
    pub pearl_in: (W, X),
}

impl<W: Digital + Default, X: Digital + Default, Y: Digital + Default> SynchronousIO
    for U<W, X, Y>
{
    type I = I<W, X, Y>;
    type O = O<W, X, Y>;
    type Kernel = kernel<W, X, Y>;
}

#[kernel]
pub fn kernel<W: Digital + Default, X: Digital + Default, Y: Digital + Default>(
    _cr: ClockReset,
    i: I<W, X, Y>,
    q: Q<W, X, Y>,
) -> (O<W, X, Y>, D<W, X, Y>) {
    // Connect the data in FF
    let mut d = D::<W, X, Y>::dont_care();
    let mut o = O::<W, X, Y>::dont_care();
    d.data_1_in_ff = i.data_1_in;
    d.data_2_in_ff = i.data_2_in;
    d.ready_in_ff = i.ready_in;
    let (data_1_in_valid, data_1_in) = unpack::<W>(q.data_1_in_ff);
    let (data_2_in_valid, data_2_in) = unpack::<X>(q.data_2_in_ff);
    let stop_in = !q.ready_in_ff;
    let halt_input = stop_in || !data_1_in_valid || !data_2_in_valid;
    d.ready_out_ff = !halt_input;
    d.data_out_ff = if halt_input { None } else { Some(i.pearl_out) };
    o.pearl_in = (data_1_in, data_2_in);
    o.pearl_strobe = !halt_input;
    o.data_out = q.data_out_ff;
    o.ready_out = q.ready_out_ff;
    (o, d)
}
