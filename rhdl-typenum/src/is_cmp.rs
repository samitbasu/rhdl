use crate::{
    bools::{False, True},
    cmp::{Cmp, CmpOut, Equal, Greater, Less},
    Bool, IsEqual, IsGreater, IsGreaterThanOrEqual, IsLess, IsLessThanOrEqual, IsNotEqual,
};

pub trait IsLessPrivate<B, T> {
    type Output: Bool;
}

type IsLessPrivateOut<A, B, T> = <A as IsLessPrivate<B, T>>::Output;

impl<A, B> IsLess<B> for A
where
    A: Cmp<B> + IsLessPrivate<B, CmpOut<A, B>>,
{
    type Output = IsLessPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsLessPrivate<B, Equal> for A {
    type Output = False;
}

impl<A, B> IsLessPrivate<B, Greater> for A {
    type Output = False;
}

impl<A, B> IsLessPrivate<B, Less> for A {
    type Output = True;
}

pub trait IsGreaterPrivate<B, T> {
    type Output: Bool;
}

type IsGreaterPrivateOut<A, B, T> = <A as IsGreaterPrivate<B, T>>::Output;

impl<A, B> IsGreater<B> for A
where
    A: Cmp<B> + IsGreaterPrivate<B, CmpOut<A, B>>,
{
    type Output = IsGreaterPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsGreaterPrivate<B, Equal> for A {
    type Output = False;
}

impl<A, B> IsGreaterPrivate<B, Greater> for A {
    type Output = True;
}

impl<A, B> IsGreaterPrivate<B, Less> for A {
    type Output = False;
}

pub trait IsEqualPrivate<B, T> {
    type Output: Bool;
}

type IsEqualPrivateOut<A, B, T> = <A as IsEqualPrivate<B, T>>::Output;

impl<A, B> IsEqual<B> for A
where
    A: Cmp<B> + IsEqualPrivate<B, CmpOut<A, B>>,
{
    type Output = IsEqualPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsEqualPrivate<B, Equal> for A {
    type Output = True;
}

impl<A, B> IsEqualPrivate<B, Greater> for A {
    type Output = False;
}

impl<A, B> IsEqualPrivate<B, Less> for A {
    type Output = False;
}

pub trait IsLessThanOrEqualPrivate<B, T = Greater> {
    type Output: Bool;
}

type IsLessThanOrEqualPrivateOut<A, B, T> = <A as IsLessThanOrEqualPrivate<B, T>>::Output;

impl<A, B> IsLessThanOrEqual<B> for A
where
    A: Cmp<B> + IsLessThanOrEqualPrivate<B, CmpOut<A, B>>,
{
    type Output = IsLessThanOrEqualPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsLessThanOrEqualPrivate<B, Equal> for A {
    type Output = True;
}

impl<A, B> IsLessThanOrEqualPrivate<B, Greater> for A {
    type Output = False;
}

impl<A, B> IsLessThanOrEqualPrivate<B, Less> for A {
    type Output = True;
}

pub trait IsGreaterThanOrEqualPrivate<B, T> {
    type Output: Bool;
}

type IsGreaterThanOrEqualPrivateOut<A, B, T> = <A as IsGreaterThanOrEqualPrivate<B, T>>::Output;

impl<A, B> IsGreaterThanOrEqual<B> for A
where
    A: Cmp<B> + IsGreaterThanOrEqualPrivate<B, CmpOut<A, B>>,
{
    type Output = IsGreaterThanOrEqualPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsGreaterThanOrEqualPrivate<B, Equal> for A {
    type Output = True;
}

impl<A, B> IsGreaterThanOrEqualPrivate<B, Greater> for A {
    type Output = True;
}

impl<A, B> IsGreaterThanOrEqualPrivate<B, Less> for A {
    type Output = False;
}

pub trait IsNotEqualPrivate<B, T> {
    type Output: Bool;
}

type IsNotEqualPrivateOut<A, B, T> = <A as IsNotEqualPrivate<B, T>>::Output;

impl<A, B> IsNotEqual<B> for A
where
    A: Cmp<B> + IsNotEqualPrivate<B, CmpOut<A, B>>,
{
    type Output = IsNotEqualPrivateOut<A, B, CmpOut<A, B>>;
}

impl<A, B> IsNotEqualPrivate<B, Equal> for A {
    type Output = False;
}

impl<A, B> IsNotEqualPrivate<B, Greater> for A {
    type Output = True;
}

impl<A, B> IsNotEqualPrivate<B, Less> for A {
    type Output = True;
}
