pub mod add;
pub mod and;
pub mod bits;
pub mod neg;
pub mod not;
pub mod or;
pub mod shl;
pub mod shr;
pub mod signed_bits;
pub mod sub;
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
