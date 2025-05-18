use rhdl::prelude::*;
use rhdl_fpga::{core::counter::Counter, doc::write_svg_as_markdown};

fn main() -> Result<(), RHDLError> {
    let input = (0..)
        .map(|_| rand::random::<bool>())
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = Counter::<U4>::default();
    let vcd = uut
        .run(input)?
        .take_while(|t| t.time < 1000)
        .collect::<Vcd>();
    let options = SvgOptions::default();
    write_svg_as_markdown(vcd, "counter.md", options)?;
    Ok(())
}
