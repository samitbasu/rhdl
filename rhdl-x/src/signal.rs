use std::any::type_name;

use rhdl_bits::{alias::*, Bits};
use rhdl_core::{compile_design, Digital, DigitalFn, KernelFnKind, Kind, Notable, TypedBits};
use rhdl_macro::{kernel, Digital};

use crate::clock;

pub trait ClockType: Copy + PartialEq + 'static {}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Signal<T: Digital, C: ClockType> {
    val: T,
    clock: std::marker::PhantomData<C>,
}

impl<T: Digital, C: ClockType> Notable for Signal<T, C> {
    fn note(&self, key: impl rhdl_core::NoteKey, writer: impl rhdl_core::NoteWriter) {
        self.val.note(key, writer);
    }
}

impl<T: Digital, C: ClockType> Digital for Signal<T, C> {
    fn static_kind() -> Kind {
        Kind::make_struct(
            type_name::<Self>(),
            vec![Kind::make_field("val", T::static_kind())],
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

// We cannot have a blanket impl for Signal<T, C> + Signal<T, C> for any C,
// because we want to be able to handle the case that Signal<T, Async> + Signal<T, C> -> Signal<T, Async>.
// As a result, we use a macro to generate the impls for the specific cases we care about.  The macro
// takes 2 clock types, and generates an impl for Signal<T, C1> + Signal<T, C2> -> Signal<T, C1>.

// The generic case
impl<T: Digital + std::ops::Add<Output = T>, C: ClockType> std::ops::Add<Signal<T, C>>
    for Signal<T, C>
{
    type Output = Signal<T, C>;

    fn add(self, rhs: Signal<T, C>) -> Self::Output {
        Signal {
            val: self.val + rhs.val,
            clock: std::marker::PhantomData,
        }
    }
}

macro_rules! impl_add {
    ($C1:ty, $C2:ty, $C3: ty) => {
        impl<T: Digital + std::ops::Add<Output = T>> std::ops::Add<Signal<T, $C2>>
            for Signal<T, $C1>
        {
            type Output = Signal<T, $C3>;

            fn add(self, rhs: Signal<T, $C2>) -> Self::Output {
                Signal {
                    val: self.val + rhs.val,
                    clock: std::marker::PhantomData,
                }
            }
        }
    };
}

// The clock tree - we also use a macro here, to generate the clock structs
// The Macro takes a list of identifiers, and creates a struct and impl for
// each one.

macro_rules! clock_tree {
    ($($name:ident),*) => {
        $(
            #[derive(Copy, Clone, PartialEq, Debug)]
            pub struct $name;

            impl ClockType for $name {}
        )*
    };
}

clock_tree! {Const, Red, Orange, Yellow, Green, Blue, Indigo, Violet, Async}

macro_rules! mix_clocks {
    ($name: ident) => {
        $name!(Const, Const, Const);
    };
}

// Handle the case of mixing constant signals and
// a single clock.

impl_add!(Const, Red, Red);
impl_add!(Red, Const, Red);

impl_add!(Const, Orange, Orange);
impl_add!(Orange, Const, Orange);

impl_add!(Const, Yellow, Yellow);
impl_add!(Yellow, Const, Yellow);

impl_add!(Const, Green, Green);
impl_add!(Green, Const, Green);

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct MySignals<C1: ClockType, C2: ClockType> {
    pub input_stuff: Signal<b8, C1>,
    pub output_stuff: Signal<b8, C2>,
}

fn red<T: Digital>(val: T) -> Signal<T, Red> {
    Signal {
        val,
        clock: std::marker::PhantomData,
    }
}

/*
impl<T: Digital, C: ClockType> std::ops::Add<Signal<T, Async>> for Signal<T, C> {
    type Output = Signal<T, Async>;

    fn add(self, rhs: T) -> Self::Output {
        Signal {
            val: self.val + rhs,
            clock: std::marker::PhantomData,
        }
    }
}
*/

#[kernel]
fn add_stuff<C: ClockType>(x: Signal<b4, C>, y: Signal<b4, C>) -> Signal<b4, C> {
    let y = match x.val {
        Bits::<4>(3) => y,
        _ => x + y,
    };
    y
}

#[test]
fn test_dump_add_stuff() {
    // Compile it:
    let Some(KernelFnKind::Kernel(kernel)) = add_stuff::<Red>::kernel_fn() else {
        panic!("No kernel function found");
    };
    compile_design(kernel).unwrap();
}
