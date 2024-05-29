use serde::{Deserialize, Serialize};

pub use crate::{kernel::KernelFnKind, Digital, Kind};

pub trait DigitalFn {
    fn kernel_fn() -> Option<KernelFnKind> {
        None
    }
}

// See: https://jsdw.me/posts/rust-fn-traits/

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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
