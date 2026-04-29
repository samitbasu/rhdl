use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, video::mipi_dpi::{In, MipiDpi}};

fn main() -> Result<(), RHDLError> {
    let uut = MipiDpi::<7, 4>::new(
        bits(64),
        bits(40),
        bits(48),
        bits(56),
        bits(8),
        bits(4),
        bits(6),
        bits(7),
    );
    let stream = std::iter::repeat_n(
        In {
            r_in: bits(0xFF),
            g_in: bits(0x80),
            b_in: bits(0x40),
        },
        512,
    )
    .with_reset(1)
    .clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "mipi_dpi.md", options)?;
    Ok(())
}
