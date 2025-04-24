use rand::random;
use rhdl::{core::trace::svg::SvgOptions, prelude::*};
use rhdl_fpga::cdc::cross_counter::{In, Out, Unit};

// This function will generate a stream of random pulses in the red
// clock domain.
fn sync_stream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
    // Start with a stream of pulses
    let red = (0..).map(|_| random::<bool>()).take(100);
    // Clock them on the red domain
    let red = red.stream_after_reset(1).clock_pos_edge(100);
    // Create an empty stream on the blue domain
    let blue = std::iter::repeat(false)
        .stream_after_reset(1)
        .clock_pos_edge(79);
    // Merge them
    merge(red, blue, |r: (ClockReset, bool), b: (ClockReset, bool)| {
        In {
            incr: signal(r.1),
            incr_cr: signal(r.0),
            cr: signal(b.0),
        }
    })
}

fn main() -> Result<(), RHDLError> {
    // Next we create an instance of the clock-domain crossing core, with
    // the appropriate clock domains.
    let uut = Unit::<Red, Blue, 4>::default();
    // Simulate the crosser, and collect into a VCD
    let vcd = uut
        .run(sync_stream())?
        .take_while(|x| x.time < 1000)
        .collect::<Vcd>();
    std::fs::create_dir_all("test_vcd").unwrap();
    let mut options = SvgOptions::default();
    options.label_width = 20;
    std::fs::write(
        "test_vcd/cross_counter.svg",
        &vcd.dump_svg(&options).to_string(),
    )
    .unwrap();
    Ok(())
}
