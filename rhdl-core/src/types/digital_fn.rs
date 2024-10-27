use serde::{Deserialize, Serialize};

pub use crate::{kernel::KernelFnKind, Digital, Kind};

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

pub trait DigitalFn3 {
    type A0: Digital;
    type A1: Digital;
    type A2: Digital;
    type O: Digital;
    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O;
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

#[derive(Clone, PartialEq, Serialize, Deserialize, Hash)]
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
                .map(|k| format!("{:?}", k))
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

impl<const N: usize> DigitalFn for rhdl_bits::Bits<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::SignedBits<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::bits<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::signed<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N))
    }
}
