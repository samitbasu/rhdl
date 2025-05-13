use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    core::slice::lsbs,
    pipe::{
        map::MapPipe,
        testing::{single_stage::single_stage, utils::stalling},
    },
    rng::xorshift::XorShift128,
};

#[kernel]
fn map_item(_cr: ClockReset, t: b4) -> b2 {
    lsbs::<U2, U4>(t)
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
    let mut b_rng = a_rng.clone();
    let a_rng = stalling(a_rng, 0.23);
    let consume = move |data: Option<b2>| {
        if let Some(data) = data {
            let orig = b_rng.next().unwrap();
            let orig_lsb = lsbs::<U2, U4>(orig);
            assert_eq!(data, orig_lsb);
        }
        rand::random::<f64>() > 0.2
    };
    let map = MapPipe::try_new::<map_item>()?;
    let uut = single_stage(map, a_rng, consume);
    // Run a few samples through
    let input = repeat_n((), 15).stream_after_reset(1).clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "map.md", SvgOptions::default())?;
    Ok(())
}
