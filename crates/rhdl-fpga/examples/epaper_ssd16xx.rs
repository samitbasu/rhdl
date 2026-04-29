use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::epaper_ssd16xx::{EpaperSsd16xx, In},
};

fn main() -> Result<(), RHDLError> {
    let uut = EpaperSsd16xx::<4>::default();
    // Send a command byte while controller is idle, then keep busy_n
    // low for a while (simulating refresh) and a final ready cycle.
    let mut stream_in: Vec<In> = vec![In {
        data: bits(0x22), // SSD1619 DISPLAY_UPDATE_CONTROL_2 command
        is_command: true,
        send: true,
        busy_n: true,
        reset_n_in: true,
    }];
    for _ in 0..40 {
        stream_in.push(In {
            data: bits(0),
            is_command: false,
            send: false,
            busy_n: true,
            reset_n_in: true,
        });
    }
    // Simulate refresh: BUSY# goes low for 80 cycles.
    for _ in 0..80 {
        stream_in.push(In {
            data: bits(0),
            is_command: false,
            send: false,
            busy_n: false,
            reset_n_in: true,
        });
    }
    for _ in 0..16 {
        stream_in.push(In {
            data: bits(0),
            is_command: false,
            send: false,
            busy_n: true,
            reset_n_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "epaper_ssd16xx.md", options)?;
    Ok(())
}
