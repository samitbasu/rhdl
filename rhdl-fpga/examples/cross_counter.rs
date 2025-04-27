use rand::random;
use rhdl::prelude::*;
use rhdl_fpga::{
    cdc::cross_counter::{CrossCounter, In, Out},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    // Start with a stream of pulses
    let red = (0..).map(|_| random::<bool>()).take(100);
    // Clock them on the red domain
    let red = red.stream_after_reset(1).clock_pos_edge(100);
    // Create an empty stream on the blue domain
    let blue = std::iter::repeat(())
        .stream_after_reset(1)
        .clock_pos_edge(79);
    // Merge them
    let inputs = merge(red, blue, |r: (ClockReset, bool), b: (ClockReset, ())| In {
        incr: signal(r.1),
        incr_cr: signal(r.0),
        cr: signal(b.0),
    });
    // Next we create an instance of the clock-domain crossing core, with
    // the appropriate clock domains.
    let uut = CrossCounter::<Red, Blue, 4>::default();
    // Simulate the crosser, and collect into a VCD
    let vcd = uut
        .run(inputs)?
        .take_while(|x| x.time < 1000)
        .collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "cross_counter.md", options)?;
    Ok(())
}
