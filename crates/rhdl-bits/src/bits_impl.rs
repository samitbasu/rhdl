#![allow(non_camel_case_types)]
use std::ops::{Add, Sub};

use super::{BitWidth, dyn_bits::DynBits, signed, signed_bits_impl::SignedBits};
use crate::rhdl_bits::bitwidth::*;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bits<Len> {
    pub(crate) marker: std::marker::PhantomData<Len>,
    pub val: u128,
}

impl<Len: BitWidth> std::cmp::PartialOrd for Bits<Len> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.val.cmp(&other.val))
    }
}

impl<Len: BitWidth> std::cmp::Ord for Bits<Len> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }
}

impl<Len: BitWidth> std::fmt::Debug for Bits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}_b{}", self.val, Len::BITS)
    }
}

impl<Len: BitWidth> std::fmt::Display for Bits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'d{}", Len::BITS, self.val)
    }
}

impl<Len: BitWidth> std::fmt::LowerHex for Bits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'h{:x}", Len::BITS, self.val)
    }
}

impl<Len: BitWidth> std::fmt::UpperHex for Bits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'H{:X}", Len::BITS, self.val)
    }
}

impl<Len: BitWidth> std::fmt::Binary for Bits<Len> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}'b{:b}", Len::BITS, self.val)
    }
}

seq!(N in 1..=128 {
    #(
        pub type b~N = Bits<U~N>;
        pub const fn b~N(value: u128) -> b~N {
            bits::<U~N>(value)
        }
    )*
});

/// Helper function for creating a bits value from
/// a constant.
/// ```
/// # use rhdl::bits::{consts::U8, Bits, bits};
/// let value : Bits<U8> = bits(0b1010_1010);
/// assert_eq!(value, 0b1010_1010);
/// ```
/// Because the function is `const`, you can use it a constant
/// context:
/// ```
/// # use rhdl::bits::{consts::U8, Bits, bits};
/// const VALUE : Bits<U8> = bits(0b1010_1010);
/// ```
pub const fn bits<N: BitWidth>(value: u128) -> Bits<N> {
    assert!(value <= Bits::<N>::mask().val);
    Bits {
        marker: std::marker::PhantomData,
        val: value,
    }
}
pub const fn bits_masked<N: BitWidth>(value: u128) -> Bits<N> {
    Bits {
        marker: std::marker::PhantomData,
        val: value & Bits::<N>::mask().val,
    }
}

pub struct bits<N: BitWidth> {
    marker: std::marker::PhantomData<N>,
}

impl<N: BitWidth> Bits<N> {
    /// Defines a constant Bits value with all bits set to 1.
    pub const MASK: Self = Self::mask();
    pub const MAX: Self = Self::mask();
    pub const ZERO: Self = Self {
        marker: std::marker::PhantomData,
        val: 0,
    };
    /// Return a [Bits] value with all bits set to 1.
    /// ```
    /// # use rhdl::bits::{consts::U8, Bits};
    /// let bits = Bits::<U8>::mask();
    /// assert_eq!(bits, 0xFF);
    /// ```
    pub const fn mask() -> Self {
        Self {
            marker: std::marker::PhantomData,
            val: u128::MAX >> (128 - N::BITS),
        }
    }
    /// Extend a value by the number of bits specified.
    pub const fn xext<M>(self) -> Bits<op!(N + M)>
    where
        N: Add<M>,
        op!(N + M): BitWidth,
    {
        bits::<op!(N + M)>(self.val)
    }
    pub const fn xshr<M>(self) -> Bits<op!(N - M)>
    where
        N: Sub<M>,
        M: BitWidth,
        op!(N - M): BitWidth,
    {
        bits::<op!(N - M)>(self.val >> M::BITS)
    }
    pub const fn xshl<M>(self) -> Bits<op!(N + M)>
    where
        N: Add<M>,
        M: BitWidth,
        op!(N + M): BitWidth,
    {
        bits::<op!(N + M)>(self.val << M::BITS)
    }
    pub const fn resize<M: BitWidth>(self) -> Bits<M> {
        let mask = Bits::<M>::mask();
        bits(self.val & mask.val & Self::mask().val)
    }
    /// Reinterpret the [Bits] value as a [SignedBits] value.
    pub const fn as_signed(self) -> SignedBits<N> {
        // Need to a sign extension here.
        if self.val & (1_u128 << (N::BITS - 1)) != 0 {
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
            bits: N::BITS,
        }
        .wrapped()
    }
    /// Build a (dynamic, stack allocated) vector containing
    /// the bits that make up this value.  This will be slow.
    pub fn to_bools(self) -> Vec<bool> {
        let mut v = Vec::with_capacity(N::BITS);
        let mut x = self.val;
        for _i in 0..N::BITS {
            v.push(x & 1 == 1);
            x = x.wrapping_shr(1);
        }
        v
    }
    pub fn any(self) -> bool {
        (self.val & Self::mask().val) != 0
    }
    pub fn all(self) -> bool {
        (self.val & Self::mask().val) == Self::mask().val
    }
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
}

