use rhdl::prelude::*;

// ANCHOR: output_example
//                                          👇 - new!
#[derive(PartialEq, Digital, Clone, Copy, Timed)]
pub struct Output {
    out1: Signal<b16, Red>,
    out2: Signal<b16, Red>,
    t_clk: Signal<Clock, Red>,
    t_out: Signal<b16, Red>,
    p_out: Signal<b16, Red>,
    bt_ready: Signal<bool, Red>,
}
// ANCHOR_END: output_example

#[cfg(feature = "timed_impl")]
// ANCHOR: timed_impl
impl Timed for Output
where
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<Clock, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<b16, Red>: rhdl::core::Timed,
    Signal<bool, Red>: rhdl::core::Timed,
{
}
// ANCHOR_END: timed_impl

#[cfg(feature = "timed_blanket_impl")]
// ANCHOR: timed_blanket_impl
impl Timed for Output {}
// ANCHOR_END: timed_blanket_impl
