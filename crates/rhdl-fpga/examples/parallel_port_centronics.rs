use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::parallel_port_centronics::{
        CentronicsTimings, FSM_TRANSITIONS, In, ParallelPortCentronics,
    },
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<ParallelPortCentronics<8>>(
        FSM_TRANSITIONS,
        "parallel_port_centronics_fsm.md",
    )?;

    let timings = CentronicsTimings::<8> {
        t_setup: bits(2),
        t_strobe_low: bits(4),
        t_ack_timeout: bits(40),
    };
    let uut = ParallelPortCentronics::<8>::new(timings);
    let mut stream_in: Vec<In> = vec![In {
        data: bits(0x41), // ASCII 'A'
        send: true,
        ack_n_in: true,
        busy_in: false,
    }];
    for _ in 0..12 {
        stream_in.push(In {
            data: bits(0),
            send: false,
            ack_n_in: true,
            busy_in: false,
        });
    }
    // Device pulses ACK_n low for 2 cycles.
    for _ in 0..2 {
        stream_in.push(In {
            data: bits(0),
            send: false,
            ack_n_in: false,
            busy_in: false,
        });
    }
    for _ in 0..8 {
        stream_in.push(In {
            data: bits(0),
            send: false,
            ack_n_in: true,
            busy_in: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "parallel_port_centronics.md", options)?;
    Ok(())
}
