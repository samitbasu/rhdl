use rhdl::prelude::*;
use rhdl_fpga::{
    audio::dtmf_generator::{DtmfGenerator, In},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    let uut = DtmfGenerator::<8>::default();
    // Smaller-than-real phase increments so the trace shows several cycles.
    let mut stream_in: Vec<In<8>> = Vec::new();
    for _ in 0..128 {
        stream_in.push(In {
            phase_inc_row: bits(8),
            phase_inc_col: bits(13),
            enable: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "dtmf_generator.md", options)?;
    Ok(())
}
