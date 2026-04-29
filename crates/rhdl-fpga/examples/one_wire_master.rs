use rhdl::prelude::*;
use rhdl_fpga::{
    serial_bus::one_wire_master::{In, OneWireMaster, OneWireOp, OneWireTimings},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    // Compact test timings — every duration small enough that the full trace
    // fits in a few hundred FPGA cycles.
    let timings = OneWireTimings::<10> {
        t_rst_low: bits(48),
        t_rst_sample: bits(7),
        t_rst_total: bits(96),
        t_w0: bits(6),
        t_w1: bits(1),
        t_read_low: bits(1),
        t_read_sample: bits(2),
        t_slot: bits(7),
    };
    let uut = OneWireMaster::<10>::new(timings);
    let mut stream_in: Vec<In> = vec![In {
        op: OneWireOp::WriteByte,
        data: bits(0xA5),
        start: true,
        bus_in: true,
    }];
    for _ in 0..400 {
        stream_in.push(In {
            op: OneWireOp::WriteByte,
            data: bits(0),
            start: false,
            bus_in: true,
        });
    }
    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "one_wire_master.md", options)?;
    Ok(())
}
