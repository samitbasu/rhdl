use rand::{Rng, SeedableRng};
use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    reset::conditioner::{In, ResetConditioner},
};

fn sync_stream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
    // Assume the red stuff comes on the edges of a clock
    let red = (0..)
        .map(move |_| rng.gen::<u8>() > 150)
        .take(100)
        .with_reset(1)
        .clock_pos_edge(50);
    let blue = std::iter::repeat(false)
        .with_reset(1)
        .clock_pos_edge(79);
    red.merge(blue, |r, g| In {
        reset: signal(reset(r.1)),
        clock: signal(g.0.clock),
    })
}

fn main() -> Result<(), RHDLError> {
    let uut = ResetConditioner::<Red, Blue>::default();
    let input = sync_stream();
    let vcd = uut
        .run(input)?
        .take_while(|t| t.time < 1400)
        .collect::<Vcd>();
    write_svg_as_markdown(vcd, "reset_conditioner.md", SvgOptions::default())?;
    Ok(())
}
