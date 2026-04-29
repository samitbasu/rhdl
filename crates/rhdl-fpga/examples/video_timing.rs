use rhdl::prelude::*;
use rhdl_fpga::{video::video_timing::VideoTimingCore, doc::write_svg_as_markdown};

fn main() -> Result<(), RHDLError> {
    // Mini test mode for a compact trace.
    let uut = VideoTimingCore::<4, 4>::new(
        bits(10),
        bits(6),
        bits(7),
        bits(9),
        bits(4),
        bits(3),
        bits(3),
        bits(4),
    );
    let stream = std::iter::repeat_n((), 80)
        .with_reset(1)
        .clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "video_timing.md", options)?;
    Ok(())
}
