use std::cmp::Ordering;

use rhdl_bits::Bits;
use rhdl_bits::SignedBits;

use crate::{Digital, Domain, Kind, Timed};

#[derive(Clone, Copy, Debug)]
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

    fn bin(self) -> Vec<bool> {
        self.val.bin()
    }

    fn init() -> Self {
        Self {
            val: T::init(),
            domain: std::marker::PhantomData,
        }
    }
}

/* macro_rules! impl_index {
    ($M: expr) => {
        impl<T: Digital, C: Domain, const N: usize> std::ops::Index<Signal<Bits<N>, C>>
            for Signal<[T; $M], C>
        {
            type Output = T;

            fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
                &self.val[index.val]
            }
        }

        impl<T: Digital, C: Domain, const N: usize> std::ops::IndexMut<Signal<Bits<N>, C>>
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

impl<T: Digital, C: Domain, const M: usize, const N: usize> std::ops::Index<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
{
    type Output = T;

    fn index(&self, index: Signal<Bits<N>, C>) -> &Self::Output {
        &self.val[index.val]
    }
}

impl<T: Digital, C: Domain, const M: usize, const N: usize> std::ops::IndexMut<Signal<Bits<N>, C>>
    for Signal<[T; M], C>
{
    fn index_mut(&mut self, index: Signal<Bits<N>, C>) -> &mut Self::Output {
        &mut self.val[index.val]
    }
}

macro_rules! impl_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: Domain> std::ops::$trait for Signal<T, C> {
            fn $op(&mut self, rhs: Signal<T, C>) {
                std::ops::$trait::$op(&mut self.val, rhs.val);
            }
        }
    };
}

macro_rules! impl_shift_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: Domain> std::ops::$trait<T> for Signal<T, C> {
            fn $op(&mut self, rhs: T) {
                std::ops::$trait::$op(&mut self.val, rhs);
            }
        }
    };
}

macro_rules! impl_cmpop {
    ($trait: ident, $op: ident, $ret: ty) => {
        // Case for Signal == Sig
        impl<T: Digital + std::cmp::$trait, C: Domain> std::cmp::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            fn $op(&self, rhs: &Signal<T, C>) -> $ret {
                std::cmp::$trait::$op(&self.val, &rhs.val)
            }
        }

        // Case for Signal == literal (unsigned)
        impl<const N: usize, C: Domain> std::cmp::$trait<u128> for Signal<Bits<N>, C> {
            fn $op(&self, rhs: &u128) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for Signal == literal (signed)
        impl<const N: usize, C: Domain> std::cmp::$trait<i128> for Signal<SignedBits<N>, C> {
            fn $op(&self, rhs: &i128) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for literal == Signal (unsigned)
        impl<const N: usize, C: Domain> std::cmp::$trait<Signal<Bits<N>, C>> for u128 {
            fn $op(&self, rhs: &Signal<Bits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for literal == Signal (signed)
        impl<const N: usize, C: Domain> std::cmp::$trait<Signal<SignedBits<N>, C>> for i128 {
            fn $op(&self, rhs: &Signal<SignedBits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for Signal == constant
        impl<T: Digital + std::cmp::$trait, C: Domain> std::cmp::$trait<T> for Signal<T, C> {
            fn $op(&self, rhs: &T) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for constant == Sig
        impl<const N: usize, C: Domain> std::cmp::$trait<Signal<Bits<N>, C>> for Bits<N> {
            fn $op(&self, rhs: &Signal<Bits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for signed == Sig
        impl<const N: usize, C: Domain> std::cmp::$trait<Signal<SignedBits<N>, C>>
            for SignedBits<N>
        {
            fn $op(&self, rhs: &Signal<SignedBits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }
    };
}

macro_rules! impl_shiftop {
    ($trait: ident, $op: ident) => {
        // Case for Signal << Sig
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: Signal<T, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Signal << literal
        impl<const N: usize, C: Domain> std::ops::$trait<u128> for Signal<Bits<N>, C> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: u128) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Signal << constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<T>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for constant << Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Signal<Bits<N>, C>> for Bits<N> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for signed << Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Signal<Bits<N>, C>> for SignedBits<N> {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! impl_binop {
    ($trait: ident, $op: ident) => {
        // Case for Signal + Sig
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: Signal<T, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Signal + literal
        impl<const N: usize, C: Domain> std::ops::$trait<u128> for Signal<Bits<N>, C> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: u128) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Signal + literal (signed)
        impl<const N: usize, C: Domain> std::ops::$trait<i128> for Signal<SignedBits<N>, C> {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: i128) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for literal + Signal (unsigned)
        impl<const N: usize, C: Domain> std::ops::$trait<Signal<Bits<N>, C>> for u128 {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for literal + Signal (signed)
        impl<const N: usize, C: Domain> std::ops::$trait<Signal<SignedBits<N>, C>> for i128 {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: Signal<SignedBits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Signal + constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<T>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for constant + Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Signal<Bits<N>, C>> for Bits<N> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        impl<const N: usize, C: Domain> std::ops::$trait<Signal<SignedBits<N>, C>>
            for SignedBits<N>
        {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: Signal<SignedBits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }
    };
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
{
    type Output = T;

    fn index(&self, index: Bits<N>) -> &Self::Output {
        &self.val[index]
    }
}

impl_binop!(Add, add);
impl_binop!(Sub, sub);
impl_binop!(BitAnd, bitand);
impl_binop!(BitOr, bitor);
impl_binop!(BitXor, bitxor);
impl_shiftop!(Shl, shl);
impl_shiftop!(Shr, shr);
impl_assign_op!(AddAssign, add_assign);
impl_assign_op!(SubAssign, sub_assign);
impl_assign_op!(BitAndAssign, bitand_assign);
impl_assign_op!(BitOrAssign, bitor_assign);
impl_assign_op!(BitXorAssign, bitxor_assign);
impl_shift_assign_op!(ShlAssign, shl_assign);
impl_shift_assign_op!(ShrAssign, shr_assign);
impl_cmpop!(PartialEq, eq, bool);
impl_cmpop!(PartialOrd, partial_cmp, Option<Ordering>);
