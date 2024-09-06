#![allow(non_camel_case_types)]
use crate::{bits_impl::bit_cast, Bits};
use derive_more::{
    Binary, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Display, LowerHex,
    UpperHex,
};
use seq_macro::seq;

/// The [SignedBits] type is a fixed-size bit vector.  It is
/// meant to imitate the behavior of signed bit vectors in hardware.
/// Due to the design of the [SignedBits] type, you can only create a
/// signed bit vector of up to 128 bits in lnegth for now.  However,
/// you can easily express larger constructs in hardware using arrays,
/// tuples and structs.  The only real limitation of the [SignedBits]
/// type being 128 bits is that you cannot perform arbitrary arithmetic
/// on longer bit values in your hardware designs.
///
/// Signed arithmetic is performed using 2's complement arithmetic.
/// See <https://en.wikipedia.org/wiki/Two%27s_complement> for more
/// information.
///
/// Note that unlike the [Bits] type, comparisons are performed using
/// signed arithmetic.  Note also that the right shift operator when
/// applied to a signed value will sign extend the value.  This is
/// the same behavior as is seen in Rust (i.e., ((-4) >> 1) == -2).
///
/// If you want to right shift a signed value without sign extension,
/// then you should convert it to a [Bits] type first.
#[derive(
    Clone,
    Debug,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    BitXor,
    BitXorAssign,
    Binary,
    LowerHex,
    UpperHex,
    Display,
)]
#[repr(transparent)]
pub struct SignedBits<const N: usize>(pub i128);

seq!(N in 1..=128 {
    #(
        pub type s~N = SignedBits<N>;
        pub fn s~N(value: i128) -> s~N {
            s~N::from(value)
        }
    )*
});

/// Helper function for creating a signed bits value
/// from a constant.
/// ```
/// # use rhdl_bits::{SignedBits, signed};
/// let value : SignedBits<8> = signed(0b1010_1010);
/// assert_eq!(value, -86);
/// ```
/// Because the function is `const`, you can use it a constant
/// context:
/// ```
/// # use rhdl_bits::{SignedBits, signed};
/// const VALUE : SignedBits<8> = signed(0b1010_1010);
/// assert_eq!(VALUE, -86);
/// ```
pub const fn signed<const N: usize>(value: i128) -> SignedBits<N> {
    SignedBits(if (value & (1 << (N - 1))) != 0 {
        value | !(SignedBits::<N>::mask().0)
    } else {
        value
    })
}

pub struct signed<const N: usize> {}

/// Helper function to cast a signed bits value to a different
/// length using either truncation (if the new length is
/// shorter) or sign extension (if the new length is longer).
/// ```
/// # use rhdl_bits::{SignedBits, signed};
/// let value : SignedBits<8> = signed(0b1010_1010);
/// let extended : SignedBits<16> = signed_cast(value);
/// assert_eq!(extended, -86);
/// ```
/// Note that the value is sign extended.
/// ```
/// # use rhdl_bits::{SignedBits, signed};
/// let value : SignedBits<8> = signed(-86);
/// let truncated : SignedBits<4> = signed_cast(value);
/// assert_eq!(truncated, -6);
/// ```
/// Note that the value is truncated.
pub const fn signed_cast<const M: usize, const N: usize>(value: SignedBits<N>) -> SignedBits<M> {
    if M < N {
        bit_cast::<M, N>(value.as_unsigned()).as_signed()
    } else {
        SignedBits::<M>(value.0)
    }
}

pub struct signed_cast<const M: usize, const N: usize> {}

impl<const N: usize> SignedBits<N> {
    /// Return a [SignedBits] value with all bits set to 1.
    pub const MASK: Self = Self::mask();
    /// Return a [SignedBits] value with all bits set to 1.
    /// ```
    /// # use rhdl_bits::SignedBits;
    /// let mask = SignedBits::<8>::mask();
    /// assert_eq!(mask.as_unsigned(), 0xFF);
    /// ```
    /// Note that for a [SignedBits] value, the mask is the same
    /// as a representation of -1.
    pub const fn mask() -> Self {
        // Do not compute this as you will potentially
        // cause overflow.
        if N < 128 {
            Self((1 << N) - 1)
        } else {
            Self(-1)
        }
    }
    /// Return the largest positive value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl_bits::SignedBits;
    /// assert_eq!(SignedBits::<8>::max_value(), i8::MAX as i128);
    /// ```
    pub fn max_value() -> i128 {
        ((Self::mask().0 as u128) >> 1) as i128
    }
    /// Return the smallest negative value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl_bits::SignedBits;
    /// assert_eq!(SignedBits::<8>::min_value(), i8::MIN as i128);
    /// ```
    pub fn min_value() -> i128 {
        (-1) << (N - 1)
    }
    /// Test if the value is negative.
    /// ```
    /// # use rhdl_bits::signed;
    /// assert!(signed::<8>(-1).is_negative());
    /// assert!(!signed::<8>(0).is_negative());
    /// assert!(!signed::<8>(1).is_negative());
    /// ```
    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }
    /// Test if the value is positive or zero.
    /// ```
    /// # use rhdl_bits::signed;
    /// assert!(!signed::<8>(-1).is_non_negative());
    /// assert!(signed::<8>(0).is_non_negative());
    /// assert!(signed::<8>(1).is_non_negative());
    /// ```
    pub fn is_non_negative(&self) -> bool {
        self.0 >= 0
    }
    /// Reinterpret the [SignedBits] value as an unsigned
    /// [Bits] value.  This is useful for performing
    /// bit manipulations on the value that may or not
    /// preserve the 2's complement nature of the value.
    /// ```
    /// # use rhdl_bits::{Bits, SignedBits, signed};
    /// let x = signed::<8>(-14); // In binary: 1111_0010
    /// let y : Bits<8> = x.as_unsigned();
    /// assert_eq!(y, 0b1111_0010);
    /// ```
    pub const fn as_unsigned(self) -> Bits<N> {
        Bits(self.0 as u128 & Bits::<N>::mask().0)
    }
    /// Extract the raw signed `i128` backing this SignedBits
    /// value.
    pub fn raw(self) -> i128 {
        self.0
    }
    /// Build a (dynamic, stack allocated) vector
    /// containing the bits that make up this value.
    /// This will be slow.
    pub fn to_bools(self) -> Vec<bool> {
        self.as_unsigned().to_bools()
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

impl<const N: usize> Default for SignedBits<N> {
    fn default() -> Self {
        Self(0)
    }
}

impl<const N: usize> PartialEq<i128> for SignedBits<N> {
    fn eq(&self, other: &i128) -> bool {
        self == &Self::from(*other)
    }
}

impl<const N: usize> PartialEq<SignedBits<N>> for i128 {
    fn eq(&self, other: &SignedBits<N>) -> bool {
        &SignedBits::<N>::from(*self) == other
    }
}

impl<const N: usize> PartialOrd<i128> for SignedBits<N> {
    fn partial_cmp(&self, other: &i128) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::from(*other))
    }
}

