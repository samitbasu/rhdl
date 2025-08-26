#![allow(non_camel_case_types)]
use crate::rhdl_bits::bitwidth::*;
use std::ops::{Add, Sub};

use super::{bits, bits_impl::bits_masked, signed_dyn_bits::SignedDynBits, BitWidth, Bits};
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
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct SignedBits<Len: BitWidth> {
    pub(crate) marker: std::marker::PhantomData<Len>,
    pub(crate) val: i128,
}

impl<Len: BitWidth> std::cmp::PartialOrd for SignedBits<Len> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.val.cmp(&other.val))
    }
}

impl<Len: BitWidth> std::cmp::Ord for SignedBits<Len> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl<Len: BitWidth> std::fmt::Display for SignedBits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sd{}", Len::BITS, -self.val)
        } else {
            write!(f, "{}'sd{}", Len::BITS, self.val)
        }
    }
}

impl<Len: BitWidth> std::fmt::LowerHex for SignedBits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sh{:x}", Len::BITS, -self.val)
        } else {
            write!(f, "{}'sh{:x}", Len::BITS, self.val)
        }
    }
}

impl<Len: BitWidth> std::fmt::UpperHex for SignedBits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sH{:X}", Len::BITS, -self.val)
        } else {
            write!(f, "{}'sH{:X}", Len::BITS, self.val)
        }
    }
}

impl<Len: BitWidth> std::fmt::Binary for SignedBits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.val < 0 {
            write!(f, "-{}'sb{:b}", Len::BITS, -self.val)
        } else {
            write!(f, "{}'sb{:b}", Len::BITS, self.val)
        }
    }
}

seq!(N in 2..=128 {
    #(
        pub type s~N = SignedBits<U~N>;
        pub const fn s~N(value: i128) -> s~N {
            signed::<U~N>(value)
        }
    )*
});

/// Helper function for creating a signed bits value
/// from a constant.
/// ```
/// # use rhdl::bits::{consts::U8, SignedBits, signed, alias::b8};
/// let value : SignedBits<U8> = b8(0b1010_1010).as_signed();
/// assert_eq!(value.raw(), -86);
/// ```
/// Because the function is `const`, you can use it a constant
/// context:
/// ```
/// # use rhdl::bits::{consts::U8, SignedBits, signed, alias::b8};
/// const VALUE : SignedBits<U8> = b8(0b1010_1010).as_signed();
/// assert_eq!(VALUE.raw(), -86);
/// ```
pub const fn signed<N: BitWidth>(val: i128) -> SignedBits<N> {
    assert!(val <= SignedBits::<N>::max_value());
    assert!(val >= SignedBits::<N>::min_value());
    SignedBits {
        marker: std::marker::PhantomData,
        val,
    }
}

pub const fn signed_wrapped<N: BitWidth>(val: i128) -> SignedBits<N> {
    bits_masked::<N>(val as u128).as_signed()
}

pub struct signed<N: BitWidth> {
    marker: std::marker::PhantomData<N>,
}

