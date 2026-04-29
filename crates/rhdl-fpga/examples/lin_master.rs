use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::lin_master::{In, LinMaster, FSM_TRANSITIONS},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14.
    write_fsm_diagram_as_markdown::<LinMaster<6, 8>>(FSM_TRANSITIONS, "lin_master_fsm.md")?;

    let uut = LinMaster::<6, 8>::new(bits(4), bits(52));
    let mut stream_in: Vec<In> = vec![In {
        id: bits(0x12),
        data: bits(0xA5),
        start: true,
    }];
    for _ in 0..300 {
        stream_in.push(In {
            id: bits(0),
            data: bits(0),
            start: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "lin_master.md", options)?;
    Ok(())
}
