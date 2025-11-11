use rhdl::prelude::*;
use rhdl_fpga::{
    lid::carloni::{self, Carloni},
    rng::xorshift::XorShift128,
};

fn main() -> Result<(), RHDLError> {
    // The buffer will manage items of 4 bits
    let uut = Carloni::<b4>::default();
    // The test harness will include a consumer that
    // randomly pauses the upstream producer.  The producer
    // will also randomly pause it's output.
    let mut need_reset = true;
    let mut source_rng = XorShift128::default();
    let vcd = uut
        .run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(None);
                }
                let mut input = carloni::In::<b4>::dont_care();
                // See if we want to pause the stream as the consumer
                let want_to_pause = rand::random::<u8>() > 200;
                input.stop_in = want_to_pause;
                // Decide if the producer will generate a data item
                let want_to_send = rand::random::<u8>() < 200;
                input.void_in = true;
                input.data_in = bits(0);
                if !out.stop_out && want_to_send {
                    // The receiver did not tell us to stop, and
                    // we want to send something.
                    input.data_in = bits((source_rng.next().unwrap() & 0xF) as u128);
                    input.void_in = false;
                }
                Some(Some(input))
            },
            100,
        )
        .take_while(|t| t.time < 1500)
        .collect::<Vcd>();
    rhdl_fpga::doc::write_svg_as_markdown(vcd, "carloni.md", SvgOptions::default())?;
    Ok(())
}
