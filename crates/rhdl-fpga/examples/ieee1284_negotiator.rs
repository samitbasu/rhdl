use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::ieee1284_negotiator::{FSM_TRANSITIONS, Ieee1284Negotiator, In, NegTimings},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<Ieee1284Negotiator<8>>(
        FSM_TRANSITIONS,
        "ieee1284_negotiator_fsm.md",
    )?;

    let timings = NegTimings::<8> {
        t_strobe_low: bits(4),
        t_response_timeout: bits(40),
    };
    let uut = Ieee1284Negotiator::<8>::new(timings);
    // Demo: full successful negotiation into EPP mode.
    let mut stream_in: Vec<In> = vec![In {
        mode_byte: bits(0xC0),
        d_in_eli: bits(0),
        start: true,
        terminate: false,
        n_ack_in: true,
        pe_in: true,
        select_in: false,
        n_fault_in: true,
    }];
    // 5 cycles of waiting (Setup → WaitDeviceReady).
    for _ in 0..5 {
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        });
    }
    // Device ready (PE↓ + Select↑ + nAck↓).
    for _ in 0..3 {
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: false,
            pe_in: false,
            select_in: true,
            n_fault_in: false,
        });
    }
    for _ in 0..5 {
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: false,
            pe_in: false,
            select_in: true,
            n_fault_in: false,
        });
    }
    // Device acknowledges (nAck↑ + PE↑ + Select↑ for "supported").
    for _ in 0..6 {
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: true,
            n_fault_in: true,
        });
    }
    for _ in 0..30 {
        stream_in.push(In {
            mode_byte: bits(0),
            d_in_eli: bits(0),
            start: false,
            terminate: false,
            n_ack_in: true,
            pe_in: true,
            select_in: false,
            n_fault_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ieee1284_negotiator.md", options)?;
    Ok(())
}
