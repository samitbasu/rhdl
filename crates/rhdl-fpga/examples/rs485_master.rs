use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::rs485_master::{FSM_TRANSITIONS, In, Rs485Master},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<Rs485Master<6, 4, 8>>(
        FSM_TRANSITIONS,
        "rs485_master_fsm.md",
    )?;

    let divisor = 6u128;
    let uut = Rs485Master::<6, 4, 8>::new(bits(divisor), bits(20));
    let mut stream_in: Vec<In> = vec![In {
        tx_data: bits(0xA5),
        tx_push: true,
        rx_pop: false,
        rx: true,
    }];
    for _ in 0..(15 * divisor as usize + 60) {
        stream_in.push(In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: false,
            rx: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "rs485_master.md", options)?;
    Ok(())
}
