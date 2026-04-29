use rhdl::prelude::*;
use rhdl_fpga::{
    doc::{write_fsm_diagram_as_markdown, write_svg_as_markdown},
    serial_bus::ir_nec_rx::{In, IrNecRx, NecTimings, FSM_TRANSITIONS},
};

fn main() -> Result<(), RHDLError> {
    // Emit the FSM diagram first — required by CLAUDE.md §12 rule 14.
    write_fsm_diagram_as_markdown::<IrNecRx<14>>(FSM_TRANSITIONS, "ir_nec_rx_fsm.md")?;

    // Compact test timings — small enough that a frame fits in a few hundred
    // FPGA cycles, with the standard NEC ratios preserved.
    let timings = NecTimings::<14> {
        t_lead_burst_min: bits(80),
        t_lead_burst_max: bits(120),
        t_lead_data_threshold: bits(35),
        t_lead_space_max: bits(60),
        t_data_zero_one_threshold: bits(11),
        t_data_space_max: bits(30),
    };
    let uut = IrNecRx::<14>::new(timings);

    // Build a hand-crafted NEC waveform encoding the 32-bit code 0x12345678.
    let code: u32 = 0x12345678;
    let mut stream_in: Vec<In> = Vec::new();
    let push = |v: &mut Vec<In>, level: bool, n: u32| {
        for _ in 0..n {
            v.push(In { ir_in: level });
        }
    };
    push(&mut stream_in, true, 16); // settle
    push(&mut stream_in, false, 90); // leading burst (~9 ms scaled)
    push(&mut stream_in, true, 45); // leading space (~4.5 ms scaled)
    for bit in (0..32).rev() {
        push(&mut stream_in, false, 6); // burst
        let one = (code >> bit) & 1 == 1;
        push(&mut stream_in, true, if one { 17 } else { 6 });
    }
    push(&mut stream_in, false, 6); // final stop burst
    push(&mut stream_in, true, 50); // trailing idle

    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "ir_nec_rx.md", options)?;
    Ok(())
}