/// The default value for a [Bits] value is 0.
impl<N: BitWidth> Default for Bits<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Provide conversion from a `u128` to a [Bits] value.
/// This will panic if you try to convert a value that
/// is larger than the [Bits] value can hold.
impl<N: BitWidth> From<u128> for Bits<N> {
    fn from(value: u128) -> Self {
        assert!(value <= Self::mask().val);
        Self {
            marker: std::marker::PhantomData,
            val: value,
        }
    }
}

impl<N: BitWidth> PartialEq<Bits<N>> for u128 {
    fn eq(&self, other: &Bits<N>) -> bool {
        other.val == bits::<N>(*self).val
    }
}

impl<N: BitWidth> PartialEq<u128> for Bits<N> {
    fn eq(&self, other: &u128) -> bool {
        self.val == bits::<N>(*other).val
    }
}

impl<N: BitWidth> PartialOrd<Bits<N>> for u128 {
    fn partial_cmp(&self, other: &Bits<N>) -> Option<std::cmp::Ordering> {
        let self_as_bits = bits::<N>(*self);
        self_as_bits.val.partial_cmp(&other.val)
    }
}

impl<N: BitWidth> PartialOrd<u128> for Bits<N> {
    fn partial_cmp(&self, other: &u128) -> Option<std::cmp::Ordering> {
        let other_as_bits = bits::<N>(*other);
        self.val.partial_cmp(&other_as_bits.val)
    }
}

impl<T, N: BitWidth, const M: usize> std::ops::Index<Bits<N>> for [T; M] {
    type Output = T;
    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self[index.val as usize]
    }
}

impl<T, N: BitWidth, const M: usize> std::ops::IndexMut<Bits<N>> for [T; M] {
    fn index_mut(&mut self, index: Bits<N>) -> &mut Self::Output {
        &mut self[index.val as usize]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mask() {
        let bits = Bits::<U128>::mask();
        assert_eq!(bits.val, u128::MAX);
        let bits = Bits::<U32>::mask();
        assert_eq!(bits.val, 0xFFFF_FFFF_u128);
    }

    #[test]
    fn test_binary_format() {
        let bits: Bits<U8> = 0b1101_1010.into();
        assert_eq!(format!("{bits:b}"), "8'b11011010");
    }

    #[test]
    fn test_hex_format() {
        let bits: Bits<U8> = 0b1101_1010.into();
        assert_eq!(format!("{bits:x}"), "8'hda");
        assert_eq!(format!("{bits:X}"), "8'HDA");
    }

    #[test]
    fn test_to_bits_method() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let result = bits.to_bools();
        assert_eq!(
            result,
            vec![false, true, false, true, true, false, true, true]
        );
    }

    #[test]
    fn test_self_cast() {
        let bits: Bits<U8> = 0b1101_1010.into();
        let new_bits: Bits<U8> = bits.resize();
        assert_eq!(new_bits, 0b1101_1010);
        let new_bits = bits.resize::<U4>();
        assert_eq!(new_bits, 0b1010);
        let new_bits = bits.resize::<U16>();
        assert_eq!(new_bits, 0b0000_0000_1101_1010);
    }

    const OPT1: Bits<U8> = b8(0b1101_1010);
    const OPT2: Bits<U8> = b8(0b0010_0100);

    #[test]
    fn test_match() {
        let bits: Bits<U8> = 0b1101_1010.into();
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
            let b = a.xext::<U1>();
            assert_eq!(b, b9(i as u128));
        }
    }

    #[test]
    fn test_xshl() {
        for i in 0..=u8::MAX {
            let a = b8(i as u128);
            let b = a.xshl::<U1>();
            assert_eq!(b, b9((i as u128) << 1));
        }
    }

    #[test]
    fn test_xshr() {
        for i in 0..=u8::MAX {
            let a = b8(i as u128);
            let b = a.xshr::<U1>();
            assert_eq!(b, b7((i as u128) >> 1));
        }
    }

    #[test]
    fn test_no_wide_bits() {
        let _a = Bits::<U128>::mask();
        let _b = Bits::<op!(U127 + U1)>::mask();
    }
}
