use rhdl::prelude::*;
use rhdl_fpga::{serial_bus::uart_rx::UartRx, doc::write_svg_as_markdown};

/// Encode a byte as a UART frame at the given divisor: idle, start, 8 LSB-first data, stop, idle.
fn encode_frame(byte: u128, divisor: usize) -> Vec<bool> {
    let mut out = vec![true; 8];
    for _ in 0..divisor {
        out.push(false);
    }
    for k in 0..8 {
        let bit = ((byte >> k) & 1) != 0;
        for _ in 0..divisor {
            out.push(bit);
        }
    }
    for _ in 0..divisor {
        out.push(true);
    }
    out
}

fn main() -> Result<(), RHDLError> {
    let divisor = 8;
    let mut frame = Vec::new();
    frame.extend(encode_frame(0xA5, divisor));
    frame.extend(encode_frame(0x42, divisor));
    frame.extend(vec![true; 8]);
    let stream = frame.into_iter().with_reset(1).clock_pos_edge(100);
    let uut = UartRx::<6>::new(bits(divisor as u128));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "uart_rx.md", options)?;
    Ok(())
}
