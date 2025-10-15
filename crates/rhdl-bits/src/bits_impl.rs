#![allow(non_camel_case_types)]
use super::{BitWidth, dyn_bits::DynBits, signed, signed_bits_impl::SignedBits};
use crate::bitwidth::W;
use seq_macro::seq;
/// The [Bits] type is a fixed-sized bit vector.  It is meant to
/// imitate the behavior of bit vectors in hardware.  Due to the
/// design of the [Bits] type, you can only create a [Bits] type of
/// up to 128 bits in length for now.  However, you can easily express
/// larger constructs in hardware using arrays, tuples and structs.
/// The only real limitation of the [Bits] type being 128 bits is that
/// you cannot perform arbitrary arithmetic on longer bit values in your
/// hardware designs.  I don't think this is a significant issue, but
/// the [Bits] design of the `rust-hdl` crate was much slower and harder
/// to maintain and use.  I think this is a good trade-off.
///
/// Note that the [Bits] type implements 2's complement arithmetic.
/// See <https://en.wikipedia.org/wiki/Two%27s_complement> for more
/// information.
///
/// Note also that the [Bits] kind is treated as an unsigned value for
/// the purposes of comparisons.  If you need signed comparisons, you
/// will need the [SignedBits] type.
///
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct Bits<const N: usize>
where
    W<N>: BitWidth,
{
    /// The raw value of the bits.  Only the lowest N bits are used.
    pub(crate) val: u128,
}

impl<const N: usize> std::fmt::Debug for Bits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}_b{}", self.val, N)
    }
}

impl<const N: usize> std::fmt::Display for Bits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'d{}", N, self.val)
    }
}

impl<const N: usize> std::fmt::LowerHex for Bits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'h{:x}", N, self.val)
    }
}

impl<const N: usize> std::fmt::UpperHex for Bits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'H{:X}", N, self.val)
    }
}

impl<const N: usize> std::fmt::Binary for Bits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'b{:b}", N, self.val)
    }
}

seq!(N in 1..=128 {
    #(
        /// A helper type alias for [Bits] of size N.
        pub type b~N = Bits<N>;
        /// A helper function for creating a [Bits] value of size N
        pub const fn b~N(value: u128) -> b~N {
            bits::<N>(value)
        }
    )*
});

/// Helper function for creating a bits value from
/// a constant.
/// ```
/// # use rhdl_bits::{Bits, bits};
/// let value : Bits<8> = bits(0b1010_1010);
/// assert_eq!(value, 0b1010_1010);
/// ```
/// Because the function is `const`, you can use it a constant
/// context:
/// ```
/// # use rhdl_bits::{Bits, bits};
/// const VALUE : Bits<8> = bits(0b1010_1010);
/// ```
pub const fn bits<const N: usize>(value: u128) -> Bits<N>
where
    W<N>: BitWidth,
{
    assert!(value <= Bits::<N>::mask().val);
    Bits { val: value }
}

/// Helper function for creating a bits value from
/// a constant, masking off any excess bits.
/// ```
/// # use rhdl_bits::{Bits, bits_masked};
/// let value : Bits<8> = bits_masked(0b1_1010_1010);
/// assert_eq!(value, 0b1010_1010);
/// ```
/// Because the function is `const`, you can use it a constant context:
/// ```
/// # use rhdl_bits::{Bits, bits_masked};
/// const VALUE : Bits<8> = bits_masked(0b1_1010_1010);
/// assert_eq!(VALUE, 0b1010_1010);
/// ```
pub const fn bits_masked<const N: usize>(value: u128) -> Bits<N>
where
    W<N>: BitWidth,
{
    Bits {
        val: value & Bits::<N>::mask().val,
    }
}

/// This struct is needed so that the `bits` function can be used in synthesizable
/// contexts.
#[doc(hidden)]
pub struct bits<const N: usize>
where
    W<N>: BitWidth, {}

