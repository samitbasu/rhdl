#![allow(non_camel_case_types)]
use crate::signed_bits_impl::SignedBits;
use derive_more::{
    Binary, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Display, LowerHex,
    UpperHex,
};
use seq_macro::seq;
use typenum::consts::*;
use typenum::{IsLessOrEqual, NonZero, Unsigned, U128};

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
#[derive(
    Clone,
    Debug,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    /*     BitAnd,
       BitAndAssign,
       BitOr,
       BitOrAssign,
       BitXor,
       BitXorAssign,
    Display,
    LowerHex,
    UpperHex,
    Binary,
    */
)]
//pub struct Bits<const N: usize>(pub u128);
pub struct Bits<Len>
where
    Len: Unsigned,
    Len: NonZero,
    Len: IsLessOrEqual<U128>,
{
    marker: std::marker::PhantomData<Len>,
    val: u128,
}

seq!(N in 1..=128 {
    #(
        pub type b~N = Bits<U~N>;
        pub fn b~N(value: u128) -> b~N {
            b~N::from(value)
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
pub const fn bits<const N: usize>(value: u128) -> Bits<N> {
    assert!(N <= 128);
    assert!(value <= Bits::<N>::mask().0);
    Bits(value)
}

pub struct bits<const N: usize> {}

impl<const N: usize> Bits<N> {
    /// Defines a constant Bits value with all bits set to 1.
    pub const MASK: Self = Self::mask();
    pub const ZERO: Self = Self(0);
    /// Return a [Bits] value with all bits set to 1.
    /// ```
    /// # use rhdl_bits::Bits;
    /// let bits = Bits::<8>::mask();
    /// assert_eq!(bits, 0xFF);
    /// ```
    pub const fn mask() -> Self {
        // Do not compute this as you will potentially
        // cause overflow.
        if N < 128 {
            Self((1 << N) - 1)
        } else {
            Self(u128::MAX)
        }
    }
    pub const fn resize<const M: usize>(self) -> Bits<M> {
        assert!(M <= 128);
        assert!(N <= 128);
        let mask = Bits::<M>::mask();
        bits(self.0 & mask.0 & Self::mask().0)
    }
    /// Reinterpret the [Bits] value as a [SignedBits] value.
    pub const fn as_signed(self) -> SignedBits<N> {
        // Need to a sign extension here.
        if self.0 & (1_u128 << (N - 1)) != 0 {
            SignedBits((self.0 | !(Self::mask().0)) as i128)
        } else {
            SignedBits(self.0 as i128)
        }
    }
    /// Extract the raw `u128` behind the [Bits] value.
    pub fn raw(self) -> u128 {
        self.0
    }
    /// Build a (dynamic, stack allocated) vector containing
    /// the bits that make up this value.  This will be slow.
    pub fn to_bools(self) -> Vec<bool> {
        let mut v = Vec::with_capacity(N);
        let mut x = self.0;
        for _i in 0..N {
            v.push(x & 1 == 1);
            x = x.wrapping_shr(1);
        }
        v
    }
    pub fn any(self) -> bool {
        (self.0 & Self::mask()) != 0
    }
    pub fn all(self) -> bool {
        (self.0 & Self::mask()) == Self::mask().0
    }
    pub fn xor(self) -> bool {
        let mut x = (self.0 & Self::mask()).0;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x & 1 == 1
    }
}

trait ExtendBy<const N: usize> {
    type Output;

    fn extend_by(self) -> Self::Output;
}

impl ExtendBy<1> for Bits<1> {
    type Output = Bits<2>;

    fn extend_by(self) -> Self::Output {
        self.resize::<2>()
    }
}

/// The default value for a [Bits] value is 0.
impl<const N: usize> Default for Bits<N> {
    fn default() -> Self {
        Self(0)
    }
}

/// Provide conversion from a `u128` to a [Bits] value.
/// This will panic if you try to convert a value that
/// is larger than the [Bits] value can hold.
impl<const N: usize> From<u128> for Bits<N> {
    fn from(value: u128) -> Self {
        assert!(N <= 128);
        assert!(value <= Self::mask().0);
        Self(value)
    }
}

impl<const N: usize> PartialEq<Bits<N>> for u128 {
    fn eq(&self, other: &Bits<N>) -> bool {
        other == &Bits::from(*self)
    }
}

impl<const N: usize> PartialEq<u128> for Bits<N> {
    fn eq(&self, other: &u128) -> bool {
        self == &Self::from(*other)
    }
}

impl<const N: usize> PartialOrd<Bits<N>> for u128 {
    fn partial_cmp(&self, other: &Bits<N>) -> Option<std::cmp::Ordering> {
        other.partial_cmp(&Bits::from(*self))
    }
}

impl<const N: usize> PartialOrd<u128> for Bits<N> {
    fn partial_cmp(&self, other: &u128) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl<T, const M: usize, const N: usize> std::ops::Index<Bits<N>> for [T; M] {
    type Output = T;
    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl<T, const M: usize, const N: usize> std::ops::IndexMut<Bits<N>> for [T; M] {
    fn index_mut(&mut self, index: Bits<N>) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask() {
        let bits = Bits::<128>::mask();
        assert_eq!(bits.0, u128::MAX);
        let bits = Bits::<32>::mask();
        assert_eq!(bits.0, 0xFFFF_FFFF_u128);
    }

    #[test]
    fn test_binary_format() {
        let bits: Bits<8> = 0b1101_1010.into();
        assert_eq!(format!("{:b}", bits), "11011010");
    }

    #[test]
    fn test_hex_format() {
        let bits: Bits<8> = 0b1101_1010.into();
        assert_eq!(format!("{:x}", bits), "da");
        assert_eq!(format!("{:X}", bits), "DA");
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
}
