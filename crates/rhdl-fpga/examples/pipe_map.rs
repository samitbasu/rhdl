use rhdl::prelude::*;
use rhdl_fpga::{
    core::slice::lsbs, pipe::map::Map, rng::xorshift::XorShift128, stream::testing::utils::stalling,
};

#[kernel]
fn map_item(_cr: ClockReset, t: b4) -> b2 {
    lsbs::<2, 4>(t)
}

fn main() -> Result<(), RHDLError> {
    let input = XorShift128::default().map(|x| b4(x as u128 & 0xF));
    let uut = Map::try_new::<map_item>()?;
    let input = stalling(input, 0.23);
    let input = input
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);
    let vcd = uut.run(input).collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "pipe_map.md", SvgOptions::default())?;
    Ok(())
}
