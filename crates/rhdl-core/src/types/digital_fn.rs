#![allow(clippy::type_complexity)]

use crate::rhdl_bits::BitWidth;

pub use crate::rhdl_core::{kernel::KernelFnKind, Digital, Kind};

pub trait DigitalFn {
    fn kernel_fn() -> Option<KernelFnKind> {
        None
    }
}

impl DigitalFn for () {}

pub trait DigitalFn0 {
    type O: Digital;
    fn func() -> fn() -> Self::O;
}

pub trait DigitalFn1 {
    type A0: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0) -> Self::O;
}

pub trait DigitalFn2 {
    type A0: Digital;
    type A1: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1) -> Self::O;
}

pub struct NoKernel2<A0, A1, O> {
    _a0: std::marker::PhantomData<A0>,
    _a1: std::marker::PhantomData<A1>,
    _o: std::marker::PhantomData<O>,
}

impl<A0, A1, O> DigitalFn for NoKernel2<A0, A1, O> {}
impl<A0: Digital, A1: Digital, O: Digital> DigitalFn2 for NoKernel2<A0, A1, O> {
    type A0 = A0;
    type A1 = A1;
    type O = O;

    fn func() -> fn(Self::A0, Self::A1) -> Self::O {
        unimplemented!()
    }
}

pub trait DigitalFn3 {
    type A0: Digital;
    type A1: Digital;
    type A2: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O;
}

pub struct NoKernel3<A0, A1, A2, O> {
    _a0: std::marker::PhantomData<A0>,
    _a1: std::marker::PhantomData<A1>,
    _a2: std::marker::PhantomData<A2>,
    _o: std::marker::PhantomData<O>,
}

impl<A0, A1, A2, O> DigitalFn for NoKernel3<A0, A1, A2, O> {}
impl<A0: Digital, A1: Digital, A2: Digital, O: Digital> DigitalFn3 for NoKernel3<A0, A1, A2, O> {
    type A0 = A0;
    type A1 = A1;
    type A2 = A2;
    type O = O;

    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O {
        unimplemented!()
    }
}

pub trait DigitalFn4 {
    type A0: Digital;
    type A1: Digital;
    type A2: Digital;
    type A3: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3) -> Self::O;
}

pub trait DigitalFn5 {
    type A0: Digital;
    type A1: Digital;
    type A2: Digital;
    type A3: Digital;
    type A4: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3, Self::A4) -> Self::O;
}

pub trait DigitalFn6 {
    type A0: Digital;
    type A1: Digital;
    type A2: Digital;
    type A3: Digital;
    type A4: Digital;
    type A5: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3, Self::A4, Self::A5) -> Self::O;
}

// See: https://jsdw.me/posts/rust-fn-traits/

#[derive(Clone, PartialEq, Hash)]
pub struct DigitalSignature {
    pub arguments: Vec<Kind>,
    pub ret: Kind,
}

impl std::fmt::Debug for DigitalSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] -> {:?}",
            self.arguments
                .iter()
                .map(|k| format!("{k:?}"))
                .collect::<Vec<_>>()
                .join(", "),
            self.ret
        )
    }
}

pub trait Describable<Args> {
    fn describe() -> DigitalSignature;
}

macro_rules! describable {
    ($( $($arg:ident)* => $res:ident), +) => (
        $(
            impl <F, $res, $($arg),*> Describable<($res, $($arg),*)> for F
            where
            F: Fn($($arg),*) -> $res,
            $res: Digital,
            $ ($arg: Digital), *
            {
                fn describe() -> DigitalSignature {
                    DigitalSignature {
                        arguments: vec![$($arg::static_kind(),)*],
                        ret: $res::static_kind(),
                    }
                }
            }
        )+
    )
}

describable!(
    => T1,
    T1 => T2,
    T1 T2 => T3,
    T1 T2 T3 => T4,
    T1 T2 T3 T4 => T5,
    T1 T2 T3 T4 T5 => T6
);

pub fn inspect_digital<F, Args>(_f: F) -> DigitalSignature
where
    F: Describable<Args>,
{
    F::describe()
}

impl<N> DigitalFn for crate::rhdl_bits::Bits<N>
where
    N: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N::BITS))
    }
}

impl<N> DigitalFn for crate::rhdl_bits::SignedBits<N>
where
    N: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N::BITS))
    }
}

impl<N> DigitalFn for crate::rhdl_bits::bits<N>
where
    N: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N::BITS))
    }
}

impl<N> DigitalFn for crate::rhdl_bits::signed<N>
where
    N: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N::BITS))
    }
}