impl<N: BitWidth> SignedBits<N> {
    pub const MAX: Self = Self {
        marker: std::marker::PhantomData,
        val: Self::max_value(),
    };
    pub const MIN: Self = Self {
        marker: std::marker::PhantomData,
        val: Self::min_value(),
    };
    /// Return a [SignedBits] value with all bits set to 1.
    pub const ZERO: Self = Self {
        marker: std::marker::PhantomData,
        val: 0,
    };
    /// Return the largest positive value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl::bits::{consts::U8, SignedBits};
    /// assert_eq!(SignedBits::<U8>::max_value(), i8::MAX as i128);
    /// ```
    pub const fn max_value() -> i128 {
        // The maximum value for an i128 is 0x7FF..FF
        // Each bit less in the representation reduces this by 2x
        i128::MAX >> (128 - N::BITS)
    }
    /// Return the smallest negative value that can be represented
    /// by this sized [SignedBits] value.
    /// ```
    /// # use rhdl::bits::{consts::U8, SignedBits};
    /// assert_eq!(SignedBits::<U8>::min_value(), i8::MIN as i128);
    /// ```
    pub const fn min_value() -> i128 {
        i128::MIN >> (128 - N::BITS)
    }
    /// Test if the value is negative.
    /// ```
    /// # use rhdl::bits::{consts::U8, signed};
    /// assert!(signed::<U8>(-1).is_negative());
    /// assert!(!signed::<U8>(0).is_negative());
    /// assert!(!signed::<U8>(1).is_negative());
    /// ```
    pub const fn is_negative(&self) -> bool {
        self.val < 0
    }
    /// Test if the value is positive or zero.
    /// ```
    /// # use rhdl::bits::{consts::U8, signed};
    /// assert!(!signed::<U8>(-1).is_non_negative());
    /// assert!(signed::<U8>(0).is_non_negative());
    /// assert!(signed::<U8>(1).is_non_negative());
    /// ```
    pub const fn is_non_negative(&self) -> bool {
        self.val >= 0
    }
    /// Reinterpret the [SignedBits] value as an unsigned
    /// [Bits] value.  This is useful for performing
    /// bit manipulations on the value that may or not
    /// preserve the 2's complement nature of the value.
    /// ```
    /// # use rhdl::bits::{consts::U8, Bits, SignedBits, signed};
    /// let x = signed::<U8>(-14); // In binary: 1111_0010
    /// let y : Bits<U8> = x.as_unsigned();
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
            bits: N::BITS,
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
    /// Sign extend a value by the number of bits specified.
    pub const fn xext<M>(self) -> SignedBits<op!(N + M)>
    where
        N: Add<M>,
        op!(N + M): BitWidth,
    {
        signed_wrapped::<op!(N + M)>(self.val)
    }
    pub const fn xshr<M>(self) -> SignedBits<op!(N - M)>
    where
        N: Sub<M>,
        M: BitWidth,
        op!(N - M): BitWidth,
    {
        signed_wrapped::<op!(N - M)>(self.val >> M::BITS)
    }
    pub const fn xshl<M>(self) -> SignedBits<op!(N + M)>
    where
        N: Add<M>,
        M: BitWidth,
        op!(N + M): BitWidth,
    {
        signed_wrapped::<op!(N + M)>(self.val << M::BITS)
    }
    pub const fn resize<M: BitWidth>(self) -> SignedBits<M> {
        if M::BITS > N::BITS {
            SignedBits {
                marker: std::marker::PhantomData,
                val: self.val,
            }
        } else {
            self.as_unsigned().resize::<M>().as_signed()
        }
    }
}

impl<N: BitWidth> Default for SignedBits<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<N: BitWidth> PartialEq<i128> for SignedBits<N> {
    fn eq(&self, other: &i128) -> bool {
        self.val == signed::<N>(*other).val
    }
}

impl<N: BitWidth> PartialEq<SignedBits<N>> for i128 {
    fn eq(&self, other: &SignedBits<N>) -> bool {
        signed::<N>(*self).val == other.val
    }
}

impl<N: BitWidth> PartialOrd<i128> for SignedBits<N> {
    fn partial_cmp(&self, other: &i128) -> Option<std::cmp::Ordering> {
        let other_as_bits = signed::<N>(*other);
        self.val.partial_cmp(&other_as_bits.val)
    }
}

impl<N: BitWidth> PartialOrd<SignedBits<N>> for i128 {
    fn partial_cmp(&self, other: &SignedBits<N>) -> Option<std::cmp::Ordering> {
        let self_as_bits = signed::<N>(*self);
        self_as_bits.val.partial_cmp(&other.val)
    }
}

