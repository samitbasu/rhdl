//! Here the `slice` operator will extract the upper 4 bits of the `bits` value, and
//! they can then be operated on using the `&` operator.  Note that the `slice` operator
//! is generic over the width of the slice, so you can extract any number of bits from
//! the [Bits] value.  If you request more bits than the [Bits] value has, the extra
//! bits will be initialized to 0.
//!
//! ```
//! # use rhdl_bits::{Bits, alias::*};
//! let bits: b8 = 0b1101_1010.into();
//! let word: b16 = bits.slice(0);
//! assert_eq!(word, 0b0000_0000_1101_1010);
//! ```
//!
//! You can also `slice` [SignedBits] as well.  However, in this case, extra bits
//! are sign-extended, not zero-extended.  And the end result is a [Bits] type,
//! not a [SignedBits] type.  For example:
//!
//! ```
//! # use rhdl_bits::{alias::*};
//! let bits: s8 = (-42).into();
//! let word: b16 = bits.slice(0);
//! assert_eq!(word, 0xFF_D6);
//! ```
//!
//! * Be careful * when using the `slice` operator on [SignedBits] values.  If you
//! slice a [SignedBits] value to a smaller size, the sign bit will be lost.  For
//! example:
//!
//! ```
//! # use rhdl_bits::{alias::*};
//! let bits: s8 = (-42).into();
//! let nibble: b4 = bits.slice(0);
//! assert_eq!(nibble, 6);
//! ```
//!
//! To elaborate on this example, -42 in 8 bits is 1101_0110.  If you slice this
//! to 4 bits, you get 0110, which is 6.  The sign bit is lost in the slicing.
//!
//!
//! For example:
//! ```
//! # use rhdl_bits::{Bits, alias::*};
//! let bits: b8 = 0b1101_1010.into();
//! let nibble: b4 = 0b1111.into();
//! let result = bits.slice(4) & nibble;
//! assert_eq!(result, 0b1101);
//! ```
//!

pub mod all;
pub mod any;
pub mod get_bit;
pub mod set_bit;
pub mod sign_bit;
pub mod signed;
pub mod slice;
pub mod unsigned;
pub mod xor;

pub use all::*;
pub use any::*;
pub use get_bit::*;
pub use set_bit::*;
pub use sign_bit::*;
pub use signed::*;
pub use slice::*;
pub use unsigned::*;
pub use xor::*;
