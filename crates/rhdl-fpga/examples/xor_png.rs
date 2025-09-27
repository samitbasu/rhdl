use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, rng::xorshift::XorShift};

fn main() -> Result<(), RHDLError> {
    let input = std::iter::repeat_n(true, 10)
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = XorShift::default();
    let vcd = uut.run(input).collect::<Vcd>();
    write_svg_as_markdown(
        vcd,
        "xor_png.md",
        SvgOptions::default().with_filter("(^top.input(.*))|(^top.outputs(.*))|(^top.reset)"),
    )?;
    Ok(())
}
