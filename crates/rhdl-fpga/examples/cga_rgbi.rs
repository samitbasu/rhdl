use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, video::cga_rgbi::CgaRgbi};

fn main() -> Result<(), RHDLError> {
    // Mini-mode for the example trace: a 64×8 frame so the full pattern
    // fits in a few hundred FPGA cycles and is visually inspectable.
    let uut = CgaRgbi::<7, 4>::new(
        bits(64), // h_total (HW=7 holds values up to 127)
        bits(63), // h_active_end (one cycle of blanking before sync)
        bits(56), // h_sync_start
        bits(60), // h_sync_end
        bits(8),  // v_total
        bits(6),  // v_active_end
        bits(7),  // v_sync_start
        bits(8),  // v_sync_end
    );
    // Run for one full frame (64 * 8 = 512 cycles).
    let stream = std::iter::repeat_n((), 600)
        .with_reset(1)
        .clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "cga_rgbi.md", options)?;
    Ok(())
}
