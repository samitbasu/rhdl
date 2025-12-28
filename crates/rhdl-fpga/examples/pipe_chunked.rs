use rhdl::prelude::*;
use rhdl_fpga::{
    pipe::chunked::Chunked, rng::xorshift::XorShift128, stream::testing::utils::stalling,
};

fn main() -> Result<(), RHDLError> {
    let uut = Chunked::<b4, 2, 4>::default();
    let source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
    let input = stalling(source_rng, 0.1)
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);
    let vcd = uut.run(input).collect::<Svg>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "pipe_chunked.md",
        SvgOptions::default().with_io_filter(),
    )?;
    Ok(())
}
