use rhdl_bits::alias::*;
use rhdl_bits::Bits;
use rhdl_macro::Digital;

use crate::clock::Clock;
use crate::{circuit::Circuit, constant::Constant, dff::DFF, strobe::Strobe};

#[derive(Clone)]
pub struct Push {
    strobe: Strobe<32>,
    stroke: DFF<bool>,
    value: Constant<Bits<32>>,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushQ {
    strobe: <Strobe<32> as Circuit>::O,
    stroke: <DFF<bool> as Circuit>::O,
    value: <Constant<Bits<8>> as Circuit>::O,
}

// Auto generated
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct PushD {
    strobe: <Strobe<32> as Circuit>::I,
    stroke: <DFF<bool> as Circuit>::I,
    value: <Constant<Bits<8>> as Circuit>::I,
}

impl Circuit for PushD {
    type I = Clock;

    type O = ();

    type IO = b8;

    type Q = PushQ;

    type D = PushD;

    type S = (
        Self::Q,
        <Strobe<32> as Circuit>::S,
        <DFF<bool> as Circuit>::S,
        <Constant<Bits<8>> as Circuit>::S,
    );

    type Update = Self;

    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = |i, _| ((), i);
}