// Provide conversion from a `i128` to a [SignedBits] value.
// This will panic if you try to convert a value that
// is larger than the [SignedBits] value can hold.
impl<N: BitWidth> From<i128> for SignedBits<N> {
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
        println!("SignedBits::<U8>(-1) = {:x}", signed::<U8>(-1));
    }

    #[test]
    fn test_set_bit_for_signed_values() {
        for bit in 0..8 {
            let mut value = SignedBits::<U8>::from(0);
            value = crate::rhdl_bits::test::set_bit(value.as_unsigned(), bit, true).as_signed();
            assert_eq!(value, i8::wrapping_shl(1, bit as u32) as i128);
        }
    }

    #[test]
    fn test_sign_test_is_correct() {
        assert!(signed::<U8>(-1).is_negative());
        assert!(!signed::<U8>(-1).is_non_negative());
        assert!(!signed::<U8>(0).is_negative());
        assert!(signed::<U8>(0).is_non_negative());
        assert!(!signed::<U8>(1).is_negative());
        assert!(signed::<U8>(1).is_non_negative());
    }

    #[test]
    fn test_max_value_is_correct() {
        assert_eq!(SignedBits::<U8>::max_value(), i8::MAX as i128);
        assert_eq!(SignedBits::<U16>::max_value(), i16::MAX as i128);
        assert_eq!(SignedBits::<U32>::max_value(), i32::MAX as i128);
        assert_eq!(SignedBits::<U64>::max_value(), i64::MAX as i128);
        assert_eq!(SignedBits::<U128>::max_value(), i128::MAX);
        assert_eq!(SignedBits::<U12>::max_value(), 0b0111_1111_1111);
    }

    #[test]
    fn test_min_value_is_correct() {
        assert_eq!(SignedBits::<U8>::min_value(), i8::MIN as i128);
        assert_eq!(SignedBits::<U16>::min_value(), i16::MIN as i128);
        assert_eq!(SignedBits::<U32>::min_value(), i32::MIN as i128);
        assert_eq!(SignedBits::<U64>::min_value(), i64::MIN as i128);
        assert_eq!(SignedBits::<U128>::min_value(), i128::MIN);
        assert_eq!(SignedBits::<U12>::min_value(), -0b1000_0000_0000);
    }

    #[test]
    #[should_panic]
    fn test_overflow_causes_panic() {
        let _ = signed::<U8>(128);
    }

    #[test]
    #[should_panic]
    fn test_underflow_causes_panic() {
        let _ = signed::<U8>(-129);
    }

    #[test]
    fn test_signed_cast() {
        let value = signed::<U8>(-14);
        let extended = value.resize::<U16>();
        assert_eq!(extended, -14);
        let value = signed::<U8>(-86);
        let truncated = value.resize::<U4>();
        assert_eq!(truncated, -6);
        let truncated = value.resize::<U5>();
        assert_eq!(truncated, 10);
        let value = signed::<U8>(3);
        let extended = value.resize::<U16>();
        assert_eq!(extended, 3);
    }

    #[test]
    fn test_comparison_signed() {
        let a1 = s8(-32);
        let b1 = s8(-24);
        assert!(a1 < b1);
    }

    const OPT1: SignedBits<U8> = s8(-0b0101_1010);
    const OPT2: SignedBits<U8> = s8(0b0010_0100);

    #[test]
    fn test_match_works() {
        let bits: SignedBits<U8> = (-0b101_1010).into();
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
            let b = a.xext::<U1>();
            assert_eq!(b, s9(i as i128));
        }
    }

    #[test]
    fn test_xshl() {
        for i in i8::MIN..=i8::MAX {
            let a = s8(i as i128);
            let b = a.xshl::<U1>();
            assert_eq!(b, s9((i as i128) << 1));
        }
    }

    #[test]
    fn test_xshr() {
        for i in i8::MIN..=i8::MAX {
            let a = s8(i as i128);
            let b = a.xshr::<U1>();
            assert_eq!(b, s7((i as i128) >> 1));
        }
    }
}
