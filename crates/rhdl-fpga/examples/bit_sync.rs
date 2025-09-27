use rand::random;
use rhdl::prelude::*;
use rhdl_fpga::{
    cdc::synchronizer::{In, Sync1Bit},
    doc::write_svg_as_markdown,
};

fn main() -> Result<(), RHDLError> {
    // Start with a stream of pulses
    let red = (0..).map(|_| random::<u8>() > 200).take(100);
    // Clock them on the red domain
    let red = red.with_reset(1).clock_pos_edge(100);
    // Create an empty stream on the blue domain
    let blue = std::iter::repeat(()).with_reset(1).clock_pos_edge(79);
    // Merge them
    let inputs = merge(red, blue, |r: (ClockReset, bool), b: (ClockReset, ())| In {
        data: signal(r.1),
        cr: signal(b.0),
    });
    // Next we create an instance of the 1-bit synchronizercore, with
    // the appropriate clock domains.
    let uut = Sync1Bit::<Red, Blue>::default();
    // Simulate the crosser, and collect into a VCD
    let vcd = uut
        .run(inputs)
        .take_while(|x| x.time < 2000)
        .collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "sync_cross.md", options)?;
    Ok(())
}
