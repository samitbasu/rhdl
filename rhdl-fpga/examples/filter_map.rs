use std::iter::repeat_n;

use rhdl::prelude::*;
use rhdl_fpga::{
    core::slice::lsbs,
    pipe::{
        filter_map::FilterMapPipe,
        map::MapPipe,
        testing::{single_stage::single_stage, utils::stalling},
    },
    rng::xorshift::XorShift128,
};

// Let's assume we are processing a stream of enums
#[derive(PartialEq, Digital, Default)]
enum Item {
    I(b4),
    Q(b4),
    #[default]
    Space,
}

// And we only want to process `I` values in the
// downstream pipe.
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
    // Generate a random stream of items
    let a_rng = XorShift128::default().map(make_item);
    let mut b_rng = a_rng.clone().filter_map(|x| match x {
        Item::I(x) => Some(x),
        _ => None,
    });
    let a_rng = stalling(a_rng, 0.23);
    let consume = move |data: Option<b4>| {
        if let Some(data) = data {
            let orig = b_rng.next().unwrap();
            assert_eq!(data, orig);
        }
        rand::random::<f64>() > 0.2
    };
    let filter_map = FilterMapPipe::try_new::<extract_i_values>()?;
    let uut = single_stage(filter_map, a_rng, consume);
    // Run a few samples through
    let input = repeat_n((), 15).stream_after_reset(1).clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "filter_map.md", SvgOptions::default())?;
    Ok(())
}