impl<const N: usize> Bits<N>
where
    W<N>: BitWidth,
{
    /// Defines a constant Bits value with all bits set to 1.
    pub const MASK: Self = Self::mask();
    /// Defines a constant Bits value set to the maximum storable value.
    pub const MAX: Self = Self::mask();
    /// Defines a constant Bits value set to zero.
    pub const ZERO: Self = Self { val: 0 };
    /// The number of bits in this [Bits] value.
    pub const fn len(&self) -> usize {
        N
    }
    /// Return true if the [Bits] value has zero bits.
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
    /// Return a [Bits] value with all bits set to 1.
    /// ```
    /// # use rhdl_bits::{Bits};
    /// let bits = Bits::<8>::mask();
    /// assert_eq!(bits, 0xFF);
    /// ```
    pub const fn mask() -> Self {
        Self {
            val: u128::MAX >> (128 - { N }),
        }
    }
    /// Resize the [Bits] value to a different size.
    /// If the new size is smaller, the value is truncated.
    /// If the new size is larger, the value is zero-extended.
    /// ```
    /// # use rhdl_bits::{Bits, bits};
    /// let bits: Bits<8> = bits(0b1101_1010);
    /// let new_bits: Bits<4> = bits.resize();
    /// assert_eq!(new_bits, 0b1010);
    /// let new_bits: Bits<16> = bits.resize();
    /// assert_eq!(new_bits, 0b0000_0000_1101_1010);
    /// ```
    pub const fn resize<const M: usize>(self) -> Bits<M>
    where
        W<M>: BitWidth,
    {
        let mask = Bits::<M>::mask();
        bits(self.val & mask.val & Self::mask().val)
    }
    /// Reinterpret the [Bits] value as a [SignedBits] value.
    pub const fn as_signed(self) -> SignedBits<N> {
        // Need to a sign extension here.
        if self.val & (1_u128 << ({ N } - 1)) != 0 {
            signed((self.val | !(Self::mask().val)) as i128)
        } else {
            signed(self.val as i128)
        }
    }
    /// Extract the raw `u128` behind the [Bits] value.
    pub const fn raw(self) -> u128 {
        self.val
    }
    /// Convert the compile-time sized [Bits] to a run-time
    /// tracked [DynBits] value.
    pub const fn dyn_bits(self) -> DynBits {
        DynBits {
            val: self.val,
            bits: N,
        }
        .wrapped()
    }
    /// Build a (dynamic, stack allocated) vector containing
    /// the bits that make up this value.  This will be slow.
    /// Not available in synthesizable functions.
    pub fn to_bools(self) -> Vec<bool> {
        let mut v = Vec::with_capacity(N);
        let mut x = self.val;
        for _i in 0..N {
            v.push(x & 1 == 1);
            x = x.wrapping_shr(1);
        }
        v
    }
    /// Return true if any bit is set.
    /// Available in synthesizable functions.
    pub fn any(self) -> bool {
        (self.val & Self::mask().val) != 0
    }
    /// Return true if all bits are set.
    /// Available in synthesizable functions.
    pub fn all(self) -> bool {
        (self.val & Self::mask().val) == Self::mask().val
    }
    /// Return true if an odd number of bits are set.
    /// Available in synthesizable functions.
    pub fn xor(self) -> bool {
        let mut x = self.val & Self::mask().val;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x ^= x >> 64;
        x & 1 == 1
    }
    /// Shift left by a constant amount, returning a [DynBits] value.
    /// The output size is increased by the shift amount.
    /// Available in synthesizable functions.
    pub fn xshl<const M: usize>(self) -> DynBits {
        self.dyn_bits().xshl::<M>()
    }
    /// Shift right by a constant amount, returning a [DynBits] value.
    /// The output size is decreased by the shift amount.
    /// Available in synthesizable functions.
    pub fn xshr<const M: usize>(self) -> DynBits {
        self.dyn_bits().xshr::<M>()
    }
    /// Pad the [Bits] value to a larger size, returning a [DynBits] value.
    /// The output size is increased by M bits.
    /// Available in synthesizable functions.
    pub fn xext<const M: usize>(self) -> DynBits {
        self.dyn_bits().xext::<M>()
    }
}

/// The default value for a [Bits] value is 0.
impl<const N: usize> Default for Bits<N>
where
    W<N>: BitWidth,
{
    fn default() -> Self {
        Self::ZERO
    }
}

/// Provide conversion from a `u128` to a [Bits] value.
/// This will panic if you try to convert a value that
/// is larger than the [Bits] value can hold.
impl<const N: usize> From<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    fn from(value: u128) -> Self {
        assert!(value <= Self::mask().val);
        Self { val: value }
    }
}

