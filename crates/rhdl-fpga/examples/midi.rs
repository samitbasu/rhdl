use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::midi::{In, MidiInterface},
};

fn encode_frame(byte: u128, divisor: usize) -> Vec<bool> {
    let mut out = vec![true; 4];
    for _ in 0..divisor {
        out.push(false);
    }
    for k in 0..8 {
        let b = ((byte >> k) & 1) != 0;
        for _ in 0..divisor {
            out.push(b);
        }
    }
    for _ in 0..divisor {
        out.push(true);
    }
    out
}

fn main() -> Result<(), RHDLError> {
    let mut frame = Vec::new();
    frame.extend(encode_frame(0x90, 6)); // Note On status
    frame.extend(encode_frame(0x40, 6)); // Note 64 (E4)
    frame.extend(encode_frame(0x7F, 6)); // Velocity 127
    let mut stream_in: Vec<In> = Vec::new();
    for &rx in &frame {
        stream_in.push(In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: false,
            rx,
        });
    }
    for _ in 0..16 {
        stream_in.push(In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: true,
            rx: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = MidiInterface::<6, 4>::new(bits(6));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "midi.md", options)?;
    Ok(())
}
