use rhdl::core::{BitX, Kind};

// ANCHOR: digital-trait
pub trait Digital: Copy + PartialEq + Sized + Clone + 'static {
    /// Associated constant that gives the total number of bits needed to represent the value.
    const BITS: usize;
    /// Returns the [Kind] (run time type descriptor) of the value as a static method
    fn static_kind() -> Kind;
    /// Returns the binary representation of the value as a vector of [BitX].
    fn bin(self) -> Box<[BitX]>;
    /// Returns a "don't care" value for the type.
    fn dont_care() -> Self;
}
// ANCHOR_END: digital-trait
