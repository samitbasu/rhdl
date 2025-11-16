#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

use rhdl_bits::BitWidth;

pub use crate::{Digital, Kind, kernel::KernelFnKind};

/// The trait used to describe synthesizable functions
///
/// Any synthesizable function should include a data structure
/// with the same name that implements this trait.
///
pub trait DigitalFn {
    /// If this DigitalFn has a kernel function associated with it,
    /// return its kind here.
    fn kernel_fn() -> Option<KernelFnKind> {
        None
    }
}

impl DigitalFn for () {}

/// A synthesizable function with no arguments.
pub trait DigitalFn0 {
    /// The output type of the function.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn() -> Self::O;
}

/// A synthesizable function with one argument.
pub trait DigitalFn1 {
    /// The argument type.
    type A0: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0) -> Self::O;
}

/// A synthesizable function with two arguments.
pub trait DigitalFn2 {
    /// The first argument type.
    type A0: Digital;
    /// The second argument type.
    type A1: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0, Self::A1) -> Self::O;
}

/// A placeholder DigitalFn2 implementation for functions without kernels.
pub struct NoCircuitKernel<A0, A1, O> {
    _a0: std::marker::PhantomData<A0>,
    _a1: std::marker::PhantomData<A1>,
    _o: std::marker::PhantomData<O>,
}

impl<A0, A1, O> DigitalFn for NoCircuitKernel<A0, A1, O> {}

impl<A0: Digital, A1: Digital, O: Digital> DigitalFn2 for NoCircuitKernel<A0, A1, O> {
    type A0 = A0;
    type A1 = A1;
    type O = O;

    fn func() -> fn(Self::A0, Self::A1) -> Self::O {
        unimplemented!()
    }
}

/// A synthesizable function with three arguments.
pub trait DigitalFn3 {
    /// The first argument type.
    type A0: Digital;
    /// The second argument type.
    type A1: Digital;
    /// The third argument type.
    type A2: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O;
}

/// A placeholder DigitalFn3 implementation for functions without kernels.
pub struct NoSynchronousKernel<A0, A1, A2, O> {
    _a0: std::marker::PhantomData<A0>,
    _a1: std::marker::PhantomData<A1>,
    _a2: std::marker::PhantomData<A2>,
    _o: std::marker::PhantomData<O>,
}

impl<A0, A1, A2, O> DigitalFn for NoSynchronousKernel<A0, A1, A2, O> {}
impl<A0: Digital, A1: Digital, A2: Digital, O: Digital> DigitalFn3
    for NoSynchronousKernel<A0, A1, A2, O>
{
    type A0 = A0;
    type A1 = A1;
    type A2 = A2;
    type O = O;

    fn func() -> fn(Self::A0, Self::A1, Self::A2) -> Self::O {
        unimplemented!()
    }
}

/// A synthesizable function with four arguments.
pub trait DigitalFn4 {
    /// The first argument type.
    type A0: Digital;
    /// The second argument type.
    type A1: Digital;
    /// The third argument type.
    type A2: Digital;
    /// The fourth argument type.
    type A3: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3) -> Self::O;
}

/// A synthesizable function with five arguments.
pub trait DigitalFn5 {
    /// The first argument type.
    type A0: Digital;
    /// The second argument type.
    type A1: Digital;
    /// The third argument type.
    type A2: Digital;
    /// The fourth argument type.
    type A3: Digital;
    /// The fifth argument type.
    type A4: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3, Self::A4) -> Self::O;
}

/// A synthesizable function with six arguments.
pub trait DigitalFn6 {
    /// The first argument type.
    type A0: Digital;
    /// The second argument type.
    type A1: Digital;
    /// The third argument type.
    type A2: Digital;
    /// The fourth argument type.
    type A3: Digital;
    /// The fifth argument type.
    type A4: Digital;
    /// The sixth argument type.
    type A5: Digital;
    /// The output type.
    type O: Digital;
    /// The function pointer.
    fn func() -> fn(Self::A0, Self::A1, Self::A2, Self::A3, Self::A4, Self::A5) -> Self::O;
}

/// A description of a Digital function's signature.
/// See: https://jsdw.me/posts/rust-fn-traits/
#[derive(Clone, PartialEq, Hash)]
pub struct DigitalSignature {
    /// The argument [Kind]s.
    pub arguments: Vec<Kind>,
    /// The return [Kind].
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

/// A trait used to describe synthesizable functions.
pub trait Describable<Args> {
    /// Describe the function signature.
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
                        arguments: vec![$($arg::static_kind()),*],
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

/// Inspect the Digital signature of a function.
pub fn inspect_digital<F, Args>(_f: F) -> DigitalSignature
where
    F: Describable<Args>,
{
    F::describe()
}

impl<const N: usize> DigitalFn for rhdl_bits::Bits<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::SignedBits<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::bits<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::BitConstructor(N))
    }
}

impl<const N: usize> DigitalFn for rhdl_bits::signed<N>
where
    rhdl_bits::W<N>: BitWidth,
{
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(KernelFnKind::SignedBitsConstructor(N))
    }
}
