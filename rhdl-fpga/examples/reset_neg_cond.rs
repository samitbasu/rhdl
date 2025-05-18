use rand::{Rng, SeedableRng};
use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    reset::negating_conditioner::{In, NegatingConditioner},
};

fn istream() -> impl Iterator<Item = TimedSample<In<Red, Blue>>> {
    // Use a seeded RNG to get repeatable results
    let mut rng = rand::rngs::StdRng::seed_from_u64(0xdead_beef);
    let red = (0..)
        .map(move |_| rng.random::<u8>() < 200)
        .take(100)
        .without_reset()
        .clock_pos_edge(100);
    let blue = std::iter::repeat(()).without_reset().clock_pos_edge(79);
    red.merge(blue, |r, b| In {
        reset_n: signal(reset_n(r.1)),
        clock: signal(b.0.clock),
    })
}

fn main() -> Result<(), RHDLError> {
    let uut = NegatingConditioner::default();
    let vcd = uut.run(istream())?.collect::<Vcd>();
    write_svg_as_markdown(vcd, "reset_neg_cond.md", SvgOptions::default())?;
    Ok(())
}
