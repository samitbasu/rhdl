use std::{any::type_name, process::Output};

use crate::{types::clock::ClockType, Digital, Kind, Notable, NoteKey, NoteWriter, TypedBits};
use rhdl_bits::Bits;
use rhdl_bits::SignedBits;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug)]
pub struct Signal<T: Digital, C: ClockType> {
    val: T,
    clock: std::marker::PhantomData<C>,
}

impl<T: Digital, C: ClockType> Signal<T, C> {
    pub fn val(&self) -> T {
        self.val
    }

    pub fn new(val: T) -> Self {
        Self {
            val,
            clock: std::marker::PhantomData,
        }
    }
}

impl<T: Digital, C: ClockType> Notable for Signal<T, C> {
    fn note(&self, key: impl NoteKey, writer: impl NoteWriter) {
        self.val.note(key, writer);
    }
}

impl<T: Digital, C: ClockType> Digital for Signal<T, C> {
    fn static_kind() -> Kind {
        Kind::make_struct(
            type_name::<Self>(),
            vec![
                Kind::make_field("val", T::static_kind()),
                Kind::make_field("clock", Kind::Clock(C::color())),
            ],
        )
    }
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn bin(self) -> Vec<bool> {
        self.val.bin()
    }
    fn typed_bits(self) -> TypedBits {
        self.val.typed_bits()
    }
    fn discriminant(self) -> TypedBits {
        self.val.discriminant()
    }
    fn variant_kind(self) -> Kind {
        self.val.variant_kind()
    }
    fn binary_string(self) -> String {
        self.val.binary_string()
    }
}

macro_rules! impl_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: ClockType> std::ops::$trait for Signal<T, C> {
            fn $op(&mut self, rhs: Signal<T, C>) {
                std::ops::$trait::$op(&mut self.val, rhs.val);
            }
        }
    };
}

macro_rules! impl_shift_assign_op {
    ($trait: ident, $op: ident) => {
        impl<T: Digital + std::ops::$trait, C: ClockType> std::ops::$trait<T> for Signal<T, C> {
            fn $op(&mut self, rhs: T) {
                std::ops::$trait::$op(&mut self.val, rhs);
            }
        }
    };
}

macro_rules! impl_cmpop {
    ($trait: ident, $op: ident, $ret: ty) => {
        // Case for signal == signal
        impl<T: Digital + std::cmp::$trait, C: ClockType> std::cmp::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            fn $op(&self, rhs: &Signal<T, C>) -> $ret {
                std::cmp::$trait::$op(&self.val, &rhs.val)
            }
        }

        // Case for signal == constant
        impl<T: Digital + std::cmp::$trait, C: ClockType> std::cmp::$trait<T> for Signal<T, C> {
            fn $op(&self, rhs: &T) -> $ret {
                std::cmp::$trait::$op(&self.val, rhs)
            }
        }

        // Case for constant == signal
        impl<const N: usize, C: ClockType> std::cmp::$trait<Signal<Bits<N>, C>> for Bits<N> {
            fn $op(&self, rhs: &Signal<Bits<N>, C>) -> $ret {
                std::cmp::$trait::$op(self, &rhs.val)
            }
        }

        // Case for signed == signal
        impl<const N: usize, C: ClockType> std::cmp::$trait<Signal<SignedBits<N>, C>>
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
        // Case for signal << signal
        impl<T: Digital + std::ops::$trait<Output = T>, C: ClockType> std::ops::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: Signal<T, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }

        // Case for signal << constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: ClockType> std::ops::$trait<T>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    clock: std::marker::PhantomData,
                }
            }
        }

        // Case for constant << signal
        impl<const N: usize, C: ClockType> std::ops::$trait<Signal<Bits<N>, C>> for Bits<N> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }

        // Case for signed << signal
        impl<const N: usize, C: ClockType> std::ops::$trait<Signal<Bits<N>, C>> for SignedBits<N> {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }
    };
}

macro_rules! impl_binop {
    ($trait: ident, $op: ident) => {
        // Case for signal + signal
        impl<T: Digital + std::ops::$trait<Output = T>, C: ClockType> std::ops::$trait<Signal<T, C>>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: Signal<T, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }

        // Case for signal + constant
        impl<T: Digital + std::ops::$trait<Output = T>, C: ClockType> std::ops::$trait<T>
            for Signal<T, C>
        {
            type Output = Signal<T, C>;

            fn $op(self, rhs: T) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self.val, rhs),
                    clock: std::marker::PhantomData,
                }
            }
        }

        // Case for constant + signal
        impl<const N: usize, C: ClockType> std::ops::$trait<Signal<Bits<N>, C>> for Bits<N> {
            type Output = Signal<Bits<N>, C>;

            fn $op(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }

        impl<const N: usize, C: ClockType> std::ops::$trait<Signal<SignedBits<N>, C>>
            for SignedBits<N>
        {
            type Output = Signal<SignedBits<N>, C>;

            fn $op(self, rhs: Signal<SignedBits<N>, C>) -> Self::Output {
                Signal {
                    val: std::ops::$trait::$op(self, rhs.val),
                    clock: std::marker::PhantomData,
                }
            }
        }
    };
}

impl<T: Digital + std::ops::Not<Output = T>, C: ClockType> std::ops::Not for Signal<T, C> {
    type Output = Signal<T, C>;

    fn not(self) -> Self::Output {
        Signal {
            val: std::ops::Not::not(self.val),
            clock: std::marker::PhantomData,
        }
    }
}

impl<T: Digital + std::ops::Neg<Output = T>, C: ClockType> std::ops::Neg for Signal<T, C> {
    type Output = Signal<T, C>;

    fn neg(self) -> Self::Output {
        Signal {
            val: std::ops::Neg::neg(self.val),
            clock: std::marker::PhantomData,
        }
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
