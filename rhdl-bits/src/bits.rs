#![allow(non_camel_case_types)]
use crate::signed_bits::SignedBits;
use derive_more::{
    Binary, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Display, LowerHex,
    UpperHex,
};
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
    Display,
    LowerHex,
    UpperHex,
    Binary,
)]
#[repr(transparent)]
pub struct Bits<const N: usize>(pub(crate) u128);

seq!(N in 1..=128 {
    #(
        pub type b~N = Bits<N>;
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

impl<const N: usize> Bits<N> {
    /// Defines a constant Bits value with all bits set to 1.
    pub const MASK: Self = Self::mask();
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
    /// Set a specific bit of a [Bits] value to 1 or 0.
    /// Panics if the index of the bit is outside the range
    /// of the [Bits] value.
    /// ```
    /// # use rhdl_bits::Bits;
    /// let mut bits = Bits::<8>::mask();
    /// bits.set_bit(0, false);
    /// assert_eq!(bits, 0xFE);
    /// bits.set_bit(0, true);
    /// assert_eq!(bits, 0xFF);
    /// ```
    pub fn set_bit(&mut self, bit: usize, value: bool) {
        assert!(bit < N);
        if value {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }
    /// Get the value of a specific bit of a [Bits] value.
    /// Panics if the index of the bit is outside the range
    /// of the [Bits] value.
    /// ```
    /// # use rhdl_bits::Bits;
    /// let bits : Bits<8> = 0b1101_1010.into();
    /// assert!(!bits.get_bit(0));
    /// assert!(bits.get_bit(7));
    /// assert!(bits.get_bit(6));
    /// ```
    pub fn get_bit(&self, bit: usize) -> bool {
        assert!(bit < N);
        (self.0 & (1 << bit)) != 0
    }
    /// Returns true if any of the bits are set to 1.
    /// ```
    /// # use rhdl_bits::Bits;
    /// let bits : Bits<8> = 0b1101_1010.into();
    /// assert!(bits.any());
    /// let bits : Bits<8> = 0.into();
    /// assert!(!bits.any());
    /// ```
    pub fn any(self) -> bool {
        (self.0 & Self::mask().0) != 0
    }
    /// Returns true if all of the bits are set to 1.
    /// ```
    /// # use rhdl_bits::Bits;
    /// let bits : Bits<8> = 0b1101_1010.into();
    /// assert!(!bits.all());
    /// let bits : Bits<8> = Bits::mask();
    /// assert!(bits.all());
    /// ```
    pub fn all(self) -> bool {
        (self.0 & Self::mask().0) == Self::mask().0
    }
    /// Computes the xor of all of the bits in the value.
    pub fn xor(self) -> bool {
        let mut x = self.0 & Self::mask().0;
        x ^= x >> 64;
        x ^= x >> 32;
        x ^= x >> 16;
        x ^= x >> 8;
        x ^= x >> 4;
        x ^= x >> 2;
        x ^= x >> 1;
        x & 1 == 1
    }
    /// Extracts a range of bits from the [Bits] value.
    pub fn slice<const M: usize>(&self, start: usize) -> Bits<M> {
        Bits((self.0 >> start) & Bits::<M>::mask().0)
    }
    /// Reinterpret the [Bits] value as a [SignedBits] value.
    pub fn as_signed(self) -> SignedBits<N> {
        // Need to a sign extension here.
        if self.get_bit(N - 1) {
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
        let mut v = Vec::new();
        let mut x = self.0;
        for _i in 0..N {
            v.push(x & 1 == 1);
            x = x.wrapping_shr(1);
        }
        v
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

impl<const N: usize> PartialEq<u128> for Bits<N> {
    fn eq(&self, other: &u128) -> bool {
        self == &Self::from(*other)
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
    fn test_xor() {
        let bits = Bits::<128>::mask();
        assert!(!bits.xor());
        let bits = Bits::<32>::mask();
        assert!(!bits.xor());
        let bits = Bits::<1>::mask();
        assert!(bits.xor());
        let bits: Bits<5> = 0b11010.into();
        assert!(bits.xor());
    }

    #[test]
    fn test_all() {
        let bits = Bits::<128>::mask();
        assert!(bits.all());
        let bits = Bits::<32>::mask();
        assert!(bits.all());
        let bits = Bits::<1>::mask();
        assert!(bits.all());
        let bits: Bits<5> = 0b11111.into();
        assert!(bits.all());
        let bits: Bits<5> = 0b11110.into();
        assert!(!bits.all());
    }

    #[test]
    fn test_any() {
        let bits = Bits::<128>::mask();
        assert!(bits.any());
        let bits = Bits::<32>::mask();
        assert!(bits.any());
        let bits = Bits::<1>::mask();
        assert!(bits.any());
        let bits: Bits<5> = 0b11111.into();
        assert!(bits.any());
        let bits: Bits<5> = 0b00000.into();
        assert!(!bits.any());
    }

    #[test]
    fn test_set_bit() {
        let mut bits = Bits::<128>::mask();
        bits.set_bit(0, false);
        assert_eq!(bits.0, u128::MAX - 1);
        bits.set_bit(0, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(127, false);
        assert_eq!(bits.0, u128::MAX - (1 << 127));
        bits.set_bit(127, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(64, false);
        assert_eq!(bits.0, u128::MAX - (1 << 64));
        bits.set_bit(64, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(32, false);
        assert_eq!(bits.0, u128::MAX - (1 << 32));
        bits.set_bit(32, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(16, false);
        assert_eq!(bits.0, u128::MAX - (1 << 16));
        bits.set_bit(16, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(8, false);
        assert_eq!(bits.0, u128::MAX - (1 << 8));
        bits.set_bit(8, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(4, false);
        assert_eq!(bits.0, u128::MAX - (1 << 4));
        bits.set_bit(4, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(2, false);
        assert_eq!(bits.0, u128::MAX - (1 << 2));
        bits.set_bit(2, true);
        assert_eq!(bits.0, u128::MAX);
        bits.set_bit(1, false);
        assert_eq!(bits.0, u128::MAX - (1 << 1));
        bits.set_bit(1, true);
        assert_eq!(bits.0, u128::MAX);
    }

    #[test]
    fn test_get_bit() {
        let bits = Bits::<128>::mask();
        assert!(bits.get_bit(0));
        assert!(bits.get_bit(127));
        assert!(bits.get_bit(64));
        assert!(bits.get_bit(32));
        assert!(bits.get_bit(16));
        assert!(bits.get_bit(8));
        assert!(bits.get_bit(4));
        assert!(bits.get_bit(2));
        assert!(bits.get_bit(1));
        let bits = Bits::<32>::mask();
        assert!(bits.get_bit(0));
        assert!(bits.get_bit(31));
        assert!(bits.get_bit(16));
        assert!(bits.get_bit(8));
        assert!(bits.get_bit(4));
        assert!(bits.get_bit(2));
        assert!(bits.get_bit(1));
        let bits = Bits::<1>::mask();
        assert!(bits.get_bit(0));
        let bits: Bits<5> = 0b11010.into();
        assert!(bits.get_bit(4));
        assert!(bits.get_bit(3));
        assert!(!bits.get_bit(2));
        assert!(bits.get_bit(1));
        assert!(!bits.get_bit(0));
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
    fn test_slice_function() {
        let bits: Bits<8> = 0b1101_1010.into();
        let result = bits.slice::<4>(0);
        assert_eq!(result.0, 0b1010);
        let result = bits.slice::<4>(4);
        assert_eq!(result.0, 0b1101);
        let result = bits.slice::<2>(6);
        assert_eq!(result.0, 0b11);
    }

    #[test]
    fn test_round_trip_unsigned_signed() {
        let bits: Bits<8> = 0b1101_1010.into();
        let signed = bits.as_signed();
        println!("{}", signed);
        assert!(signed.is_negative());
        let unsigned = signed.as_unsigned();
        assert_eq!(bits.0, unsigned.0);
        let signed = unsigned.as_signed();
        assert!(signed.is_negative());
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
}
