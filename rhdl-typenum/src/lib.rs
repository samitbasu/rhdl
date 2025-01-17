// Tell clippy to ignore this module
#![allow(clippy::all)]
pub mod add;
pub mod bools;
pub mod cmp;
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
    use cmp::{Equal, FoldCmp, FoldOut, PCmp, PrivateCmp};
    use sub::PDiff;

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
        let a = U142::new();
        let c = U55::new();
        let d = a + c;
        assert_eq!(d, U197::new());
        let e = a + U0::new();
        assert_eq!(e, U142::new());
    }

    #[test]
    fn test_trim() {
        let a: U180 = UInt::new();
        let b = Trimmed::<U180>::new();
        assert_eq!(a, b);
        let c: UInt<UInt<UInt<UTerm, D0>, D5>, D0> = Default::default();
        let d = c.trim();
        assert_eq!(d, U50::new());
        let e: UInt<UTerm, D0> = Default::default();
        let f = Trimmed::<UInt<UTerm, D0>>::new();
        type T = Trimmed<UInt<UInt<UTerm, D1>, D0>>;
        assert_eq!(T::USIZE, 10);
    }

    #[test]
    fn test_sub() {
        let a: U12 = UInt::new();
        let c = Diff::<U12, D2>::new();
        let c = a - D2::new();
        let c = Diff::<U12, D9>::new();
        let c = a - D9::new();
        let c = Diff::<U8, D8>::new();
        let c = U8::new() - D8::new();
        assert_eq!(c, U0::new());
        let c = Diff::<U5, U3>::new();
        let c = Diff::<U42, U13>::new();
        let c = Diff::<U42, U41>::new();
        let c = Diff::<U42, U41>::new();
        let c = Diff::<U42, U42>::new();

        /*         let b: U13 = UInt::new();
               let c = a - b;
               assert_eq!(c, U180::default());
               type Q = Add1<Diff<U93, U13>>;
               assert_eq!(Q::USIZE, 81);
        */
    }

    #[test]
    fn test_cmp() {
        let c: PCmp<U245, U145> = Default::default();
        let a: PCmp<U138, U41> = Default::default();
        let b: PCmp<U204, U203> = Default::default();
        let b: FoldOut<PCmp<U105, U145>, Equal> = Default::default();
        let c: CmpOut<U245, U145> = Default::default();
        let c: CmpOut<U145, U145> = Default::default();
        let c: CmpOut<U45, U145> = Default::default();
        let c: CmpOut<U245, U243> = Default::default();
        let c: CmpOut<U245, U245> = Default::default();
        let c: CmpOut<U245, U247> = Default::default();
        let c: CmpOut<U4, U5> = Default::default();
        let c: CmpOut<U5, U4> = Default::default();
        let c: CmpOut<U5, Sum<U2, U3>> = Default::default();
    }
}
