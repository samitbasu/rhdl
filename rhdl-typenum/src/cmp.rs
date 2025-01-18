use seq_macro::seq;

use crate::bools::{False, True};
use crate::{digits::*, Bool, CmpOut, IsGreater, IsLess, Max, Min, Select, SelectOut};
use crate::{Cmp, Digit, UInt, UTerm, Unsigned};

/// A potential output from `Cmp`, this is the type equivalent to the enum variant
/// `core::cmp::Ordering::Greater`.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Greater {}

/// A potential output from `Cmp`, this is the type equivalent to the enum variant
/// `core::cmp::Ordering::Less`.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Less;

/// A potential output from `Cmp`, this is the type equivalent to the enum variant
/// `core::cmp::Ordering::Equal`.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct Equal;

// ---------------------------------------------------------------------------------------
// Compare unsigned integers

pub trait ComparisonResult {}

impl ComparisonResult for Greater {}
impl ComparisonResult for Less {}
impl ComparisonResult for Equal {}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct CTerm;

pub trait PrivateCmp<Rhs> {
    type Output;
}

pub type PCmp<A, B> = <A as PrivateCmp<B>>::Output;

seq!(N in 0..=9 {
    impl Cmp<D~N> for D~N {
        type Output = Equal;
    }
});

macro_rules! impl_cmp_digits {
    ($a: ty, $b:ty) => {
        impl Cmp<$b> for $a {
            type Output = Less;
        }

        impl Cmp<$a> for $b {
            type Output = Greater;
        }
    };
}

impl_cmp_digits!(D0, D1);
impl_cmp_digits!(D0, D2);
impl_cmp_digits!(D0, D3);
impl_cmp_digits!(D0, D4);
impl_cmp_digits!(D0, D5);
impl_cmp_digits!(D0, D6);
impl_cmp_digits!(D0, D7);
impl_cmp_digits!(D0, D8);
impl_cmp_digits!(D0, D9);
impl_cmp_digits!(D1, D2);
impl_cmp_digits!(D1, D3);
impl_cmp_digits!(D1, D4);
impl_cmp_digits!(D1, D5);
impl_cmp_digits!(D1, D6);
impl_cmp_digits!(D1, D7);
impl_cmp_digits!(D1, D8);
impl_cmp_digits!(D1, D9);
impl_cmp_digits!(D2, D3);
impl_cmp_digits!(D2, D4);
impl_cmp_digits!(D2, D5);
impl_cmp_digits!(D2, D6);
impl_cmp_digits!(D2, D7);
impl_cmp_digits!(D2, D8);
impl_cmp_digits!(D2, D9);
impl_cmp_digits!(D3, D4);
impl_cmp_digits!(D3, D5);
impl_cmp_digits!(D3, D6);
impl_cmp_digits!(D3, D7);
impl_cmp_digits!(D3, D8);
impl_cmp_digits!(D3, D9);
impl_cmp_digits!(D4, D5);
impl_cmp_digits!(D4, D6);
impl_cmp_digits!(D4, D7);
impl_cmp_digits!(D4, D8);
impl_cmp_digits!(D4, D9);
impl_cmp_digits!(D5, D6);
impl_cmp_digits!(D5, D7);
impl_cmp_digits!(D5, D8);
impl_cmp_digits!(D5, D9);
impl_cmp_digits!(D6, D7);
impl_cmp_digits!(D6, D8);
impl_cmp_digits!(D6, D9);
impl_cmp_digits!(D7, D8);
impl_cmp_digits!(D7, D9);
impl_cmp_digits!(D8, D9);

/// Zero == Zero
impl PrivateCmp<UTerm> for UTerm {
    type Output = CTerm;
}

/// Nonzero > Zero
impl<U: Unsigned, B: Digit> PrivateCmp<UTerm> for UInt<U, B> {
    type Output = CompChain<CTerm, Greater>;
}

/// Zero < Nonzero
impl<U: Unsigned, B: Digit> PrivateCmp<UInt<U, B>> for UTerm {
    type Output = CompChain<CTerm, Less>;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
pub struct CompChain<U, B> {
    pub msb: U,
    pub lsb: B,
}

impl<U, B> ComparisonResult for CompChain<U, B>
where
    U: ComparisonResult,
    B: ComparisonResult,
{
}

// Build a digit-wise comparison chain
impl<Ul: Unsigned, Ur: Unsigned, Bl: Digit, Br: Digit> PrivateCmp<UInt<Ur, Br>> for UInt<Ul, Bl>
where
    Ul: PrivateCmp<Ur>,
    Bl: Cmp<Br>,
{
    type Output = CompChain<PCmp<Ul, Ur>, CmpOut<Bl, Br>>;
}

pub trait FoldCmp<T> {
    type Output: ComparisonResult;
}

pub type FoldOut<A, T> = <A as FoldCmp<T>>::Output;

impl<A, T> FoldCmp<T> for CompChain<A, Equal>
where
    A: FoldCmp<T>,
    T: ComparisonResult,
{
    type Output = FoldOut<A, T>;
}

impl<A, T> FoldCmp<T> for CompChain<A, Greater>
where
    A: FoldCmp<Greater>,
    T: ComparisonResult,
{
    type Output = FoldOut<A, Greater>;
}

impl<A, T> FoldCmp<T> for CompChain<A, Less>
where
    A: FoldCmp<Less>,
    T: ComparisonResult,
{
    type Output = FoldOut<A, Less>;
}

impl<T> FoldCmp<T> for CTerm
where
    T: ComparisonResult,
{
    type Output = T;
}

impl<Ul: Unsigned, Bl: Digit, Ur: Unsigned, Br: Digit> Cmp<UInt<Ur, Br>> for UInt<Ul, Bl>
where
    Ul: PrivateCmp<Ur>,
    Bl: Cmp<Br>,
    PCmp<Ul, Ur>: FoldCmp<CmpOut<Bl, Br>>,
{
    type Output = FoldOut<PCmp<Ul, Ur>, CmpOut<Bl, Br>>;
}

impl<Ul: Unsigned, Bl: Digit> Cmp<UTerm> for UInt<Ul, Bl> {
    type Output = Greater;
}

impl<Ur: Unsigned, Br: Digit> Cmp<UInt<Ur, Br>> for UTerm {
    type Output = Less;
}

impl Cmp<UTerm> for UTerm {
    type Output = Equal;
}

impl<A: Unsigned, B: Unsigned> Max<B> for A
where
    A: Cmp<B> + Select<B, CmpOut<A, B>>,
    CmpOut<A, B>: ComparisonResult,
{
    type Output = SelectOut<A, B, CmpOut<A, B>>;
}

impl<A: Unsigned, B: Unsigned> Select<B, Greater> for A {
    type Output = A;
}

impl<A: Unsigned, B: Unsigned> Select<B, Less> for A {
    type Output = B;
}

impl<A: Unsigned, B: Unsigned> Select<B, Equal> for A {
    type Output = A;
}

impl<A: Unsigned, B: Unsigned> Min<B> for A
where
    A: Cmp<B>,
    B: Select<A, CmpOut<A, B>>,
    CmpOut<A, B>: ComparisonResult,
{
    type Output = SelectOut<B, A, CmpOut<A, B>>;
}
