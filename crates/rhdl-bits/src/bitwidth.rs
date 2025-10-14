/// A marker trait used to constrain a const generic parameter to the range 1..=128
/// This is used to ensure that the number of bits in a [Bits] or [SignedBits]
/// value is in a valid range.
/// ```
/// use rhdl_bits::consts::*;
/// fn takes_variable_bit_widths<const N: usize>(value: Bits<N>) where W<N> : BitWidth {}
/// ```
/// This is a work around for the constraint `where N >= 1 && N <= 128` which is not
/// currently supported in Rust.
///
pub trait BitWidth {}

/// A type-level representation of a bit width.  This is used to constrain
/// const generic parameters to the range 1..=128.
///
/// This is a work around for the constraint `where N >= 1 && N <= 128` which is not
/// currently supported in Rust.  So instead, you write `W<N>: BitWidth`, which is only
/// valid if `N` is in the range 1..=128.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct W<const N: usize>;

seq_macro::seq!(N in 1..=128 {
    #(
        impl BitWidth for W<N> {}
    )*
});
