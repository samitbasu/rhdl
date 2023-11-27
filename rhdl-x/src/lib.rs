use rhdl_bits::alias::*;
use rhdl_macro::Digital;

#[derive(Clone, Copy, Debug, PartialEq, Digital, Default)]
pub struct Foo {
    pub field1: b4,
    pub field2: b2,
    pub field3: (b4, b6),
}
