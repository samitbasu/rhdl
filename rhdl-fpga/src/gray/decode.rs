use rhdl::prelude::*;

use super::Gray;

// A combinatorial gray code decoder.  Given a gray coded number,
// this module will generate the binary number for that gray code.
// Note that this is purely combinatorial, and thus may be too slow
// for high speed applications.

#[kernel]
pub fn gray_decode<N: BitWidth>(i: Gray<N>) -> Bits<N> {
    let mut o = i.0;
    o ^= o >> 1;
    if N::BITS > 2 {
        o ^= o >> 2;
    }
    if N::BITS > 4 {
        o ^= o >> 4;
    }
    if N::BITS > 8 {
        o ^= o >> 8;
    }
    if N::BITS > 16 {
        o ^= o >> 16;
    }
    if N::BITS > 32 {
        o ^= o >> 32;
    }
    if N::BITS > 64 {
        o ^= o >> 64;
    }
    o
}

#[cfg(test)]
mod tests {

    use crate::gray::encode::gray_code;

    use super::*;

    fn test_gray_decode<N: BitWidth>(max_val: u128) {
        let values = (0..max_val).map(bits::<N>);
        let gray = values.clone().map(gray_code);
        let bin = gray.map(gray_decode);
        assert!(bin.eq(values));
    }

    #[test]
    fn test_gray_round_trip_1() {
        test_gray_decode::<U1>(1 << 1);
    }

    #[test]
    fn test_gray_round_trip_2() {
        test_gray_decode::<U2>(1 << 2);
    }

    #[test]
    fn test_gray_round_trip_3() {
        test_gray_decode::<U3>(1 << 3);
    }

    #[test]
    fn test_gray_round_trip_8() {
        test_gray_decode::<U8>(1 << 8);
    }

    #[test]
    fn test_gray_round_trip_16() {
        test_gray_decode::<U16>(1 << 16);
    }

    #[test]
    fn test_gray_round_trip_19() {
        test_gray_decode::<U19>(1 << 19);
    }

    #[test]
    fn test_gray_round_trip_24() {
        test_gray_decode::<U24>(1 << 24);
    }
}
