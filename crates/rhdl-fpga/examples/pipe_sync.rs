use rhdl::prelude::*;
use rhdl_fpga::core::ram::pipe_sync::{In, PipeSyncBRAM};

fn main() -> Result<(), RHDLError> {
    // Generate the stream example from the timing diagram.
    // Read location 2, then 3 then 2 again
    let reads = [None, Some(b3(2)), Some(b3(3)), Some(b3(2)), None];
    // Write to location 2 while reading from 3
    let writes = [None, None, Some((b3(2), b8(42))), None, None];
    let inputs = reads
        .into_iter()
        .zip(writes)
        .map(|(r, w)| In { read: r, write: w })
        .with_reset(1)
        .clock_pos_edge(100);
    let uut = PipeSyncBRAM::new((0..).map(|x| (b3(x), b8(x))));
    let vcd = uut.run(inputs).collect::<Svg>();
    rhdl_fpga::doc::write_svg_as_markdown(
        vcd,
        "pipe_ram.md",
        SvgOptions::default()
            .with_label_width(20)
            .with_filter("(^top.clock.*)|(^top.input.*)|(^top.output.*)"),
    )
    .unwrap();
    Ok(())
}
