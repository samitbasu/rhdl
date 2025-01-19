// Tell clippy to ignore this module
#![allow(clippy::all)]
pub mod add;
pub mod bools;
pub mod cmp;
pub mod consts;
pub mod digits;
pub mod invert;
pub mod is_cmp;
pub mod len;
pub mod normalize;
pub mod operators;
pub mod prelude;
pub mod same;
pub mod sub;
pub mod traits;
pub mod trim;
pub mod unsigned;

pub use digits::*;
pub use operators::*;
pub use rhdl_macro::op;
pub use traits::*;
pub use unsigned::{T_, U_};

#[cfg(test)]
#[rust_analyzer::skip]
mod tests {
    use bools::{IsFalse, IsTrue};
    use cmp::{CmpOut, Equal, FoldCmp, FoldOut, PCmp, PrivateCmp};
    use invert::InvertOut;
    use normalize::Normalized;
    use same::IsSame;
    use static_assertions::assert_impl_all;
    //use sub::PDiff;
    use rhdl_macro::op;
    use trim::Trimmed;

    use super::consts::*;
    use super::*;

    include!(concat!(env!("OUT_DIR"), "/tests.rs"));

    #[test]
    fn test_add() {
        let a = U_::<U_<T_, D1>, D2>::new();
        let b = U_::<U_<T_, D1>, D2>::new();
        let c = a + b;
        assert_eq!(c, U_::<U_<T_, D2>, D4>::new());
        type C = Sum<U_<U_<T_, D1>, D2>, U_<U_<T_, D1>, D2>>;
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
        let a: U180 = U_::new();
        let b = Trimmed::<U180>::new();
        assert_impl_all!(U18: IsSame<Trimmed::<U180>>);
        assert_impl_all!(U1: IsSame<Trimmed::<U10>>);
        assert_impl_all!(U185: IsSame<Trimmed::<U185>>);
        assert_impl_all!(U0: IsSame<Trimmed::<U0>>);
        //assert_eq!(a, b);
        let c: U_<U_<U_<T_, D0>, D5>, D0> = Default::default();
        let d: Trimmed<U_<U_<U_<T_, D0>, D5>, D0>> = Default::default();
        assert_impl_all!(Trimmed<U_<U_<U_<T_, D0>, D5>, D0>>: IsSame<U_<U_<T_, D0>, D5>>);
        assert_impl_all!(Trimmed<U_<U_<U_<T_, D0>, D0>, D0>>: IsSame<U0>);
        assert_impl_all!(Normalized<U_<U_<U_<T_, D0>, D5>, D0>>: IsSame<U50>);
        let c: U_<U_<U_<T_, D0>, D5>, D0> = Default::default();

        let e: U_<T_, D0> = Default::default();
        let f = Trimmed::<U_<T_, D0>>::new();
        type T = Trimmed<U_<U_<T_, D1>, D0>>;
    }

    #[test]
    fn test_sub() {
        let a: U12 = U_::new();
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
        let c = Diff::<U128, U65>::new();
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
        let c: CmpOut<U5, U0> = Default::default();
        let c: CmpOut<U0, U0> = Default::default();
        let c: CmpOut<U5, Sum<U2, U3>> = Default::default();
        let h: Maximum<U245, U145> = Default::default();
        let i: Maximum<U134, U200> = Default::default();
        let k: Maximum<U33, U33> = Default::default();
        let p: Maximum<U23, U0> = Default::default();
        let q: Minimum<U33, U23> = Default::default();
        assert_eq!(CmpOut::<Sum<U45, U1>, U46>::default(), Equal);
        assert_eq_num!(Sum<U45, U1>, U46);
    }

    #[test]
    fn test_is_ops() {
        let c: IsLessThan<U245, U145> = Default::default();
        let d: IsLessThanOrEqualTo<U123, U128> = Default::default();
        assert_impl_all!(IsLessThanOrEqualTo<U123, U128>: IsTrue);
        assert_impl_all!(IsLessThanOrEqualTo<U123, U123>: IsTrue);
        assert_impl_all!(IsLessThanOrEqualTo<U123, U120>: IsFalse);
        assert_impl_all!(IsEqualTo<Sum<U123,U5>,U128>: IsTrue);
        assert_impl_all!(IsGreaterThanOrEqualTo<U123, U128>: IsFalse);
        assert_impl_all!(IsGreaterThanOrEqualTo<U130, U128>: IsTrue);
        let c: InvertOut<U245> = Default::default();
        assert_impl_all!(IsEqualTo<U245, op!(U245)>: IsTrue);
        assert_impl_all!(IsEqualTo<U245, op!(U240 + U5)>: IsTrue);
        assert_impl_all!(IsEqualTo<U245, op!(U240 + U7 - U2)>: IsTrue);
        assert_impl_all!(IsEqualTo<U245, op!(max(U240, U100) + U7 - U2)>: IsTrue);
        assert_impl_all!(IsEqualTo<U245, op!(max(U240, U100) + U7 - min(U2, U92))>: IsTrue);
        assert_impl_all!(op!(U130 > U128): IsTrue);
        assert_impl_all!(op!(U120 > U130): IsFalse);
        assert_impl_all!(op!(U120 >= U120): IsTrue);
        assert_impl_all!(op!(U120 >= U10): IsTrue);
        assert_impl_all!(op!(U13 == U13): IsTrue);
        assert_impl_all!(op!(U13 != U13): IsFalse);
        assert_impl_all!(op!(U13 != U15): IsTrue);
        assert_impl_all!(op!(U13 < U15): IsTrue);
        assert_impl_all!(op!(U13 <= U15): IsTrue);
        assert_impl_all!(op!(U13 <= U13): IsTrue);
    }
}

#[macro_export]
macro_rules! assert_eq_num {
    ($ex1: ty, $ex2: ty) => {
        assert_eq!(CmpOut::<$ex1, $ex2>::default(), Equal);
    };
}
