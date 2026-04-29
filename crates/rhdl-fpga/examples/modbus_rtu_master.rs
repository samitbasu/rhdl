use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::modbus_rtu_master::{FSM_TRANSITIONS, In, ModbusRtuMaster},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<ModbusRtuMaster>(FSM_TRANSITIONS, "modbus_rtu_master_fsm.md")?;

    let uut = ModbusRtuMaster::default();
    // Canonical Modbus spec example: read 5 holding registers from slave 1
    // starting at register 0.  Expected on the wire: 01 03 00 00 00 05 85 0A.
    let mut stream_in: Vec<In> = vec![In {
        slave_addr: bits(0x01),
        start_addr: bits(0x0000),
        num_regs: bits(0x0005),
        start: true,
        tx_ready: false,
    }];
    for _ in 0..60 {
        stream_in.push(In {
            slave_addr: bits(0),
            start_addr: bits(0),
            num_regs: bits(0),
            start: false,
            tx_ready: false,
        });
    }
    for _ in 0..8 {
        stream_in.push(In {
            slave_addr: bits(0),
            start_addr: bits(0),
            num_regs: bits(0),
            start: false,
            tx_ready: true,
        });
        stream_in.push(In {
            slave_addr: bits(0),
            start_addr: bits(0),
            num_regs: bits(0),
            start: false,
            tx_ready: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "modbus_rtu_master.md", options)?;
    Ok(())
}