impl<const N: usize> PartialOrd<SignedBits<N>> for i128 {
    fn partial_cmp(&self, other: &SignedBits<N>) -> Option<std::cmp::Ordering> {
        SignedBits::<N>::from(*self).partial_cmp(other)
    }
}

// Provide conversion from a `i128` to a [SignedBits] value.
// This will panic if you try to convert a value that
// is larger than the [SignedBits] value can hold.
impl<const N: usize> From<i128> for SignedBits<N> {
    fn from(value: i128) -> Self {
        assert!(N <= 128);
        assert!(value <= Self::max_value());
        assert!(value >= Self::min_value());
        Self(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rust_right_shift_preserves_sign() {
        assert_eq!((-4_i128) >> 2, -1);
    }

    #[test]
    fn test_display_for_signed_values() {
        println!("SignedBits::<8>(-1) = {:x}", SignedBits::<8>(-1));
    }

    #[test]
    fn test_set_bit_for_signed_values() {
        for bit in 0..8 {
            let mut value = SignedBits::<8>::from(0);
            value = crate::test::set_bit(value.as_unsigned(), bit, true).as_signed();
            assert_eq!(value, i8::wrapping_shl(1, bit as u32) as i128);
        }
    }

    #[test]
    fn test_sign_test_is_correct() {
        assert!(SignedBits::<8>(-1).is_negative());
        assert!(!SignedBits::<8>(-1).is_non_negative());
        assert!(!SignedBits::<8>(0).is_negative());
        assert!(SignedBits::<8>(0).is_non_negative());
        assert!(!SignedBits::<8>(1).is_negative());
        assert!(SignedBits::<8>(1).is_non_negative());
    }

    #[test]
    fn test_max_value_is_correct() {
        assert_eq!(SignedBits::<8>::max_value(), i8::MAX as i128);
        assert_eq!(SignedBits::<16>::max_value(), i16::MAX as i128);
        assert_eq!(SignedBits::<32>::max_value(), i32::MAX as i128);
        assert_eq!(SignedBits::<64>::max_value(), i64::MAX as i128);
        assert_eq!(SignedBits::<128>::max_value(), i128::MAX);
        assert_eq!(SignedBits::<12>::max_value(), 0b0111_1111_1111);
    }

    #[test]
    fn test_min_value_is_correct() {
        assert_eq!(SignedBits::<8>::min_value(), i8::MIN as i128);
        assert_eq!(SignedBits::<16>::min_value(), i16::MIN as i128);
        assert_eq!(SignedBits::<32>::min_value(), i32::MIN as i128);
        assert_eq!(SignedBits::<64>::min_value(), i64::MIN as i128);
        assert_eq!(SignedBits::<128>::min_value(), i128::MIN);
        assert_eq!(SignedBits::<12>::min_value(), -0b1000_0000_0000);
    }

    #[test]
    #[should_panic]
    fn test_overflow_causes_panic() {
        let _ = SignedBits::<8>::from(128);
    }

    #[test]
    #[should_panic]
    fn test_underflow_causes_panic() {
        let _ = SignedBits::<8>::from(-129);
    }

    #[test]
    fn test_signed_cast() {
        let value = SignedBits::<8>::from(-14);
        let extended = signed_cast::<16, 8>(value);
        assert_eq!(extended, -14);
        let value = SignedBits::<8>::from(-86);
        let truncated = signed_cast::<4, 8>(value);
        assert_eq!(truncated, -6);
        let truncated = signed_cast::<5, 8>(value);
        assert_eq!(truncated, 10);
        let value = SignedBits::<8>::from(3);
        let extended = signed_cast::<16, 8>(value);
        assert_eq!(extended, 3);
    }
}
