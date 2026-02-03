use rhdl::prelude::*;
use rhdl_fpga::{rng::xorshift::XorShift128, stream::stream_to_fifo::StreamToFIFO};

fn main() -> Result<(), RHDLError> {
    // The buffer will manage items of 4 bits
    let uut = StreamToFIFO::<b4>::default();
    // The test harness will include a consumer that
    // randomly pauses the upstream producer.
    let mut need_reset = true;
    let mut source_rng = XorShift128::default().map(|x| bits((x & 0xF) as u128));
    let mut dest_rng = source_rng.clone();
    let mut source_datum = source_rng.next();
    let vcd = uut
        .run_fn(
            |out| {
                if need_reset {
                    need_reset = false;
                    return Some(rhdl::core::sim::ResetOrData::Reset);
                }
                let mut input = rhdl_fpga::stream::stream_to_fifo::In::<b4>::dont_care();
                let may_accept = rand::random::<u8>() > 150;
                let will_accept = may_accept & out.data.is_some();
                input.next = false;
                if will_accept {
                    assert_eq!(out.data, dest_rng.next());
                    input.next = true;
                }
                let will_offer = rand::random::<u8>() > 150;
                if will_offer {
                    input.data = source_datum;
                } else {
                    input.data = None;
                }
                let will_advance = will_offer & out.ready.raw;
                if will_advance {
                    source_datum = source_rng.next();
                }
                Some(rhdl::core::sim::ResetOrData::Data(input))
            },
            100,
        )
        .take_while(|t| t.time < 1500)
        .collect::<SvgFile>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "rv_to_fifo.md",
        SvgOptions::default().with_io_filter(),
    )?;
    Ok(())
}
