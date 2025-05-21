use rhdl::prelude::*;
use rhdl_fpga::{
    rng::xorshift::XorShift128,
    stream::fifo_to_stream::{self, FIFOToStream},
};

fn main() -> Result<(), RHDLError> {
    // The buffer will manage items of 4 bits
    let uut = FIFOToStream::<b4>::default();
    // The test harness will include a consumer that
    // randomly pauses the upstream producer.
    let mut need_reset = true;
    let mut source_rng = XorShift128::default();
    let vcd = uut
        .run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = fifo_to_stream::In::<b4>::dont_care();
                let want_to_pause = rand::random::<u8>() > 200;
                input.ready = !want_to_pause;
                // Decide if the producer will generate a data item
                let want_to_send = rand::random::<u8>() < 200;
                input.data = None;
                if !out.full && want_to_send {
                    input.data = source_rng.next().map(|x| bits((x & 0xF) as u128));
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 1500)
        .collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "fifo_to_rv.md",
        SvgOptions::default().with_io_filter(),
    )?;
    Ok(())
}
