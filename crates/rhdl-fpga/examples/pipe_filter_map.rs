use rhdl::prelude::*;
use rhdl_fpga::{
    pipe::filter_map::FilterMap, rng::xorshift::XorShift128, stream::testing::utils::stalling,
};

// Let's assume we are processing a stream of enums
#[derive(PartialEq, Digital, Default, Clone, Copy)]
enum Item {
    I(b4),
    Q(b4),
    #[default]
    Space,
}

// And we only want to process `I` values in the
// downstream.
#[kernel]
fn extract_i_values(_cr: ClockReset, t: Item) -> Option<b4> {
    match t {
        Item::I(x) => Some(x),
        _ => None,
    }
}

fn make_item(x: u32) -> Item {
    if x & 0b1_0000 != 0 {
        Item::I(bits((x & 0xF) as u128))
    } else {
        Item::Q(bits((x & 0xF) as u128))
    }
}

fn main() -> Result<(), RHDLError> {
    let a_rng = XorShift128::default().map(make_item);
    let a_rng = stalling(a_rng, 0.1);
    let uut = FilterMap::try_new::<extract_i_values>()?;
    let input = a_rng.with_reset(1).clock_pos_edge(100);
    let vcd = uut
        .run(input)
        .take_while(|t| t.time < 1500)
        .collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "pipe_filter_map.md",
        SvgOptions::default().with_io_filter(),
    )?;
    Ok(())
}
