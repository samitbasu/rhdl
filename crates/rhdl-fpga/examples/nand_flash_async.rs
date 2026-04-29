use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::nand_flash_async::{FSM_TRANSITIONS, In, NandFlashAsync, NandOp, NandTimings},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<NandFlashAsync<8>>(FSM_TRANSITIONS, "nand_flash_async_fsm.md")?;

    let timings = NandTimings::<8> {
        t_setup: bits(2),
        t_we_low: bits(4),
        t_re_low: bits(4),
    };
    let uut = NandFlashAsync::<8>::new(timings);
    // Issue the ONFI Reset command (0xFF), then read status (0x70 cmd + read).
    let mut stream_in: Vec<In> = vec![In {
        op: NandOp::SendCommand,
        data: bits(0xFF),
        d_in: bits(0),
        start: true,
        r_b_n_in: true,
    }];
    for _ in 0..14 {
        stream_in.push(In {
            op: NandOp::SendCommand,
            data: bits(0),
            d_in: bits(0),
            start: false,
            r_b_n_in: true,
        });
    }
    stream_in.push(In {
        op: NandOp::SendCommand,
        data: bits(0x70),
        d_in: bits(0),
        start: true,
        r_b_n_in: true,
    });
    for _ in 0..14 {
        stream_in.push(In {
            op: NandOp::SendCommand,
            data: bits(0),
            d_in: bits(0),
            start: false,
            r_b_n_in: true,
        });
    }
    stream_in.push(In {
        op: NandOp::ReadData,
        data: bits(0),
        d_in: bits(0xC0), // simulated chip status: ready + write-protect-OK
        start: true,
        r_b_n_in: true,
    });
    for _ in 0..14 {
        stream_in.push(In {
            op: NandOp::ReadData,
            data: bits(0),
            d_in: bits(0xC0),
            start: false,
            r_b_n_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "nand_flash_async.md", options)?;
    Ok(())
}
