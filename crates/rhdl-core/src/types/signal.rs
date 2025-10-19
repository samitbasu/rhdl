use rhdl_bits::{BitWidth, Bits};

use crate::{BitX, Digital, Domain, Kind, Timed};

/// A signal carrying a Digital value in a specific time domain.
///
/// The `Signal` struct is parameterized by a type `T` that implements
/// the `Digital` trait, representing the value type of the signal, and a
/// type `C` that implements the `Domain` trait, representing the time domain
/// of the signal.
///
/// In synthesizable code, you can usually just extract the underlying value with
/// `val()`, and then resynthesize it into a new signal using the [signal] function and
/// type inference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signal<T: Digital, C: Domain> {
    val: T,
    domain: std::marker::PhantomData<C>,
}

/// Create a new signal with the given value in the specified domain.
pub fn signal<T: Digital, C: Domain>(val: T) -> Signal<T, C> {
    Signal {
        val,
        domain: std::marker::PhantomData,
    }
}

impl<T: Digital, C: Domain> Signal<T, C> {
    /// Get the underlying value of the signal.
    ///
    /// Can be used in synthesizable context.
    pub fn val(&self) -> T {
        self.val
    }
    /// Get a mutable reference to the underlying value of the signal.
    ///
    /// Cannot be used in synthesizable context.
    pub fn val_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

impl<T: Digital, C: Domain> Timed for Signal<T, C> {}

impl<T: Digital, C: Domain> Digital for Signal<T, C> {
    const BITS: usize = T::BITS;
    fn static_kind() -> Kind {
        Kind::make_signal(T::static_kind(), C::color())
    }
    fn bin(self) -> Box<[BitX]> {
        self.val.bin()
    }
    fn dont_care() -> Self {
        Self {
            val: T::dont_care(),
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: Digital, C: Domain, const N: usize, const M: usize> std::ops::Index<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
where
    rhdl_bits::W<N>: BitWidth,
{
    type Output = T;

    fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
        &self.val[index.val]
    }
}

impl<T: Digital, C: Domain, const N: usize, const M: usize> std::ops::IndexMut<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
where
    rhdl_bits::W<N>: BitWidth,
{
    fn index_mut(&mut self, index: Signal<Bits<N>, C>) -> &mut Self::Output {
        &mut self.val[index.val]
    }
}

impl<T: Digital + std::ops::Not<Output = T>, C: Domain> std::ops::Not for Signal<T, C> {
    type Output = Signal<T, C>;

    fn not(self) -> Self::Output {
        Signal {
            val: std::ops::Not::not(self.val),
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: Digital + std::ops::Neg<Output = T>, C: Domain> std::ops::Neg for Signal<T, C> {
    type Output = Signal<T, C>;

    fn neg(self) -> Self::Output {
        Signal {
            val: std::ops::Neg::neg(self.val),
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: Digital, const M: usize, const N: usize, C: Domain> std::ops::Index<Signal<Bits<N>, C>>
    for [T; M]
where
    [T; M]: Digital,
    rhdl_bits::W<N>: BitWidth,
{
    type Output = T;

    fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
        &self[index.val]
    }
}

impl<T: Digital, const M: usize, const N: usize, C: Domain> std::ops::Index<Bits<N>>
    for Signal<[T; M], C>
where
    [T; M]: Digital,
    rhdl_bits::W<N>: BitWidth,
{
    type Output = T;

    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self.val[index]
    }
}
