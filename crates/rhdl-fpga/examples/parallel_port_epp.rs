use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::parallel_port_epp::{EppOp, FSM_TRANSITIONS, In, ParallelPortEpp},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<ParallelPortEpp>(FSM_TRANSITIONS, "parallel_port_epp_fsm.md")?;

    let uut = ParallelPortEpp::default();
    let mut stream_in: Vec<In> = vec![In {
        op: EppOp::AddrWrite,
        data: bits(0x42),
        d_in: bits(0),
        start: true,
        wait_n_in: true,
    }];
    for _ in 0..3 {
        stream_in.push(In {
            op: EppOp::AddrWrite,
            data: bits(0x42),
            d_in: bits(0),
            start: false,
            wait_n_in: true,
        });
    }
    for _ in 0..3 {
        stream_in.push(In {
            op: EppOp::AddrWrite,
            data: bits(0x42),
            d_in: bits(0),
            start: false,
            wait_n_in: false,
        });
    }
    for _ in 0..10 {
        stream_in.push(In {
            op: EppOp::AddrWrite,
            data: bits(0),
            d_in: bits(0),
            start: false,
            wait_n_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "parallel_port_epp.md", options)?;
    Ok(())
}
