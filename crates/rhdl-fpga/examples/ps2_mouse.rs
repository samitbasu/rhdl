use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::ps2_mouse::{FSM_TRANSITIONS, In, Ps2Mouse},
};

fn drive_byte(data: u8, out: &mut Vec<In>) {
    let parity = (data.count_ones() % 2) == 0;
    let frame_bits: Vec<bool> = std::iter::once(false)
        .chain((0..8).map(|i| (data >> i) & 1 == 1))
        .chain(std::iter::once(parity))
        .chain(std::iter::once(true))
        .collect();
    for &b in &frame_bits {
        for _ in 0..3 {
            out.push(In { clk_in: true, data_in: b });
        }
        for _ in 0..3 {
            out.push(In { clk_in: false, data_in: b });
        }
    }
    for _ in 0..3 {
        out.push(In { clk_in: true, data_in: true });
    }
}

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<Ps2Mouse>(FSM_TRANSITIONS, "ps2_mouse_fsm.md")?;

    let uut = Ps2Mouse::default();
    let mut stream_in: Vec<In> = vec![In { clk_in: true, data_in: true }; 2];
    drive_byte(0x29, &mut stream_in); // status
    drive_byte(0x10, &mut stream_in); // X delta = +16
    drive_byte(0xFB, &mut stream_in); // Y delta = -5
    for _ in 0..6 {
        stream_in.push(In { clk_in: true, data_in: true });
    }
    let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ps2_mouse.md", options)?;
    Ok(())
}
