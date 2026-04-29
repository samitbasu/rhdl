use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    serial_bus::sent_rx::{In, SentRx, SentTimings},
};

fn main() -> Result<(), RHDLError> {
    // Compact test timings — 1 SENT tick = 4 FPGA cycles.
    let tick_cycles: u32 = 4;
    let timings = SentTimings::<10> {
        t_nibble_min: bits((12 * tick_cycles) as u128),
        t_nibble_max: bits((27 * tick_cycles) as u128),
        t_sync_min: bits((50 * tick_cycles) as u128),
        t_sync_max: bits((62 * tick_cycles) as u128),
    };
    let uut = SentRx::<10>::new(timings);

    // Build a SENT frame: sync (56 ticks) + 8 nibbles.
    let nibbles: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let mut stream_in: Vec<In> = Vec::new();
    let push = |v: &mut Vec<In>, level: bool, n: u32| {
        for _ in 0..n {
            v.push(In { sent_in: level });
        }
    };
    push(&mut stream_in, true, 16);
    // Sync: 5 low + 51 high = 56 ticks.
    push(&mut stream_in, false, 5 * tick_cycles);
    push(&mut stream_in, true, 51 * tick_cycles);
    // 8 nibbles, each 5 low + (12 + N - 5) high = (12 + N) ticks.
    for n in nibbles {
        let total = 12 + n as u32;
        push(&mut stream_in, false, 5 * tick_cycles);
        push(&mut stream_in, true, (total - 5) * tick_cycles);
    }
    // Trailing falling edge so the 8th measurement completes.
    push(&mut stream_in, false, 5 * tick_cycles);
    push(&mut stream_in, true, 80);

    let stream = stream_in.into_iter().with_reset(1).clock_pos_edge(100);
    let vcd = uut.run(stream).collect::<SvgFile>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)|(^top.clock.*)|(^top.reset.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "sent_rx.md", options)?;
    Ok(())
}
