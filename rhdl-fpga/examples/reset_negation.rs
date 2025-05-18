use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, reset::negation::ResetNegation};

fn main() -> Result<(), RHDLError> {
    let input = (0..15)
        .map(|_| rand::random::<bool>())
        .map(|b| signal(reset_n(b)))
        .without_reset()
        .clock_pos_edge(100)
        .map(|t| t.map(|x| x.1));
    let uut = ResetNegation::<Red>::default();
    let vcd = uut.run(input)?.collect::<Vcd>();
    write_svg_as_markdown(vcd, "reset_negation.md", SvgOptions::default())?;
    Ok(())
}
