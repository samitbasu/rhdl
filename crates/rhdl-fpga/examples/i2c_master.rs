use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::i2c_master::{I2cMaster, In, FSM_TRANSITIONS},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14.
    write_fsm_diagram_as_markdown::<I2cMaster<4>>(FSM_TRANSITIONS, "i2c_master_fsm.md")?;

    let mut stream_in: Vec<In> = vec![In {
        addr: bits(0x42),
        data: bits(0x55),
        start: true,
        sda_in: false,
    }];
    for _ in 0..160 {
        stream_in.push(In {
            addr: bits(0),
            data: bits(0),
            start: false,
            sda_in: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = I2cMaster::<4>::new(bits(2));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "i2c_master.md", options)?;
    Ok(())
}
