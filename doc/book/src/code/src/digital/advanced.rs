// ANCHOR: lsb
use rhdl::prelude::*;

#[derive(Copy, PartialEq, Clone, Digital, Default)]
#[rhdl(discriminant_align = "lsb")] // 👈 - New!
pub enum OpCodeLsb {
    #[default]
    Nop,
    Add(b8, b8),
    Sub(b8, b8),
    Not(b8),
}
// ANCHOR_END: lsb

// ANCHOR: lsb_svg_test
#[test]
fn lsb_svg_test() {
    let svg = OpCodeLsb::static_kind().svg("OpCodeLsb (Autoderive LSB)");
    std::fs::write("opcode_lsb_derived.svg", svg.to_string()).unwrap();
}
// ANCHOR_END: lsb_svg_test

// ANCHOR: lsb_print_test
#[test]
fn lsb_print_test() {
    let op = OpCodeLsb::Not(0xA5.into());
    println!("{}", bitx_string(&op.bin()));
}
// ANCHOR_END: lsb_print_test

// ANCHOR: disc4bit
#[derive(Copy, PartialEq, Clone, Digital, Default)]
#[rhdl(discriminant_width = 4)] // 👈 - New!
pub enum OpCode {
    #[default]
    Nop,
    Add(b8, b8),
    Sub(b8, b8),
    Not(b8),
}

#[test]
fn disc4bit_svg_test() {
    let svg = OpCode::static_kind().svg("OpCode (4 bit discriminant)");
    std::fs::write("opcode_4bit_derived.svg", svg.to_string()).unwrap();
}
// ANCHOR_END: disc4bit

// ANCHOR: state-naive
#[derive(PartialEq, Clone, Copy, Digital, Default)]
pub enum State {
    Idle,
    Processing(b8),
    Sending(b1),
    Done,
    Fault,
    #[default]
    Reset,
}
// ANCHOR_END: state-naive

// ANCHOR: state-naive-test
#[test]
fn state_naive_svg_test() {
    let svg = State::static_kind().svg("State (Naive)");
    std::fs::write("state_naive.svg", svg.to_string()).unwrap();
}
// ANCHOR_END: state-naive-test

#[cfg(feature = "doc1hot")]
// ANCHOR: one-hot
#[derive(PartialEq, Clone, Copy, Digital, Default)]
pub enum State1Hot {
    Idle = 1, // 👈 - note the explicit values
    Processing(b8) = 2,
    Sending(b1) = 4,
    Done = 8,
    Fault = 16,
    #[default]
    Reset = 0,
}
// ANCHOR_END: one-hot

#[cfg(not(feature = "doc1hot"))]
// ANCHOR: one-hot-fixed
#[derive(PartialEq, Clone, Copy, Digital, Default)]
#[repr(u8)] // 👈 - for rustc.  ignored by rhdl
pub enum State1Hot {
    Idle = 1,
    Processing(b8) = 2,
    Sending(b1) = 4,
    Done = 8,
    Fault = 16,
    #[default]
    Reset = 0,
}
// ANCHOR_END: one-hot-fixed

// ANCHOR: one-hot-test
#[test]
fn state_one_hot_svg_test() {
    let svg = State1Hot::static_kind().svg("State (One-Hot)");
    std::fs::write("state_one_hot.svg", svg.to_string()).unwrap();
}
// ANCHOR_END: one-hot-test

// ANCHOR: state-signed-disc
#[derive(PartialEq, Clone, Copy, Digital, Default)]
#[repr(i8)] // 👈 Now `i8`, not `u8`
pub enum StateSigned {
    Idle = -5,
    Processing(b8) = -3,
    Sending(b1) = 1,
    Done = 6,
    Fault = 9,
    #[default]
    Reset = 0,
}
// ANCHOR_END: state-signed-disc

#[test]
fn state_signed_svg_test() {
    let svg = StateSigned::static_kind().svg("State (Signed Discriminant)");
    std::fs::write("state_signed_disc.svg", svg.to_string()).unwrap();
}
