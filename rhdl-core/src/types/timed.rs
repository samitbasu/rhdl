use crate::{Digital, Domain, Kind, Notable};
use rhdl_bits::{Bits, SignedBits};
use std::cmp::Ordering;

pub trait Timed: Copy + Sized + PartialEq + Clone + 'static + Notable {
    fn static_kind() -> Kind;
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
    }
}

impl<T: Digital> Timed for T {
    fn static_kind() -> Kind {
        T::static_kind()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sig<T: Digital, C: Domain> {
    val: T,
    domain: std::marker::PhantomData<C>,
}

pub fn signal<T: Digital, C: Domain>(val: T) -> Sig<T, C> {
    Sig {
        val,
        domain: std::marker::PhantomData,
    }
}

impl<T: Digital, C: Domain> Sig<T, C> {
    pub fn val(&self) -> T {
        self.val
    }
}

impl<T: Digital, C: Domain> Timed for Sig<T, C> {
    fn static_kind() -> Kind {
        Kind::make_signal(T::static_kind(), C::color())
    }
}

impl<T: Digital, C: Domain> Notable for Sig<T, C> {
    fn note(&self, key: impl crate::NoteKey, writer: impl crate::NoteWriter) {
        self.val.note(key, writer);
    }
}

impl<T: Digital, C: Domain, const M: usize, const N: usize> std::ops::Index<Sig<Bits<N>, C>>
    for Sig<[T; M], C>
{
    type Output = T;

    fn index(&self, index: Sig<Bits<N>, C>) -> &Self::Output {
        &self.val[index.val]
    }
}

impl<T: Digital, C: Domain, const M: usize, const N: usize> std::ops::IndexMut<Sig<Bits<N>, C>>
    for Sig<[T; M], C>
{
    fn index_mut(&mut self, index: Sig<Bits<N>, C>) -> &mut Self::Output {
        &mut self.val[index.val]
    }
}

macro_rules! impl_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: Domain> std::ops::$trait for Sig<T, C> {
            fn $op(&mut self, rhs: Sig<T, C>) {
                std::ops::$trait::$op(&mut self.val, rhs.val);
            }
        }
    };
}

macro_rules! impl_shift_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: Domain> std::ops::$trait<T> for Sig<T, C> {
            fn $op(&mut self, rhs: T) {
                std::ops::$trait::$op(&mut self.val, rhs);
            }
        }
    };
}

