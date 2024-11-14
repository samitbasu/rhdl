use rhdl::prelude::*;

use super::Gray;

// A combinatorial gray code generator.  Given a binary number with the
// same number of bits as the gray code, this module will generate the
// gray code for that number.
pub type U<const N: usize> = Func<Bits<N>, Gray<N>>;

#[kernel]
pub fn gray_code<const N: usize>(_cr: ClockReset, i: Bits<N>) -> Gray<N> {
    Gray::<{ N }>(i ^ (i >> 1))
}

pub fn new<const N: usize>() -> Result<U<N>, RHDLError> {
    Func::new::<gray_code<N>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gray_code() {
        let values = (0..128).map(bits);
        let gray = values.map(|x| gray_code::<7>(ClockReset::default(), x));
        let gray = gray.collect::<Vec<_>>();
        assert!(gray.windows(2).all(|x| {
            let a = x[0].0;
            let b = x[1].0;
            let c = a ^ b;
            c.to_bools().into_iter().filter(|x| *x).count() == 1
        }));
    }
}
