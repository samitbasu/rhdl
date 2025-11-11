use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, fifo::synchronous::SyncFIFO};

fn main() -> Result<(), RHDLError> {
    type Uut = SyncFIFO<Bits<16>, 4>;
    let uut = Uut::default();
    let mut need_reset = true;
    let test_seq = (0..)
        .map(|_| b16(rand::random::<u16>() as u128))
        .take(100)
        .collect::<Vec<_>>();
    let mut input_seq = test_seq.iter().copied();
    let mut output_seq = test_seq.iter().copied();
    let vcd = uut
        .run_fn(
            |output| {
                // Handle the reset pulse at the begining
                if need_reset {
                    need_reset = false;
                    return Some(None);
                }
                let mut input = <Uut as SynchronousIO>::I::dont_care();
                // By default, we do not insert more data.
                input.data = None;
                // The FIFO is not full, so we can insert data.  Toss a coin.
                if !output.full && rand::random::<bool>() {
                    input.data = input_seq.next();
                }
                // By default, we do not advance the FIFO
                input.next = false;
                // The FIFO has valid data.  So we can receive data. Toss a coin.
                if output.data.is_some() && rand::random::<bool>() {
                    input.next = true;
                    assert_eq!(output_seq.next().unwrap(), output.data.unwrap());
                }
                // Return the constructed input
                Some(Some(input))
            },
            100,
        )
        .take_while(|t| t.time < 1500)
        .collect::<Vcd>();
    let options = SvgOptions::default().with_filter("(^top.input(.*))|(^top.outputs(.*))");
    write_svg_as_markdown(vcd, "sync_fifo.md", options)?;
    Ok(())
}