macro_rules! impl_cmpop {
    ($trait: ident, $op: ident, $ret: ty) => {
        // Case for Sig == Sig
        impl<T: Digital + std::cmp::$trait, C: Domain> std::cmp::$trait<Sig<T, C>> for Sig<T, C> {
            fn $op(&self, rhs: &Sig<T, C>) -> $ret {
                std::cmp::$trait::$op(&self.val, &rhs.val)
            }
        }

        // Case for Sig == literal (unsigned)
        impl<const N: usize, C: Domain> std::cmp::$trait<u128> for Sig<Bits<N>, C> {
            fn $op(&self, rhs: &u128) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for Sig == literal (signed)
        impl<const N: usize, C: Domain> std::cmp::$trait<i128> for Sig<SignedBits<N>, C> {
            fn $op(&self, rhs: &i128) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for literal == Sig (unsigned)
        impl<const N: usize, C: Domain> std::cmp::$trait<Sig<Bits<N>, C>> for u128 {
            fn $op(&self, rhs: &Sig<Bits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for literal == Sig (signed)
        impl<const N: usize, C: Domain> std::cmp::$trait<Sig<SignedBits<N>, C>> for i128 {
            fn $op(&self, rhs: &Sig<SignedBits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for Sig == constant
        impl<T: Digital + std::cmp::$trait, C: Domain> std::cmp::$trait<T> for Sig<T, C> {
            fn $op(&self, rhs: &T) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for constant == Sig
        impl<const N: usize, C: Domain> std::cmp::$trait<Sig<Bits<N>, C>> for Bits<N> {
            fn $op(&self, rhs: &Sig<Bits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for signed == Sig
        impl<const N: usize, C: Domain> std::cmp::$trait<Sig<SignedBits<N>, C>> for SignedBits<N> {
            fn $op(&self, rhs: &Sig<SignedBits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }
    };
}

macro_rules! impl_shiftop {
    ($trait: ident, $op: ident) => {
        // Case for Sig << Sig
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<Sig<T, C>>
            for Sig<T, C>
        {
            type Output = Sig<T, C>;

            fn $op(self, rhs: Sig<T, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Sig << literal
        impl<const N: usize, C: Domain> std::ops::$trait<u128> for Sig<Bits<N>, C> {
            type Output = Sig<Bits<N>, C>;

            fn $op(self, rhs: u128) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Sig << constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<T>
            for Sig<T, C>
        {
            type Output = Sig<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for constant << Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Sig<Bits<N>, C>> for Bits<N> {
            type Output = Sig<Bits<N>, C>;

            fn $op(self, rhs: Sig<Bits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for signed << Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Sig<Bits<N>, C>> for SignedBits<N> {
            type Output = Sig<SignedBits<N>, C>;

            fn $op(self, rhs: Sig<Bits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! impl_binop {
    ($trait: ident, $op: ident) => {
        // Case for Sig + Sig
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<Sig<T, C>>
            for Sig<T, C>
        {
            type Output = Sig<T, C>;

            fn $op(self, rhs: Sig<T, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Sig + literal
        impl<const N: usize, C: Domain> std::ops::$trait<u128> for Sig<Bits<N>, C> {
            type Output = Sig<Bits<N>, C>;

            fn $op(self, rhs: u128) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Sig + literal (signed)
        impl<const N: usize, C: Domain> std::ops::$trait<i128> for Sig<SignedBits<N>, C> {
            type Output = Sig<SignedBits<N>, C>;

            fn $op(self, rhs: i128) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for literal + Sig (unsigned)
        impl<const N: usize, C: Domain> std::ops::$trait<Sig<Bits<N>, C>> for u128 {
            type Output = Sig<Bits<N>, C>;

            fn $op(self, rhs: Sig<Bits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for literal + Sig (signed)
        impl<const N: usize, C: Domain> std::ops::$trait<Sig<SignedBits<N>, C>> for i128 {
            type Output = Sig<SignedBits<N>, C>;

            fn $op(self, rhs: Sig<SignedBits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for Sig + constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: Domain> std::ops::$trait<T>
            for Sig<T, C>
        {
            type Output = Sig<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self.val, rhs),
                    domain: std::marker::PhantomData,
                }
            }
        }

        // Case for constant + Sig
        impl<const N: usize, C: Domain> std::ops::$trait<Sig<Bits<N>, C>> for Bits<N> {
            type Output = Sig<Bits<N>, C>;

            fn $op(self, rhs: Sig<Bits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }

        impl<const N: usize, C: Domain> std::ops::$trait<Sig<SignedBits<N>, C>> for SignedBits<N> {
            type Output = Sig<SignedBits<N>, C>;

            fn $op(self, rhs: Sig<SignedBits<N>, C>) -> Self::Output {
                Sig {
                    val: std::ops::$trait::$op(self, rhs.val),
                    domain: std::marker::PhantomData,
                }
            }
        }
    };
}

impl<T: Digital + std::ops::Not<Output = T>, C: Domain> std::ops::Not for Sig<T, C> {
    type Output = Sig<T, C>;

    fn not(self) -> Self::Output {
        Sig {
            val: std::ops::Not::not(self.val),
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: Digital + std::ops::Neg<Output = T>, C: Domain> std::ops::Neg for Sig<T, C> {
    type Output = Sig<T, C>;

    fn neg(self) -> Self::Output {
        Sig {
            val: std::ops::Neg::neg(self.val),
            domain: std::marker::PhantomData,
        }
    }
}

impl<T: Digital, const M: usize, const N: usize, C: Domain> std::ops::Index<Sig<Bits<N>, C>>
    for [T; M]
where
    [T; M]: Digital,
{
    type Output = T;

    fn index(&self, index: Sig<Bits<N>, C>) -> &Self::Output {
        &self[index.val]
    }
}

impl<T: Digital, const M: usize, const N: usize, C: Domain> std::ops::Index<Bits<N>>
    for Sig<[T; M], C>
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
