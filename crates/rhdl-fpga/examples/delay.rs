use rhdl::prelude::*;
use rhdl_fpga::{core::delay::Delay, doc::write_svg_as_markdown};

#[derive(PartialEq, Digital, Default)]
pub enum State {
    Init,
    Running,
    Busted,
    Reset,
    #[default]
    Unknown,
}

fn main() -> Result<(), RHDLError> {
    let input = [
        State::Unknown,
        State::Init,
        State::Reset,
        State::Busted,
        State::Running,
        State::Busted,
    ]
    .into_iter()
    .chain(std::iter::repeat_n(State::Unknown, 4));
    let input = input.with_reset(1).clock_pos_edge(100);
    let uut: Delay<State, 3> = Delay::default();
    let vcd = uut.run(input).collect::<Vcd>();
    let options = SvgOptions::default()
        .with_filter("(^top.input.*)|(^top.output.*)")
        .with_label_width(20);
    write_svg_as_markdown(vcd, "delay.md", options)?;
    Ok(())
}
