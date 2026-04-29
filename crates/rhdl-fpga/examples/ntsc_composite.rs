use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    video::ntsc_composite::{In, NtscComposite},
};

fn main() -> Result<(), RHDLError> {
    let uut = NtscComposite::<6, 4>::new(
        bits(32), // h_total
        bits(20), // h_active_end
        bits(24), // h_sync_start
        bits(28), // h_sync_end
        bits(6),  // v_total
        bits(4),  // v_active_end
        bits(5),  // v_sync_start
        bits(6),  // v_sync_end
    );
    let stream = std::iter::repeat_n(
        In {
            pic_sample: bits(2),
        },
        256,
    )
    .with_reset(1)
    .clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ntsc_composite.md", options)?;
    Ok(())
}
