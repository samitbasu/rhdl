use rhdl::prelude::*;
use rhdl_fpga::{doc::write_svg_as_markdown, fifo::asynchronous::AsyncFIFO};

fn main() -> Result<(), RHDLError> {
    let uut = AsyncFIFO::<Bits<16>, Red, Blue, 3>::default();
    let test_seq = (0..)
        .map(|_| b16(rand::random::<u16>() as u128))
        .take(100)
        .collect::<Vec<_>>();
    let mut input_seq = test_seq.iter().copied();
    let mut output_seq = test_seq.iter().copied();
    let vcd = run_async_red_blue(
        &uut,
        |output, input| {
            // By default, we do not insert data.
            input.data = signal(None);
            if !output.full.val() && rand::random::<bool>() {
                input.data = signal(input_seq.next());
            }
        },
        |output, input| {
            input.next = signal(false);
            if output.data.val().is_some() && rand::random::<bool>() {
                input.next = signal(true);
                assert_eq!(output_seq.next(), output.data.val())
            }
        },
        50,
        78,
        |red, blue, input| {
            input.cr_r = blue;
            input.cr_w = red;
        },
    )
    .take_while(|t| t.time < 1500)
    .collect::<Svg>();
    let options = SvgOptions::default().with_io_filter();
    write_svg_as_markdown(vcd, "async_fifo.md", options)?;
    Ok(())
}
