use serde::{Deserialize, Serialize};

pub trait Clock: Copy + PartialEq + 'static {
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

        impl Clock for $name {
            fn color() -> ClockColor {
                ClockColor::$name
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ClockColor {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}

impl std::fmt::Debug for ClockColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print only first letter in lower case
        match self {
            ClockColor::Red => write!(f, "r"),
            ClockColor::Orange => write!(f, "o"),
            ClockColor::Yellow => write!(f, "y"),
            ClockColor::Green => write!(f, "g"),
            ClockColor::Blue => write!(f, "b"),
            ClockColor::Indigo => write!(f, "i"),
            ClockColor::Violet => write!(f, "v"),
        }
    }
}

decl_clocks!(Red, Orange, Yellow, Green, Blue, Indigo, Violet);
