use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::dht22::{Dht22Reader, In},
    doc::write_svg_as_markdown,
};

fn build(frame: u128) -> Vec<In> {
    let mut out = Vec::new();
    out.push(In {
        start: false,
        data_in: true,
    });
    out.push(In {
        start: true,
        data_in: false,
    });
    for _ in 0..20 {
        out.push(In {
            start: false,
            data_in: false,
        });
    }
    for _ in 0..2 {
        out.push(In {
            start: false,
            data_in: true,
        });
    }
    for _ in 0..8 {
        out.push(In {
            start: false,
            data_in: false,
        });
    }
    for _ in 0..8 {
        out.push(In {
            start: false,
            data_in: true,
        });
    }
    for k in (0..40).rev() {
        let bit = ((frame >> k) & 1) != 0;
        for _ in 0..5 {
            out.push(In {
                start: false,
                data_in: false,
            });
        }
        for _ in 0..(if bit { 7 } else { 3 }) {
            out.push(In {
                start: false,
                data_in: true,
            });
        }
    }
    for _ in 0..8 {
        out.push(In {
            start: false,
            data_in: true,
        });
    }
    out
}

fn main() -> Result<(), RHDLError> {
    let frame = (0x1234u128 << 24) | (0x5678u128 << 8) | 0xAB;
    let stream = build(frame).into_iter().with_reset(1).clock_pos_edge(100);
    let uut = Dht22Reader::<10>::new(bits(18), bits(5), bits(60));
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "dht22.md", options)?;
    Ok(())
}
