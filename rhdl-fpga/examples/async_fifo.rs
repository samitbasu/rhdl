use rhdl::prelude::*;
use rhdl_fpga::{
    doc::write_svg_as_markdown,
    fifo::testing::{
        async_tester::{AsyncFIFOTester, In},
        drainer::FIFODrainer,
    },
};

fn main() -> Result<(), RHDLError> {
    // Use the AsyncFIFOTester to exercise an async fifo
    let uut =
        AsyncFIFOTester::<Red, Blue, U16, 2>::default().with_drainer(FIFODrainer::new(5, 0.812));
    let red_input = std::iter::repeat(())
        .stream_after_reset(1)
        .clock_pos_edge(50);
    let blue_input = std::iter::repeat(())
        .stream_after_reset(1)
        .clock_pos_edge(78);
    let input = red_input.merge(blue_input, |r, b| In {
        cr_w: signal(r.0),
        cr_r: signal(b.0),
    });
    let vcd = uut
        .run(input.take(10000))?
        .take_while(|t| t.time < 1500)
        .collect::<Vcd>();
    let options =
        SvgOptions::default().with_filter(r"(^top.fifo.input(.*))|(^top.fifo.outputs(.*))");
    write_svg_as_markdown(vcd, "async_fifo.md", options)?;
    Ok(())
}
