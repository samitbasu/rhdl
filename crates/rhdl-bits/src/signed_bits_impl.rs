#![allow(non_camel_case_types)]
use super::{BitWidth, Bits, bits, bits_impl::bits_masked, signed_dyn_bits::SignedDynBits};
use crate::bitwidth::W;
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
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd, Ord, Eq)]
pub struct SignedBits<const N: usize>
where
    W<N>: BitWidth,
{
    pub val: i128,
}

impl<const N: usize> std::fmt::Display for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sd{}", { N }, -self.val)
        } else {
            write!(f, "{}'sd{}", { N }, self.val)
        }
    }
}

impl<const N: usize> std::fmt::LowerHex for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sh{:x}", { N }, -self.val)
        } else {
            write!(f, "{}'sh{:x}", { N }, self.val)
        }
    }
}

impl<const N: usize> std::fmt::UpperHex for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sH{:X}", { N }, -self.val)
        } else {
            write!(f, "{}'sH{:X}", { N }, self.val)
        }
    }
}

impl<const N: usize> std::fmt::Binary for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sb{:b}", { N }, -self.val)
        } else {
            write!(f, "{}'sb{:b}", { N }, self.val)
        }
    }
}

seq!(N in 2..=128 {
    #(
        pub type s~N = SignedBits<N>;
        pub const fn s~N(value: i128) -> s~N {
            signed::<N>(value)
        }
    )*
});

/// Helper function for creating a signed bits value
/// from a constant.
/// ```
/// # use rhdl_bits::{consts::U8, SignedBits, signed, alias::b8};
/// let value : SignedBits<8> = b8(0b1010_1010).as_signed();
/// assert_eq!(value.raw(), -86);
/// ```
/// Because the function is `const`, you can use it a constant
/// context:
/// ```
/// # use rhdl_bits::{consts::U8, SignedBits, signed, alias::b8};
/// const VALUE : SignedBits<8> = b8(0b1010_1010).as_signed();
/// assert_eq!(VALUE.raw(), -86);
/// ```
pub const fn signed<const N: usize>(val: i128) -> SignedBits<N>
where
    W<N>: BitWidth,
{
    assert!(val <= SignedBits::<N>::max_value());
    assert!(val >= SignedBits::<N>::min_value());
    SignedBits { val }
}

pub const fn signed_wrapped<const N: usize>(val: i128) -> SignedBits<N>
where
    W<N>: BitWidth,
{
    bits_masked::<N>(val as u128).as_signed()
}

pub struct signed<const N: usize>
where
    W<N>: BitWidth, {}

impl<const N: usize> SignedBits<N>
where
    W<N>: BitWidth,
{
    pub const MAX: Self = Self {
        val: Self::max_value(),
    };
    pub const MIN: Self = Self {
        val: Self::min_value(),
    };
    /// Return a [SignedBits] value with all bits set to 1.
    pub const ZERO: Self = Self { val: 0 };
    /// Return the largest positive value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl_bits::{consts::U8, SignedBits};
    /// assert_eq!(SignedBits::<8>::max_value(), i8::MAX as i128);
    /// ```
    pub const fn max_value() -> i128 {
        // The maximum value for an i128 is 0x7FF..FF
        // Each bit less in the representation reduces this by 2x
        i128::MAX >> (128 - { N })
    }
    /// Return the smallest negative value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl_bits::{consts::U8, SignedBits};
    /// assert_eq!(SignedBits::<8>::min_value(), i8::MIN as i128);
    /// ```
    pub const fn min_value() -> i128 {
        i128::MIN >> (128 - { N })
    }
    /// Test if the value is negative.
    /// ```
    /// # use rhdl_bits::{consts::U8, signed};
    /// assert!(signed::<8>(-1).is_negative());
    /// assert!(!signed::<8>(0).is_negative());
    /// assert!(!signed::<8>(1).is_negative());
    /// ```
    pub const fn is_negative(&self) -> bool {
        self.val < 0
    }
    /// Test if the value is positive or zero.
    /// ```
    /// # use rhdl_bits::{consts::U8, signed};
    /// assert!(!signed::<8>(-1).is_non_negative());
    /// assert!(signed::<8>(0).is_non_negative());
    /// assert!(signed::<8>(1).is_non_negative());
    /// ```
    pub const fn is_non_negative(&self) -> bool {
        self.val >= 0
    }
    /// Reinterpret the [SignedBits] value as an unsigned
    /// [Bits] value.  This is useful for performing
    /// bit manipulations on the value that may or not
    /// preserve the 2's complement nature of the value.
    /// ```
    /// # use rhdl_bits::{consts::U8, Bits, SignedBits, signed};
    /// let x = signed::<8>(-14); // In binary: 1111_0010
    /// let y : Bits<8> = x.as_unsigned();
    /// assert_eq!(y, 0b1111_0010);
    /// ```
    pub const fn as_unsigned(self) -> Bits<N> {
        bits(self.val as u128 & Bits::<N>::mask().raw())
    }
    /// Extract the raw signed `i128` backing this SignedBits
    /// value.
    pub const fn raw(self) -> i128 {
        self.val
    }
    /// Convert the compile time sized [SignedBits] value
    /// to a run-time traced [SignedDynBits] value.
    pub const fn dyn_bits(self) -> SignedDynBits {
        SignedDynBits {
            val: self.val,
            bits: { N },
        }
    }
    /// Build a (dynamic, stack allocated) vector
    /// containing the bits that make up this value.
    /// This will be slow.
    pub fn to_bools(self) -> Vec<bool> {
        self.as_unsigned().to_bools()
    }
    pub const fn any(self) -> bool {
        self.val != 0
    }
    pub const fn all(self) -> bool {
        self.val == -1
    }
    pub const fn xor(self) -> bool {
        let mut x = self.val;
        x ^= x >> 1;
        x ^= x >> 2;
        x ^= x >> 4;
        x ^= x >> 8;
        x ^= x >> 16;
        x ^= x >> 32;
        x ^= x >> 64;
        x & 1 == 1
    }
    pub const fn resize<const M: usize>(self) -> SignedBits<M>
    where
        W<M>: BitWidth,
    {
        if { M } > { N } {
            SignedBits { val: self.val }
        } else {
            self.as_unsigned().resize::<M>().as_signed()
        }
    }
    pub fn xshl<const M: usize>(self) -> SignedDynBits {
        self.dyn_bits().xshl::<M>()
    }
    pub fn xshr<const M: usize>(self) -> SignedDynBits {
        self.dyn_bits().xshr::<M>()
    }
    pub fn xext<const M: usize>(self) -> SignedDynBits {
        self.dyn_bits().xext::<M>()
    }
}

