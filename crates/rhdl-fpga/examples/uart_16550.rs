use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::uart_16550::{In, Uart16550},
};

fn main() -> Result<(), RHDLError> {
    let divisor = 6u128;
    let uut = Uart16550::<6, 4>::new(bits(divisor));

    // Demo: probe DLAB / DLL / DLM, then write a byte to THR and
    // read back LSR + IIR.  Exercises both the divisor-latch bank
    // and the regular bank in one trace.
    let idle = In {
        addr: bits(0),
        write_data: bits(0),
        read_enable: false,
        write_enable: false,
        rx: true,
        cts_n: true,
        dsr_n: true,
        ri_n: true,
        dcd_n: true,
    };
    let write_reg = |addr, data| In {
        addr: bits(addr),
        write_data: bits(data),
        write_enable: true,
        ..idle
    };
    let read_reg = |addr| In {
        addr: bits(addr),
        read_enable: true,
        ..idle
    };

    let mut stream_in: Vec<In> = vec![
        write_reg(0x3, 0x80), // LCR ← 0x80 (set DLAB)
        write_reg(0x0, 0x06), // DLL ← 6  (matches construction divisor)
        write_reg(0x1, 0x00), // DLM ← 0
        write_reg(0x3, 0x00), // LCR ← 0  (clear DLAB)
        write_reg(0x1, 0x03), // IER ← 0x03 (enable RX + TX irqs)
        write_reg(0x4, 0x10), // MCR ← 0x10 (loopback)
        write_reg(0x0, 0x55), // THR ← 0x55
    ];
    for _ in 0..120 {
        stream_in.push(idle);
    }
    stream_in.push(read_reg(0x5)); // read LSR
    stream_in.push(read_reg(0x2)); // read IIR
    stream_in.push(read_reg(0x0)); // read RBR (loopback byte)
    for _ in 0..16 {
        stream_in.push(idle);
    }

    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "uart_16550.md", options)?;
    Ok(())
}
