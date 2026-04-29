use rhdl::prelude::*;
use rhdl_fpga::{
    core::rle_decoder::{FSM_TRANSITIONS, In, RleDecoder},
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<RleDecoder>(FSM_TRANSITIONS, "rle_decoder_fsm.md")?;

    let uut = RleDecoder::default();
    let mut stream_in: Vec<In> = Vec::new();
    // Beat sequence: literal 0x42, run-of-3 0xAA, literal 0xBB.
    for &(b, is_count) in &[(0x42u8, false), (2u8, true), (0xAA, false), (0xBB, false)] {
        stream_in.push(In {
            in_data: bits(b as u128),
            in_is_count: is_count,
            in_valid: true,
            out_ready: true,
        });
        for _ in 0..3 {
            stream_in.push(In {
                in_data: bits(0),
                in_is_count: false,
                in_valid: false,
                out_ready: true,
            });
        }
    }
    for _ in 0..16 {
        stream_in.push(In {
            in_data: bits(0),
            in_is_count: false,
            in_valid: false,
            out_ready: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "rle_decoder.md", options)?;
    Ok(())
}