impl<const N: usize> Default for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const N: usize> PartialEq<i128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn eq(&self, other: &i128) -> bool {
        self.val == signed::<N>(*other).val
    }
}

impl<const N: usize> PartialEq<SignedBits<N>> for i128
where
    W<N>: BitWidth,
{
    fn eq(&self, other: &SignedBits<N>) -> bool {
        signed::<N>(*self).val == other.val
    }
}

impl<const N: usize> PartialOrd<i128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn partial_cmp(&self, other: &i128) -> Option<std::cmp::Ordering> {
        let other_as_bits = signed::<N>(*other);
        self.val.partial_cmp(&other_as_bits.val)
    }
}

impl<const N: usize> PartialOrd<SignedBits<N>> for i128
where
    W<N>: BitWidth,
{
    fn partial_cmp(&self, other: &SignedBits<N>) -> Option<std::cmp::Ordering> {
        let self_as_bits = signed::<N>(*self);
        self_as_bits.val.partial_cmp(&other.val)
    }
}

// Provide conversion from a `i128` to a [SignedBits] value.
// This will panic if you try to convert a value that
// is larger than the [SignedBits] value can hold.
impl<const N: usize> From<i128> for SignedBits<N>
where
    W<N>: BitWidth,
{
    fn from(value: i128) -> Self {
        assert!(value <= Self::max_value());
        assert!(value >= Self::min_value());
        signed(value)
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
        println!("SignedBits::<8>(-1) = {:x}", signed::<8>(-1));
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
        assert!(signed::<8>(-1).is_negative());
        assert!(!signed::<8>(-1).is_non_negative());
        assert!(!signed::<8>(0).is_negative());
        assert!(signed::<8>(0).is_non_negative());
        assert!(!signed::<8>(1).is_negative());
        assert!(signed::<8>(1).is_non_negative());
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
        let _ = signed::<8>(128);
    }

    #[test]
    #[should_panic]
    fn test_underflow_causes_panic() {
        let _ = signed::<8>(-129);
    }

    #[test]
    fn test_signed_cast() {
        let value = signed::<8>(-14);
        let extended = value.resize::<16>();
        assert_eq!(extended, -14);
        let value = signed::<8>(-86);
        let truncated = value.resize::<4>();
        assert_eq!(truncated, -6);
        let truncated = value.resize::<5>();
        assert_eq!(truncated, 10);
        let value = signed::<8>(3);
        let extended = value.resize::<16>();
        assert_eq!(extended, 3);
    }

    #[test]
    fn test_comparison_signed() {
        let a1 = s8(-32);
        let b1 = s8(-24);
        assert!(a1 < b1);
    }

    const OPT1: SignedBits<8> = s8(-0b0101_1010);
    const OPT2: SignedBits<8> = s8(0b0010_0100);

    #[test]
    fn test_match_works() {
        let bits: SignedBits<8> = (-0b101_1010).into();
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
    fn test_xext() {
        for i in i8::MIN..=i8::MAX {
            let a = s8(i as i128);
            let b = a.dyn_bits().xext::<1>().as_signed_bits();
            assert_eq!(b, s9(i as i128));
        }
    }

    #[test]
    fn test_xshl() {
        for i in i8::MIN..=i8::MAX {
            let a = s8(i as i128);
            let b = a.dyn_bits().xshl::<1>().as_signed_bits();
            assert_eq!(b, s9((i as i128) << 1));
        }
    }

    #[test]
    fn test_xshr() {
        for i in i8::MIN..=i8::MAX {
            let a = s8(i as i128);
            let b = a.dyn_bits().xshr::<1>().as_signed_bits();
            assert_eq!(b, s7((i as i128) >> 1));
        }
    }
}
