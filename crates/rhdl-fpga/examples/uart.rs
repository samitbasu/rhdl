use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::uart::{In, Uart},
    doc::write_svg_as_markdown,
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
    let frame = encode_frame(0xA5, 6);
    let mut stream_in: Vec<In> = Vec::new();
    for &rx_bit in &frame {
        stream_in.push(In {
            tx_data: bits(0),
            tx_push: false,
            rx_pop: false,
            rx: rx_bit,
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
    let uut = Uart::<6, 4>::new(bits(6));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "uart.md", options)?;
    Ok(())
}
