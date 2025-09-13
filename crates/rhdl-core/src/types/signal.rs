use rhdl_bits::{BitWidth, Bits};

use crate::{BitX, Digital, Domain, Kind, Timed};
use rhdl_trace_type as rtt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Signal<T: Digital, C: Domain> {
    val: T,
    domain: std::marker::PhantomData<C>,
}

pub fn signal<T: Digital, C: Domain>(val: T) -> Signal<T, C> {
    Signal {
        val,
        domain: std::marker::PhantomData,
    }
}

impl<T: Digital, C: Domain> Signal<T, C> {
    pub fn val(&self) -> T {
        self.val
    }
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
    fn static_trace_type() -> rtt::TraceType {
        rtt::make_signal(T::static_trace_type(), C::color().into())
    }

    fn bin(self) -> Vec<BitX> {
        self.val.bin()
    }

    fn dont_care() -> Self {
        Self {
            val: T::dont_care(),
            domain: std::marker::PhantomData,
        }
    }
}

/* macro_rules! impl_index {
    ($M: expr) => {
        impl<T: Digital, C: Domain, N: BitWidth> std::ops::Index<Signal<Bits<N>, C>>
            for Signal<[T; $M], C>
        {
            type Output = T;

            fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
                &self.val[index.val]
            }
        }

        impl<T: Digital, C: Domain, N: BitWidth> std::ops::IndexMut<Signal<Bits<N>, C>>
            for Signal<[T; $M], C>
        {
            fn index_mut(&mut self, index: Signal<Bits<N>, C>) -> &mut Self::Output {
                &mut self.val[index.val]
            }
        }
    };
}

impl_index!(1);
impl_index!(2);
impl_index!(3);
impl_index!(4);
impl_index!(5);
impl_index!(6);
impl_index!(7);
impl_index!(8);
*/

impl<T: Digital, C: Domain, N: BitWidth, const M: usize> std::ops::Index<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
{
    type Output = T;

    fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
        &self.val[index.val]
    }
}

impl<T: Digital, C: Domain, N: BitWidth, const M: usize> std::ops::IndexMut<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
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

impl<T: Digital, const M: usize, N: BitWidth, C: Domain> std::ops::Index<Signal<Bits<N>, C>>
    for [T; M]
where
    [T; M]: Digital,
{
    type Output = T;

    fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
        &self[index.val]
    }
}

impl<T: Digital, const M: usize, N: BitWidth, C: Domain> std::ops::Index<Bits<N>>
    for Signal<[T; M], C>
where
    [T; M]: Digital,
{
    type Output = T;

    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self.val[index]
    }
}
