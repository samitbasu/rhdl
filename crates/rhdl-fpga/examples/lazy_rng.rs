use rhdl::prelude::*;
use rhdl_fpga::stream::{ready, testing::lazy_random::*};

fn main() -> Result<(), RHDLError> {
    let input = (0..)
        .map(|_| rand::random_bool(0.8))
        .map(|r| In { ready: ready(r) })
        .with_reset(1)
        .clock_pos_edge(100)
        .take_while(|t| t.time < 1500);
    let uut = LazyRng::default();
    let vcd = uut.run(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "lazy_rng.md",
        SvgOptions::default().with_io_filter(),
    )?;
    Ok(())
}
