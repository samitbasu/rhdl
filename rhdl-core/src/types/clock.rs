use super::kind::ClockColor;

pub trait ClockType: Copy + PartialEq + 'static {
    fn color() -> ClockColor;
}

// Given a list of names, generates a clock for each
macro_rules! decl_clocks {
    ($($name: ident),*) => {
        $(decl_clock!($name);)*
    };
    () => {

    };
}

macro_rules! decl_clock {
    ($name: ident) => {
        #[derive(Copy, Clone, PartialEq)]
        pub struct $name;

        impl ClockType for $name {
            fn color() -> ClockColor {
                ClockColor::$name
            }
        }
    };
}

decl_clocks!(Red, Orange, Yellow, Green, Blue, Indigo, Violet);
