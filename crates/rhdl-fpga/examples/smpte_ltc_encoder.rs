use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::smpte_ltc_encoder::{FSM_TRANSITIONS, In, SmpteLtcEncoder},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<SmpteLtcEncoder>(FSM_TRANSITIONS, "smpte_ltc_encoder_fsm.md")?;

    let uut = SmpteLtcEncoder::default();
    let pattern = [true, false, true, false, true, true, false, false];
    let mut stream_in: Vec<In> = Vec::new();
    for &b in pattern.iter() {
        stream_in.push(In {
            bit_in: b,
            cell_tick: true,
        });
        stream_in.push(In {
            bit_in: false,
            cell_tick: false,
        });
        stream_in.push(In {
            bit_in: b,
            cell_tick: true,
        });
        stream_in.push(In {
            bit_in: false,
            cell_tick: false,
        });
    }
    stream_in.push(In {
        bit_in: false,
        cell_tick: true,
    });
    stream_in.push(In {
        bit_in: false,
        cell_tick: false,
    });
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "smpte_ltc_encoder.md", options)?;
    Ok(())
}
