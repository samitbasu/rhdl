use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::ps2_keyboard::{FSM_TRANSITIONS, In, Ps2Keyboard},
};

fn main() -> Result<(), RHDLError> {
    write_fsm_diagram_as_markdown::<Ps2Keyboard>(FSM_TRANSITIONS, "ps2_keyboard_fsm.md")?;

    let uut = Ps2Keyboard::default();
    // Send the scan code 0x1C ('A' on Set 2).
    let data: u8 = 0x1C;
    let parity = (data.count_ones() % 2) == 0;
    let frame_bits: Vec<bool> = std::iter::once(false)
        .chain((0..8).map(|i| (data >> i) & 1 == 1))
        .chain(std::iter::once(parity))
        .chain(std::iter::once(true))
        .collect();
    let mut stream_in: Vec<In> = vec![In { clk_in: true, data_in: true }; 2];
    for &b in &frame_bits {
        for _ in 0..3 {
            stream_in.push(In { clk_in: true, data_in: b });
        }
        for _ in 0..3 {
            stream_in.push(In { clk_in: false, data_in: b });
        }
    }
    for _ in 0..6 {
        stream_in.push(In { clk_in: true, data_in: true });
    }
    let stream = stream_in.into_iter().with_reset(2).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ps2_keyboard.md", options)?;
    Ok(())
}
