// Tell clippy to ignore this module
#![allow(clippy::all)]
pub mod add;
pub mod consts;
pub mod digits;
pub mod len;
pub mod operators;
pub mod sub;
pub mod traits;
pub mod unsigned;

pub use digits::*;
pub use operators::*;
pub use traits::*;
pub use unsigned::{UInt, UTerm};

#[cfg(test)]
mod tests {
    use super::consts::*;
    use super::*;

    #[test]
    fn test_add() {
        let a = UInt::<UInt<UTerm, D1>, D2>::new();
        let b = UInt::<UInt<UTerm, D1>, D2>::new();
        let c = a + b;
        assert_eq!(c, UInt::<UInt<UTerm, D2>, D4>::new());
        type C = Sum<UInt<UInt<UTerm, D1>, D2>, UInt<UInt<UTerm, D1>, D2>>;
        assert_eq!(C::USIZE, 24);
        assert_eq!(Length::<C>::USIZE, 2);
        type D = Length<C>;
        let d: D = D::new();
        type E = Sum<C, D4>;
        assert_eq!(E::USIZE, 28);
        let e: E = E::new();
        type F = Trimmed<E>;
        assert_eq!(F::USIZE, 8);
        type G = Trimmed<F>;
        assert_eq!(G::USIZE, 0);
        //type I = Diff<C, F>;
        //assert_eq!(I::USIZE, 16);
        // 24 - 8 =  24
        //         + 91
        //        = 115
        //          |15 (trim)
    }

    #[test]
    fn test_sub() {
        let a: U193 = UInt::new();
        let b: U13 = UInt::new();
        let c = a - b;
        assert_eq!(c, U180::default());
        type Q = Add1<Diff<U93, U13>>;
        assert_eq!(Q::USIZE, 81);
    }
}
