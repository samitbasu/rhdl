use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::spi_slave::{In, SpiSlave},
    doc::write_svg_as_markdown,
};

fn drive_byte(byte: u128, tx: u128) -> Vec<In<8>> {
    let mut out: Vec<In<8>> = Vec::new();
    for _ in 0..2 {
        out.push(In {
            sclk_in: false,
            mosi_in: false,
            cs_n_in: true,
            tx_data: bits(tx),
        });
    }
    out.push(In {
        sclk_in: false,
        mosi_in: false,
        cs_n_in: false,
        tx_data: bits(tx),
    });
    for k in 0..8 {
        let bit = ((byte >> (7 - k)) & 1) != 0;
        out.push(In {
            sclk_in: false,
            mosi_in: bit,
            cs_n_in: false,
            tx_data: bits(tx),
        });
        out.push(In {
            sclk_in: true,
            mosi_in: bit,
            cs_n_in: false,
            tx_data: bits(tx),
        });
    }
    for _ in 0..4 {
        out.push(In {
            sclk_in: false,
            mosi_in: false,
            cs_n_in: true,
            tx_data: bits(tx),
        });
    }
    out
}

fn main() -> Result<(), RHDLError> {
    let stream = drive_byte(0xA5, 0x42)
        .into_iter()
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = SpiSlave::<8, 4>::default();
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "spi_slave.md", options)?;
    Ok(())
}
