use rhdl_bits::{alias::*, Bits};
use rhdl_core::{compile_design, Digital, DigitalFn, KernelFnKind, Kind, Notable, TypedBits};
use rhdl_macro::{kernel, Digital};

pub trait ClockType: Copy + PartialEq + 'static {}

#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct Signal<T: Digital, C: ClockType> {
    val: T,
    clock: std::marker::PhantomData<C>,
}
/*
impl<T: Digital, C: ClockType> Notable for Signal<T, C> {
    fn note(&self, key: impl rhdl_core::NoteKey, writer: impl rhdl_core::NoteWriter) {
        self.val.note(key, writer);
    }
}

impl<T: Digital, C: ClockType> Digital for Signal<T, C> {
    fn static_kind() -> Kind {
        T::static_kind()
    }
    fn bits() -> usize {
        Self::static_kind().bits()
    }
    fn kind(&self) -> Kind {
        Self::static_kind()
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
*/
#[derive(Copy, Clone, PartialEq, Debug, Digital)]
pub struct MySignals<C1: ClockType, C2: ClockType> {
    pub input_stuff: Signal<b8, C1>,
    pub output_stuff: Signal<b8, C2>,
}

impl<const N: usize, C: ClockType> std::ops::Add for Signal<Bits<N>, C> {
    type Output = Signal<Bits<N>, C>;

    fn add(self, rhs: Signal<Bits<N>, C>) -> Self::Output {
        Signal {
            val: self.val + rhs.val,
            clock: std::marker::PhantomData,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Red;

fn red<T: Digital>(val: T) -> Signal<T, Red> {
    Signal {
        val,
        clock: std::marker::PhantomData,
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Green;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Async;

impl ClockType for Red {}

impl ClockType for Green {}

impl ClockType for Async {}

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
