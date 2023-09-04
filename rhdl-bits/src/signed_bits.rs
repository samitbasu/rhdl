use crate::Bits;
use derive_more::{
    AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, SubAssign,
};
use std::fmt::{Binary, Display, Formatter, LowerHex, UpperHex};

// The [SignedBits] type is a fixed-size bit vector.  It is
// meant to imitate the behavior of signed bit vectors in hardware.
// Due to the design of the [SignedBits] type, you can only create a
// signed bit vector of up to 128 bits in lnegth for now.  However,
// you can easily express larger constructs in hardware using arrays,
// tuples and structs.  The only real limitation of the [SignedBits]
// type being 128 bits is that you cannot perform arbitrary arithmetic
// on longer bit values in your hardware designs.
//
// Signed arithmetic is performed using 2's complement arithmetic.
// See [https://en.wikipedia.org/wiki/Two%27s_complement] for more
// information.
//
// Note that unlike the [Bits] type, comparisons are performed using
// signed arithmetic.  Note also that the right shift operator when
// applied to a signed value will sign extend the value.  This is
// the same behavior as is seen in Rust (i.e., ((-4) >> 2) == -2).
//
// If you want to right shift a signed value without sign extension,
// then you should convert it to a [Bits] type first.
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
    AddAssign,
    SubAssign,
)]
#[repr(transparent)]
pub struct SignedBits<const N: usize>(pub(crate) i128);

impl<const N: usize> LowerHex for SignedBits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl<const N: usize> UpperHex for SignedBits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::UpperHex::fmt(&self.0, f)
    }
}

impl<const N: usize> Binary for SignedBits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Binary::fmt(&self.0, f)
    }
}

impl<const N: usize> Display for SignedBits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<const N: usize> SignedBits<N> {
    // Return a [SignedBits] value with all bits set to 1.
    pub fn mask() -> Self {
        // Do not compute this as you will potentially
        // cause overflow.
        if N < 128 {
            Self((1 << N) - 1)
        } else {
            Self(-1)
        }
    }
    // Extract the sign bit from the [SignedBits] value.
    pub fn sign_bit(&self) -> bool {
        self.get_bit(N - 1)
    }
    // Return the largest positive value that can be represented
    // by this sized [SignedBits] value.
    pub fn max_value() -> i128 {
        ((Self::mask().0 as u128) >> 1) as i128
    }
    // Return the smallest negative value that can be represented
    // by this sized [SignedBits] value.
    pub fn min_value() -> i128 {
        (-1) << (N - 1)
    }
    // Set a specific bit of a [SignedBits] value to 1 or 0.
    // Note that changing the MSB of a signed bit vector changes
    // the sign of that vector.
    pub fn set_bit(&mut self, bit: usize, value: bool) {
        assert!(bit < N);
        if value {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }
    // Get the value of a specific bit of a [SignedBits] value.
    pub fn get_bit(&self, bit: usize) -> bool {
        assert!(bit < N);
        (self.0 & (1 << bit)) != 0
    }
    // Returns true if any of the bits are set to 1.
    pub fn any(self) -> bool {
        (self.0 & Self::mask().0) != 0
    }
    // Returns true if all of the bits are set to 1.
    pub fn all(self) -> bool {
        (self.0 & Self::mask().0) == Self::mask().0
    }
    // Test if the value is negative.
    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }
    // Test if the value is positive or zero.
    pub fn is_non_negative(&self) -> bool {
        self.0 >= 0
    }
    // Computes the xor of all of the bits in the value.
    pub fn xor(self) -> bool {
        let mut x = self.0 & Self::mask().0;
        x ^= x >> 64;
        x ^= x >> 32;
        x ^= x >> 16;
        x ^= x >> 8;
        x ^= x >> 4;
        x ^= x >> 2;
        x ^= x >> 1;
        (x & 1) != 0
    }
    // Extracts a range of bits from the SignedBits value.
    // Because we cannot guarantee that the sliced bits
    // include the proper 2's complement representation for
    // a signed value, they are simple a [Bits] vector.
    pub fn slice<const M: usize>(&self, start: usize) -> Bits<M> {
        Bits(((self.0 >> start) as u128) & crate::Bits::<M>::mask().0)
    }
    pub fn as_unsigned(self) -> Bits<N> {
        Bits(self.0 as u128 & Bits::<N>::mask().0)
    }
}

impl<const N: usize> Default for SignedBits<N> {
    fn default() -> Self {
        Self(0)
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
}
