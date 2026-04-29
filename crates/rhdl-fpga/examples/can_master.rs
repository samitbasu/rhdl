use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::can_master::{CanMaster, In, FSM_TRANSITIONS},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14
    // for every #[derive(FsmWidget)] widget.
    write_fsm_diagram_as_markdown::<CanMaster<5>>(FSM_TRANSITIONS, "can_master_fsm.md")?;

    let uut = CanMaster::<5>::new(bits(4));
    let mut stream_in: Vec<In> = vec![In {
        id: bits(0x123),
        dlc: bits(1),
        data: bits(0xA5_00_00_00_00_00_00_00),
        start: true,
    }];
    for _ in 0..400 {
        stream_in.push(In {
            id: bits(0),
            dlc: bits(0),
            data: bits(0),
            start: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "can_master.md", options)?;
    Ok(())
}
