use rhdl::prelude::*;
use rhdl_fpga::{
    core::rle_encoder::{FSM_TRANSITIONS, In, RleEncoder},
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<RleEncoder>(FSM_TRANSITIONS, "rle_encoder_fsm.md")?;

    let uut = RleEncoder::default();
    // Stream: a literal, a 3-byte run, another literal — the canonical demo.
    let mut stream_in: Vec<In> = Vec::new();
    for &b in &[0x42u8, 0xAA, 0xAA, 0xAA, 0xBB] {
        stream_in.push(In {
            in_data: bits(b as u128),
            in_valid: true,
            flush: false,
            out_ready: true,
        });
        for _ in 0..3 {
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                out_ready: true,
            });
        }
    }
    stream_in.push(In {
        in_data: bits(0),
        in_valid: false,
        flush: true,
        out_ready: true,
    });
    for _ in 0..6 {
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: false,
            out_ready: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "rle_encoder.md", options)?;
    Ok(())
}
