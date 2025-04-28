use rhdl::prelude::*;
use rhdl_fpga::{core::constant::Constant, doc::write_svg_as_markdown};

fn main() -> Result<(), RHDLError> {
    let inputs = std::iter::repeat(()).stream().clock_pos_edge(100);
    let uut = Constant::new(b8(42));
    let vcd = uut
        .run(inputs)?
        .take_while(|x| x.time < 500)
        .collect::<Vcd>();
    let options = SvgOptions {
        label_width: 20,
        ..Default::default()
    };
    write_svg_as_markdown(vcd, "constant.md", options)?;
    Ok(())
}
