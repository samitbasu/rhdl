use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::ti_hdq::{FSM_TRANSITIONS, In, TiHdqMaster, TiHdqOp, TiHdqTimings},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<TiHdqMaster<10>>(FSM_TRANSITIONS, "ti_hdq_fsm.md")?;

    let timings = TiHdqTimings::<10> {
        t_break: bits(48),
        t_break_recovery: bits(8),
        t_w0: bits(20),
        t_w1: bits(4),
        t_read_low: bits(4),
        t_read_sample: bits(8),
        t_slot: bits(40),
    };
    let uut = TiHdqMaster::<10>::new(timings);
    let mut stream_in: Vec<In> = vec![In {
        op: TiHdqOp::WriteByte,
        data: bits(0xA5),
        start: true,
        bus_in: true,
    }];
    for _ in 0..600 {
        stream_in.push(In {
            op: TiHdqOp::WriteByte,
            data: bits(0),
            start: false,
            bus_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ti_hdq.md", options)?;
    Ok(())
}
