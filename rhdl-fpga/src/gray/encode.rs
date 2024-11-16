use rhdl::prelude::*;

use super::Gray;

#[kernel]
pub fn gray_code<const N: usize>(i: Bits<N>) -> Gray<N> {
    Gray::<{ N }>(i ^ (i >> 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gray_code() {
        let values = (0..128).map(bits);
        let gray = values.map(gray_code::<7>);
        let gray = gray.collect::<Vec<_>>();
        assert!(gray.windows(2).all(|x| {
            let a = x[0].0;
            let b = x[1].0;
            let c = a ^ b;
            c.to_bools().into_iter().filter(|x| *x).count() == 1
        }));
    }
}
