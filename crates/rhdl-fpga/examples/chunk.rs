//! In this test fixture, we assemble a source, a chunker, and a sink
//! into a single assembly:
//!
#![doc=badascii!(r"
+-+Source+-+     +-+UUT+------+      +-+Sink+---+
|          | ?T  |            | ?S   |          |
|     data +---->|in       out+----->|data      |
|          |     |            |      |          |
|          |     |            |      |          |
|    ready |<---+|ready  ready|<-----+ready     |
+----------+     +------------+      +----------+
")]

use badascii_doc::badascii;
use rhdl::{core::sim::run::synchronous::RunWithoutSynthesisSynchronousExt, prelude::*};
use rhdl_fpga::{
    rng::xorshift::XorShift128,
    stream::{
        chunked::Chunked,
        testing::{single_stage::single_stage, utils::stalling},
    },
};

fn mk_array<T, const N: usize>(mut t: impl Iterator<Item = T>) -> impl Iterator<Item = [T; N]> {
    std::iter::from_fn(move || Some(core::array::from_fn(|_| t.next().unwrap())))
}

fn main() -> Result<(), RHDLError> {
    // First, we create an iterator that generates random nibbles
    let source_rng = XorShift128::default().map(|x| b4((x & 0xF) as u128));
    // A duplicate of the iterator for the sink (to compare results)
    let dest_rng = source_rng.clone();
    // This function converts the source_rng into one that yields
    // [Option<b4>], but returns [None] with the given probability.
    // This mimics an upstream source that is stalled.
    let stalling_source = stalling(source_rng, 0.2);
    // Here, we create an iterator that collects the `dest_rng` iterator output
    // into groups of 4 elements.
    let mut dest_rng_chunked = mk_array::<_, 4>(dest_rng);
    // Create the thing to test
    let uut = Chunked::<b4, 2, 4>::default();
    // Create the consumption function.
    let consume = move |data| {
        // The sink simply compares the `Some` values with
        // the expected output of the iterator
        // It panics if the data disagree
        if let Some(data) = data {
            let validation = dest_rng_chunked.next().unwrap();
            assert_eq!(data, validation);
        }
        // The output of the closure is the value of `ready` for the
        // next clock cycle.
        rand::random::<f64>() > 0.3
    };
    // Create a single stage test fixture
    let uut = single_stage(uut, stalling_source, consume);
    // Feed the UUT a steady diet of clock pulses signals
    let input = std::iter::repeat_n((), 15)
        .with_reset(1)
        .clock_pos_edge(100);
    let vcd = uut.run_without_synthesis(input)?.collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "chunk.md",
        SvgOptions::default().with_filter("(^top.uut.input\\.)|(^top.uut.outputs.data)"),
    )?;
    Ok(())
}
