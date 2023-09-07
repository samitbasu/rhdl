#![warn(missing_docs)]
//! This crate provides two types that are important for working with hardware
//! designs.  The [Bits] type is a fixed-size unsigned integer type with a variable
//! number of bits.  The [SignedBits] type is a fixed-size signed integer type with
//! a variable number of bits.  Both types are designed to mimic the behavior of
//! fixed width binary-represented integers as typically are synthesized in hardware
//! designs.  
//!
//! One significant difference between hardware design and software programming is the need
//! (and indeed ability) to easily manipulate collections of bits that are of various lengths.
//! While Rust has built in types to represent 8, 16, 32, 64, and 128 bits (at the time of this
//! writing, anyway), it is difficult to represent a 5 bit type. Or a 256 bit type.  Or indeed
//! any bit length that is not a power of two or is larger than 128 bits.
//!
//! The [Bits] and [SignedBits] types are designed to fill this gap.  They are generic over
//! the number of bits they represent, and can be used to represent any number of bits from 1
//! to 128.  The [Bits] type is an unsigned integer type, and the [SignedBits] type is a signed
//! integer type.  Both types implement the standard Rust traits for integer types, including
//! [Add](std::ops::Add), [Sub](std::ops::Sub), [BitAnd](std::ops::BitAnd),
//! [BitOr](std::ops::BitOr), [BitXor](std::ops::BitXor), [Shl](std::ops::Shl),
//! [Shr](std::ops::Shr), [Not](std::ops::Not), [Eq], [Ord], [PartialEq], [PartialOrd],
//! [Display](std::fmt::Display), [LowerHex](std::fmt::LowerHex),
//! [UpperHex](std::fmt::UpperHex), and [Binary](std::fmt::Binary).  
//! The [SignedBits] type also implements [Neg](std::ops::Neg).  Note that in all cases,
//! these types implement 2-s complement wrapping arithmetic, just as you would find in
//! hardware designs.  They do not panic on underflow or overflow, but simply wrap around.
//! This is the behavior that best mimics real hardware design.  You can, of course,
//! implement detection for overflow and underflow in your designs, but this is not the
//! default behavior.
//!
//! The two types are also [Copy], which makes them easy to use just like intrinsic integer types.
//! Some general advice.  Hardware manipulation of bit vectors can seem counterintuitive if you
//! have not done it before.  The [Bits] and [SignedBits] types are designed to mimic the behavior
//! of hardware designs, and so they may not behave the way you expect.  If you are not familiar
//! with 2's complement arithmetic, you should read up on it before using these types.
//!
//! # Constructing [Bits]
//! There are several ways to construct a [Bits] value.  The simplest is to use the
//! [From] trait, and convert from integer literals.  For example:
//! ```
//! use rhdl_bits::Bits;
//! let bits: Bits<8> = 0b1101_1010.into();
//! ```
//! This will work for any integer literal that is in the range of the [Bits] type.
//! If the literal is outside the range of the [Bits] type, Rust will panic.
//!
//! You can also construct a [Bits] value from a [u128] value:
//! ```
//! # use rhdl_bits::Bits;
//! let bits: Bits<8> = 0b1101_1010_u128.into();
//! ```
//!
//! Note that the [Bits] type only supports up to 128 bit values.  Larger bit vectors
//! can easily be constructed using data structures (arrays, structs, enums, tuples).
//! But arithmetic on them is not supported by default.  You will need to provide your
//! own arithmetic implementations for these types.  This is a limitation of the
//! [Bits] type, but it is a limitation that is not likely to be a problem in practice.
//! Practical hardware limitations can mean that performing arithmetic on very long
//! bit vectors is likely to be very slow.
//!
//! # Constructing [SignedBits]
//! The [SignedBits] type can be constructed in the same way as the [Bits] type.  The
//! only difference is that the [SignedBits] type can be constructed from a [i128] value:
//! ```
//! # use rhdl_bits::SignedBits;
//! let bits: SignedBits<8> = 0b0101_1010_i128.into();
//! ```
//!
//! Likewise, you can construct a [SignedBits] from a signed literal
//! ```
//! # use rhdl_bits::SignedBits;
//! let bits: SignedBits<8> = (-42).into();
//! ```
//! *Note the parenthesis!*  Because of the order of operations, the negation has a lower
//! precedence than the `.into()`.  As a result, if you omit the parenthesis, you will
//! get a Rust complaint about not being able to decide what type the integer literal must
//! assume.  This is unfortunate, but unavoidable.
//!
//! # Operations
//! Only a subset of operations are defined for [Bits] and [SignedBits].  These are
//! the operations that can be synthesized in hardware without surprises (generally
//! speaking).  In Rust, you can operate between [Bits] types and other [Bits] types
//! of the _same width_, or you can use integer literals, which will be converted to
//! [Bits] types of the appropriate width.  For example:
//! ```
//! # use rhdl_bits::Bits;
//! let bits: Bits<8> = 0b1101_1010.into();
//! let result = bits & 0b1111_0000;
//! assert_eq!(result, 0b1101_0000);
//! ```
//!
//! Note that in case the `result` is being directly compared to an integer literal.
//!
//! You can also operate on [Bits] types of different widths, but you will need to
//! convert them to the same width first.  For example:
//! ```
//! # use rhdl_bits::Bits;
//! let bits: Bits<8> = 0b1101_1010.into();
//! let nibble: Bits<4> = 0b1111.into();
//! let result = bits.slice(4) & nibble;
//! assert_eq!(result, 0b1101);
//! ```
//!
//! Here the `slice` operator will extract the upper 4 bits of the `bits` value, and
//! they can then be operated on using the `&` operator.  Note that the `slice` operator
//! is generic over the width of the slice, so you can extract any number of bits from
//! the [Bits] value.  If you request more bits than the [Bits] value has, the extra
//! bits will be initialized to 0.
//!
//! ```
//! # use rhdl_bits::Bits;
//! let bits: Bits<8> = 0b1101_1010.into();
//! let word: Bits<16> = bits.slice(0);
//! assert_eq!(word, 0b0000_0000_1101_1010);
//! ```
//!
//! You can also `slice` [SignedBits] as well.  However, in this case, extra bits
//! are sign-extended, not zero-extended.  And the end result is a [Bits] type,
//! not a [SignedBits] type.  For example:
//!
//! ```
//! # use rhdl_bits::{SignedBits, Bits};
//! let bits: SignedBits<8> = (-42).into();
//! let word: Bits<16> = bits.slice(0);
//! assert_eq!(word, 0xFF_D6);
//! ```
//!
//! * Be careful * when using the `slice` operator on [SignedBits] values.  If you
//! slice a [SignedBits] value to a smaller size, the sign bit will be lost.  For
//! example:
//!
//! ```
//! # use rhdl_bits::{Bits,SignedBits};
//! let bits: SignedBits<8> = (-42).into();
//! let nibble: Bits<4> = bits.slice(0);
//! assert_eq!(nibble, 6);
//! ```
//!
//! To elaborate on this example, -42 in 8 bits is 1101_0110.  If you slice this
//! to 4 bits, you get 0110, which is 6.  The sign bit is lost in the slicing.
//!
//! ## Bit Widths and Binary Operators
//!
//! All of the binary operators follow the same rules:
//! * Both operands must be of the same width.
//! * Both operands must be of the same type (e.g., [SignedBits] or [Bits]).
//! * One of the operands may be a literal, in which case it will be converted to
//! the appropriate type before the operator is applied.
//!
//! These rules are entirely enforced in the Rust type system.  So there is nothing
//! special about following these rules that you are not already accustomed to.  The
//! following, for example, will fail to compile:
//!
//! ```compile_fail
//! # use rust_hdl_bits::Bits;
//! let x: Bits<20> = 0x1234.into();
//! let y: Bits<21> = 0x5123.into();
//! let z = x + y; // This will fail to compile.
//! ```
//!
//! ## Addition
//!
//! Addition is supported for [Bits] and [SignedBits] types.  You can add two
//! [Bits] values together, or you can add a [Bits] value to an integer literal.
//! For example:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<32> = 0xDEAD_BEEE.into();
//! let y: Bits<32> = x + 1;
//! assert_eq!(y, 0xDEAD_BEEF);
//! ```
//! The order of the arguments does not matter:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<32> = 0xDEAD_BEEE.into();
//! let y: Bits<32> = 1 + x;
//! assert_eq!(y, 0xDEAD_BEEF);
//! ```
//!
//! Or using two [Bits] values:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<32> = 0xDEAD_0000.into();
//! let y: Bits<32> = 0xBEEF.into();
//! let z: Bits<32> = x + y;
//! assert_eq!(z, 0xDEAD_BEEF);
//! ```
//!
//! The [AddAssign](std::ops::AddAssign) trait is also implemented for [Bits] and
//! [SignedBits], so you can use the `+=` operator as well:
//! ```
//! # use rhdl_bits::Bits;
//! let mut x: Bits<32> = 0xDEAD_0000.into();
//! x += 0xBEEF;
//! assert_eq!(x, 0xDEAD_BEEF);
//! ```
//!
//! Note that the addition operation os 2's complement wrapping addition.  This is
//! the behavior that is most useful for hardware designs.  If you want to detect
//! overflow, you will need to implement that yourself.
//!
//! ```
//! # use rhdl_bits::Bits;
//! let mut x: Bits<8> = 0b1111_1111.into();
//! x += 1;
//! assert_eq!(x, 0);
//! ```
//!
//! In this case, the addition of 1 caused `x` to wrap to all zeros.  This is totally normal,
//! and what one would expect from hardware addition (without a carry).  If you _need_ the
//! carry bit, then the solution is to first cast to 1 higher bit, and then add, or alternately,
//! to compute the carry directly.
//!
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<40> = (0xFF_FFFF_FFFF).into();
//! let y: Bits<41> = x.slice(0) + 1;
//! assert_eq!(y, 0x100_0000_0000);
//! ```
//!
//! ## Subtraction
//! Hardware subtraction is defined using 2s complement arithmetic.  This is pretty
//! much the universal standard for representing negative numbers and subtraction in
//! hardware.  The [Sub](std::ops::Sub) trait is implemented for [Bits] and [SignedBits],
//! and operates much like the [Wrapping](std::num::Wrapping) trait does for the
//! built in integers in Rust.  Note that overflow and underflow are _not_ detected
//! in RHDL (nor are they detected in hardware either).  You will need to explicitly
//! check for overflow or underflow conditions if you want to take action in those
//! circumstances.
//!
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b0000_0001.into();
//! let y: Bits<8> = 0b0000_0010.into();
//! let z: Bits<8> = x - y; // 1 - 2 = -1
//! assert_eq!(z, 0b1111_1111);
//! ```
//!
//! Note that in this case, we subtracted 2 from 1, and the result was -1.  However,
//! -1 in 2s complement is `0xFF`, which is stored in `z` as an unsigned value of 255.
//! This is the same behavior that you experience with `u8` in standard Rust if you
//! use Wrapping arithmetic:
//! ```
//! let x : u8 = 1;
//! let y : u8 = 2;
//! let z = u8::wrapping_sub(x, y);
//! assert_eq!(z, 0b1111_1111);
//! ```
//!
//! I don't want to belabor the point, but wrapping arithmetic and 2s complement
//! representations can catch people by surprise if they are unfamiliar with
//! hardware implementations of arithmetic.
//!
//! For [SignedBits], the result is the same, but interpreted correctly:
//! ```
//! # use rhdl_bits::SignedBits;
//! let x: SignedBits<8> = 0b0000_0001.into();
//! let y: SignedBits<8> = 0b0000_0010.into();
//! let z: SignedBits<8> = x - y; // 1 - 2 = -1
//! assert_eq!(z, -1);
//! ```
//!
//! The [SubAssign](std::ops::SubAssign) trait is implemented for both [Bits] and [SignedBits],
//! so you can use the `-=` operator as well:
//! ```
//! # use rhdl_bits::Bits;
//! let mut x: Bits<8> = 0b0000_0001.into();
//! x -= 1;
//! assert_eq!(x, 0);
//! ```
//!
//! ## Bitwise Logical Operators
//!
//! All four of the standard Rust logical operators are supported for both [Bits] and [SignedBits].
//! They operate bitwise, and are implemented using the standard Rust traits.  For completeness,
//! the list of supported bitwise operators is:
//! - [Or](std::ops::Or) and [OrAssign](std::ops::OrAssign) for `|` and `|=`
//! - [And](std::ops::And) and [AndAssign](std::ops::AndAssign) for `&` and `&=`
//! - [Xor](std::ops::Xor) and [XorAssign](std::ops::XorAssign) for `^` and `^=`
//! - [Not](std::ops::Not) for `!`
//! Other, more exotic binary operators (like Xnor or Nand) are not supported.  If you need these,
//! you will need to implement them in terms of these more basic operators.
//!
//! Here is an example of the binary operators in action:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b1101_1010.into();
//! let y: Bits<8> = 0b1111_0000.into();
//! let z: Bits<8> = x | y;
//! assert_eq!(z, 0b1111_1010);
//! let z: Bits<8> = x & y;
//! assert_eq!(z, 0b1101_0000);
//! let z: Bits<8> = x ^ y;
//! assert_eq!(z, 0b0010_1010);
//! let z: Bits<8> = !x;
//! assert_eq!(z, 0b0010_0101);
//! ```
//!
//! Note that you can apply these operators to [SignedBits] as well.  The meaning of the result is up
//! to you to interpret.  The bitwise operators simply manipulate the bits, and do not care about
//! the sign of the value.  This is also true for Rust and intrinsic types.
//!
//! ```
//! let x: i8 = -0b0101_1010;
//! let y: i8 = -0b0111_0000;
//! let z = x ^ y; // This will be positive
//! assert_eq!(z, 54);
//! ```
//!
//! ## Shifting
//!
//! Shifting is a fairly complex topic, since it involves a few additional details:
//! - The behavior of [Bits] and [SignedBits] are identical under left shifting.
//! - The behavior of [Bits] and [SignedBits] are different under right shifting.
//! - Right shifting a [Bits] will cause `0` to be inserted on the "left" of the value (the MSB).
//! - Right shifting a [SignedBits] will cause the MSB to be replicated on the "left" of the value.
//!
//! The net effect of these differences is that left shifting (to a point) will preserve the
//! sign of the value, until all the bits are shifted out of the value.  Right shifting will
//! preserve the sign of the value.  If you want to right shift a [SignedBits] value with 0 insertion
//! at the MSB, then convert it to a [Bits] first.
//!
//! The [Shl](std::ops::Shl) and [ShlAssign](std::ops::ShlAssign) traits are implemented for both
//! [Bits] and [SignedBits].  The [Shr](std::ops::Shr) and [ShrAssign](std::ops::ShrAssign) traits
//! are also implemented for both.  Note that unlike the other operators, the shift operators allow
//! you to use a *different* bit width for the shift amount.  This is because in hardware designs,
//! the amount of the shift is often controlled dynamically (using circuitry known as a Barrel Shifter).
//! And the number of bits used to encode the shift will be related to the base-2 log of the number
//! of bits in the register.  For example, if you have a 32 bit register, you will need 5 bits to
//! encode the shift amount.  If you have a 64 bit register, you will need 6 bits to encode the
//! shift amount.  And so on.
//!
//! In order to model this, the shift operators are generic over both the number of bits in the value
//! being shifted, _and_ the number of bits in the value that controls the shift.  For example:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b1101_1010.into();
//! let y: Bits<3> = 0b101.into();
//! let z: Bits<8> = x >> y;
//! assert_eq!(z, 0b0000_0110);
//! ```
//!
//! You can also use an integer literal to control the shift amount
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b1101_1010.into();
//! let z: Bits<8> = x >> 3;
//! assert_eq!(z, 0b0001_1011);
//! ```
//!
//! There is one critical difference between shift operators on [Bits]/[SignedBits] and the wrapping
//! Rust operators on intrinsic integers.  Rust will do nothing if you shift by more bits than are
//! in the value.  For example:
//! ```
//! let x: u8 = 0b1101_1010;
//! let y = u8::wrapping_shl(x,10);
//! assert_ne!(y, 0b1101_1010); // Note that this is _not_ zero - the result is not even clearly defined.
//! ```
//! This is not the case for [Bits] and [SignedBits].  If you shift by more bits than are in the value,
//! the result will simply be zero (unless you are right shifting a [SignedBits] value, in which case it
//! will converge to either zero or -1, depending on the sign bit).  This is an odd case to cover, and
//! it is not clear what the "correct" behavior should be.  But this is the behavior that is implemented
//! in RHDL.
//!
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b1101_1010.into();
//! let z: Bits<8> = x >> 10;
//! assert_eq!(z, 0);
//! ```
//!
//! ## Comparison Operators
//!
//! The standard Rust comparison operators are implemented for both [Bits] and [SignedBits].  These
//! operators are:
//! - [PartialEq](std::cmp::PartialEq), [Eq](std::cmp::Eq) for `==` and `!=`
//! - [Ord](std::cmp::Ord) and [PartialOrd](std::cmp::PartialOrd) for `<`, `>`, `<=`, and `>=`
//!
//! Note that the comparison operators are implemented using signed arithmetic for [SignedBits], and
//! unsigned arithmetic for [Bits].  This is the same behavior that you would see in hardware designs.
//! For example, with [Bits]:
//! ```
//! # use rhdl_bits::Bits;
//! let x: Bits<8> = 0b1111_1111.into();
//! let y: Bits<8> = 0b0000_0000.into();
//! assert!(x > y);
//! ```
//! On the other hand with [SignedBits]:
//! ```
//! # use rhdl_bits::SignedBits;
//! let x: SignedBits<8> = (-0b0000_0001).into();
//! let y: SignedBits<8> = 0b0000_0000.into();
//! assert!(x < y);
//! assert_eq!(x.as_unsigned(), 0b1111_1111);
//! ```
//!
/// Implementation of the addition operators
pub mod add;
/// Implementation of the logical and operators
pub mod and;
/// The base [Bits] struct used to represent arbitrary bit-width unsigned integers.
pub mod bits;
/// Implementation of the negation (-) operator for signed integers
pub mod neg;
/// Implementation of the logical not operator
pub mod not;
/// Implementation of the logical or operators
pub mod or;
/// Implementation of the shift left operator
pub mod shl;
/// Implementation of the shift right operator
pub mod shr;
/// The base [SignedBits] struct used to represent arbitrary bit-width signed integers.
pub mod signed_bits;
/// Implementation of subtraction operators
pub mod sub;
/// Implementatino of exclusive or operators
pub mod xor;

pub use bits::Bits;
pub use signed_bits::SignedBits;

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn time_adding_120_bit_values() {
        use std::time::Instant;
        let mut a = Bits::<120>::default();
        let mut b = Bits::<120>::default();
        let mut c = Bits::<120>::default();
        let start = Instant::now();
        for _k in 0..100 {
            for i in 0..120 {
                for j in 0..120 {
                    a.set_bit(i, true);
                    b.set_bit(j, true);
                    c += a + b;
                    a.set_bit(i, false);
                    b.set_bit(j, false);
                }
            }
        }
        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
        println!("c = {:b}", c);
    }
}
