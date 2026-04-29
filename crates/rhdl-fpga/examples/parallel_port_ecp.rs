use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::parallel_port_ecp::{FSM_TRANSITIONS, In, ParallelPortEcp},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<ParallelPortEcp>(FSM_TRANSITIONS, "parallel_port_ecp_fsm.md")?;

    let uut = ParallelPortEcp::default();
    let mut stream_in: Vec<In> = Vec::new();
    // Forward direction: stream a literal + a 3-byte run.
    for &b in &[0x42u8, 0xAA, 0xAA, 0xAA] {
        stream_in.push(In {
            in_data: bits(b as u128),
            in_valid: true,
            flush: false,
            periph_ack_in: false,
            d_in_rev: bits(0),
            periph_clk_in: true,
            rev_out_ready: false,
            dir_request: false,
            rev_is_count_in: false,
        });
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: false,
            periph_ack_in: false,
            d_in_rev: bits(0),
            periph_clk_in: true,
            rev_out_ready: false,
            dir_request: false,
            rev_is_count_in: false,
        });
    }
    stream_in.push(In {
        in_data: bits(0),
        in_valid: false,
        flush: true,
        periph_ack_in: false,
        d_in_rev: bits(0),
        periph_clk_in: true,
        rev_out_ready: false,
        dir_request: false,
        rev_is_count_in: false,
    });
    // Drain (alternating idle/ack to walk the forward handshake).
    for _ in 0..40 {
        for _ in 0..2 {
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                periph_ack_in: false,
                d_in_rev: bits(0),
                periph_clk_in: true,
                rev_out_ready: false,
                dir_request: false,
                rev_is_count_in: false,
            });
        }
        for _ in 0..2 {
            stream_in.push(In {
                in_data: bits(0),
                in_valid: false,
                flush: false,
                periph_ack_in: true,
                d_in_rev: bits(0),
                periph_clk_in: true,
                rev_out_ready: false,
                dir_request: false,
                rev_is_count_in: false,
            });
        }
        stream_in.push(In {
            in_data: bits(0),
            in_valid: false,
            flush: false,
            periph_ack_in: false,
            d_in_rev: bits(0),
            periph_clk_in: true,
            rev_out_ready: false,
            dir_request: false,
            rev_is_count_in: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "parallel_port_ecp.md", options)?;
    Ok(())
}