impl<const N: usize> PartialEq<Bits<N>> for u128
where
    W<N>: BitWidth,
{
    fn eq(&self, other: &Bits<N>) -> bool {
        other.val == bits::<N>(*self).val
    }
}

impl<const N: usize> PartialEq<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    fn eq(&self, other: &u128) -> bool {
        self.val == bits::<N>(*other).val
    }
}

impl<const N: usize> PartialOrd<Bits<N>> for u128
where
    W<N>: BitWidth,
{
    fn partial_cmp(&self, other: &Bits<N>) -> Option<std::cmp::Ordering> {
        let self_as_bits = bits::<N>(*self);
        self_as_bits.val.partial_cmp(&other.val)
    }
}

impl<const N: usize> PartialOrd<u128> for Bits<N>
where
    W<N>: BitWidth,
{
    fn partial_cmp(&self, other: &u128) -> Option<std::cmp::Ordering> {
        let other_as_bits = bits::<N>(*other);
        self.val.partial_cmp(&other_as_bits.val)
    }
}

impl<T, const N: usize, const M: usize> std::ops::Index<Bits<N>> for [T; M]
where
    W<N>: BitWidth,
{
    type Output = T;
    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self[index.val as usize]
    }
}

impl<T, const N: usize, const M: usize> std::ops::IndexMut<Bits<N>> for [T; M]
where
    W<N>: BitWidth,
{
    fn index_mut(&mut self, index: Bits<N>) -> &mut Self::Output {
        &mut self[index.val as usize]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mask() {
        let bits = Bits::<128>::mask();
        assert_eq!(bits.val, u128::MAX);
        let bits = Bits::<32>::mask();
        assert_eq!(bits.val, 0xFFFF_FFFF_u128);
    }

    #[test]
    fn test_binary_format() {
        let bits: Bits<8> = 0b1101_1010.into();
        assert_eq!(format!("{bits:b}"), "8'b11011010");
    }

    #[test]
    fn test_hex_format() {
        let bits: Bits<8> = 0b1101_1010.into();
        assert_eq!(format!("{bits:x}"), "8'hda");
        assert_eq!(format!("{bits:X}"), "8'HDA");
    }

    #[test]
    fn test_to_bits_method() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits.to_bools();
        assert_eq!(
            result,
            vec![false, true, false, true, true, false, true, true]
        );
    }

    #[test]
    fn test_self_cast() {
        let bits: Bits<8> = 0b1101_1010.into();
        let new_bits: Bits<8> = bits.resize();
        assert_eq!(new_bits, 0b1101_1010);
        let new_bits = bits.resize::<4>();
        assert_eq!(new_bits, 0b1010);
        let new_bits = bits.resize::<16>();
        assert_eq!(new_bits, 0b0000_0000_1101_1010);
    }

    const OPT1: Bits<8> = b8(0b1101_1010);
    const OPT2: Bits<8> = b8(0b0010_0100);

    #[test]
    fn test_match() {
        let bits: Bits<8> = 0b1101_1010.into();
        match bits {
            OPT1 => {
                eprintln!("Matched");
            }
            OPT2 => {
                panic!("Did not match");
            }
            _ => {
                panic!("Did not match");
            }
        }
    }

    #[test]
    fn test_cmp() {
        let a = b8(32);
        let b = b8(64);
        assert!(a < b);
    }

    #[test]
    fn test_xext() {
        for i in 0..=u8::MAX {
            let a = b8(i as u128);
            let b = a.dyn_bits().xext::<1>();
            assert_eq!(b, b9(i as u128).dyn_bits());
        }
    }

    #[test]
    fn test_xshl() {
        for i in 0..=u8::MAX {
            let a = b8(i as u128);
            let b = a.dyn_bits().xshl::<1>();
            assert_eq!(b, b9((i as u128) << 1).dyn_bits());
        }
    }

    #[test]
    fn test_xshr() {
        for i in 0..=u8::MAX {
            let a = b8(i as u128);
            let b = a.dyn_bits().xshr::<1>();
            assert_eq!(b, b7((i as u128) >> 1).dyn_bits());
        }
    }
}
