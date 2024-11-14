use rhdl::prelude::*;

use super::Gray;

// A combinatorial gray code decoder.  Given a gray coded number,
// this module will generate the binary number for that gray code.
// Note that this is purely combinatorial, and thus may be too slow
// for high speed applications.
pub type U<const N: usize> = Func<Gray<N>, Bits<N>>;

#[kernel]
pub fn gray_decode<const N: usize>(_cr: ClockReset, i: Gray<N>) -> Bits<N> {
    let mut o = i.0;
    o ^= o >> 1;
    if ({ N } > 2) {
        o ^= o >> 2;
    }
    if ({ N } > 4) {
        o ^= o >> 4;
    }
    if ({ N } > 8) {
        o ^= o >> 8;
    }
    if ({ N } > 16) {
        o ^= o >> 16;
    }
    if ({ N } > 32) {
        o ^= o >> 32;
    }
    if ({ N } > 64) {
        o ^= o >> 64;
    }
    o
}

pub fn new<const N: usize>() -> Result<U<N>, RHDLError> {
    Func::new::<gray_decode<N>>()
}

#[cfg(test)]
mod tests {

    use crate::gray::encode::gray_code;

    use super::*;

    fn test_gray_decode<const N: usize>(max_val: u128) {
        let values = (0..max_val).map(bits::<N>);
        let cr = ClockReset::default();
        let gray = values.clone().map(|x| gray_code(cr, x));
        let bin = gray.map(|x| gray_decode(cr, x));
        assert!(bin.eq(values));
    }

    #[test]
    fn test_gray_round_trip_1() {
        test_gray_decode::<1>(1 << 1);
    }

    #[test]
    fn test_gray_round_trip_2() {
        test_gray_decode::<2>(1 << 2);
    }

    #[test]
    fn test_gray_round_trip_3() {
        test_gray_decode::<3>(1 << 3);
    }

    #[test]
    fn test_gray_round_trip_8() {
        test_gray_decode::<8>(1 << 8);
    }

    #[test]
    fn test_gray_round_trip_16() {
        test_gray_decode::<16>(1 << 16);
    }

    #[test]
    fn test_gray_round_trip_19() {
        test_gray_decode::<19>(1 << 19);
    }

    #[test]
    fn test_gray_round_trip_24() {
        test_gray_decode::<24>(1 << 24);
    }
}
