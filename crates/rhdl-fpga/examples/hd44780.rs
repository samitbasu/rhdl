use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::hd44780::{FSM_TRANSITIONS, Hd44780, In},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14.
    write_fsm_diagram_as_markdown::<Hd44780<10>>(FSM_TRANSITIONS, "hd44780_fsm.md")?;

    // Compact timings for the trace: 4 cycles per strobe-half, 20 cycles busy.
    let uut = Hd44780::<10>::new(bits(4), bits(20));
    let mut stream_in: Vec<In> = vec![In {
        data: bits(0x48), // ASCII 'H'
        rs_in: true,      // data, not command
        send: true,
    }];
    for _ in 0..120 {
        stream_in.push(In {
            data: bits(0),
            rs_in: false,
            send: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "hd44780.md", options)?;
    Ok(())
}
