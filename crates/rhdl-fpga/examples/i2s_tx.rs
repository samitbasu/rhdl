use rhdl::prelude::*;
use rhdl_fpga::{
    audio::i2s_tx::{FSM_TRANSITIONS, I2sTx, In},
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<I2sTx>(FSM_TRANSITIONS, "i2s_tx_fsm.md")?;

    let uut = I2sTx::default();
    let mut stream_in: Vec<In> = vec![In {
        left_sample: signed(0x1234),
        right_sample: signed(0x5678),
        bclk_tick: false,
        sample_load: true,
    }];
    stream_in.push(In {
        left_sample: signed(0),
        right_sample: signed(0),
        bclk_tick: false,
        sample_load: false,
    });
    for _ in 0..80 {
        stream_in.push(In {
            left_sample: signed(0),
            right_sample: signed(0),
            bclk_tick: true,
            sample_load: false,
        });
        stream_in.push(In {
            left_sample: signed(0),
            right_sample: signed(0),
            bclk_tick: false,
            sample_load: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "i2s_tx.md", options)?;
    Ok(())
}
