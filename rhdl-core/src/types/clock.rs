use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ClockColor {
    Black,
    Brown,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Violet,
    Grey,
    White,
}

decl_clocks!(Black, Brown, Red, Orange, Yellow, Green, Blue, Violet, Grey, White);
