use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    rng::xorshift::XorShift128,
    stream::{
        filter::Filter,
        testing::{single_stage::single_stage, utils::stalling},
    },
};

#[kernel]
fn keep_even(_cr: ClockReset, t: b4) -> bool {
    !(t & bits(1)).any()
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
    let a_rng = stalling(a_rng, 0.23);
    let consume = move |data: Option<b4>| {
        if let Some(data) = data {
            // Only even values kept
            assert!(data.raw() & 1 == 0);
        }
        rand::random::<f64>() > 0.2
    };
    let filter = Filter::try_new::<keep_even>()?;
    let uut = single_stage(filter, a_rng, consume);
    // Run a few samples through
    let input = repeat_n((), 15).with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(input).collect::<Svg>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "filter.md", SvgOptions::default())?;
    Ok(())
}
