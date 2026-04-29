use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::mfm_encoder::{FSM_TRANSITIONS, In, MfmEncoder},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<MfmEncoder>(FSM_TRANSITIONS, "mfm_encoder_fsm.md")?;

    let uut = MfmEncoder::default();
    let mut stream_in: Vec<In> = vec![In {
        data: bits(0xA5),
        send: true,
    }];
    for _ in 0..32 {
        stream_in.push(In {
            data: bits(0),
            send: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "mfm_encoder.md", options)?;
    Ok(())
}
