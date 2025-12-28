use rhdl::prelude::*;
use rhdl_fpga::{
    pipe::filter::Filter, rng::xorshift::XorShift128, stream::testing::utils::stalling,
};

#[kernel]
fn keep_even(_cr: ClockReset, t: b4) -> bool {
    !(t & bits(1)).any()
}

fn main() -> Result<(), RHDLError> {
    let input = XorShift128::default().map(|x| b4((x & 0xF) as u128));
    let input = stalling(input, 0.23);
    let uut = Filter::try_new::<keep_even>()?;
    let input = input
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);
    let vcd = uut.run(input).collect::<Svg>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "pipe_filter.md", SvgOptions::default())?;
    Ok(())
}
