use std::any::type_name;

use crate::{types::clock::ClockType, Digital, Kind, Notable, NoteKey, NoteWriter, TypedBits};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Signal<T: Digital, C: ClockType> {
    val: T,
    clock: std::marker::PhantomData<C>,
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
    };
}

impl_binop!(Add, add);
impl_binop!(Sub, sub);
impl_binop!(Mul, mul);
impl_binop!(BitAnd, bitand);
impl_binop!(BitOr, bitor);
impl_binop!(BitXor, bitxor);
