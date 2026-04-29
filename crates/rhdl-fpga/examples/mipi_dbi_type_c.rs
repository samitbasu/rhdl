use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::mipi_dbi_type_c::{In, MipiDbiTypeC},
};

fn main() -> Result<(), RHDLError> {
    let uut = MipiDbiTypeC::<4>::default();
    // Send DISPLAY_ON command (0x29), then a data byte 0xA5.
    let mut stream_in: Vec<In> = vec![
        In {
            data: bits(0x29),
            is_command: true,
            send: true,
        },
    ];
    for _ in 0..40 {
        stream_in.push(In {
            data: bits(0),
            is_command: false,
            send: false,
        });
    }
    stream_in.push(In {
        data: bits(0xA5),
        is_command: false,
        send: true,
    });
    for _ in 0..40 {
        stream_in.push(In {
            data: bits(0),
            is_command: false,
            send: false,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "mipi_dbi_type_c.md", options)?;
    Ok(())
}
