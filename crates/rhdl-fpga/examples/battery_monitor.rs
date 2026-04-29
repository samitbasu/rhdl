use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::{
        battery_monitor::{BatteryMonitor, FSM_TRANSITIONS, In},
        ti_hdq::TiHdqTimings,
    },
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<BatteryMonitor<10, 8>>(
        FSM_TRANSITIONS,
        "battery_monitor_fsm.md",
    )?;

    let timings = TiHdqTimings::<10> {
        t_break: bits(48),
        t_break_recovery: bits(8),
        t_w0: bits(20),
        t_w1: bits(4),
        t_read_low: bits(4),
        t_read_sample: bits(8),
        t_slot: bits(40),
    };
    let uut = BatteryMonitor::<10, 8>::new(timings, bits(10));
    let stream_in: Vec<In> = (0..600)
        .map(|_| In {
            reg_addr: bits(0x06),
            bus_in: true,
        })
        .collect();
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "battery_monitor.md", options)?;
    Ok(())
}
