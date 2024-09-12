use serde::{Deserialize, Serialize};

pub trait Domain: Copy + PartialEq + 'static + Default {
    fn color() -> Color;
}

// Given a list of names, generates a clock for each
macro_rules! decl_domains {
    ($($name: ident),*) => {
        $(decl_domain!($name);)*
    };
    () => {

    };
}

macro_rules! decl_domain {
    ($name: ident) => {
        #[derive(Copy, Clone, Debug, PartialEq, Default)]
        pub struct $name;

        impl Domain for $name {
            fn color() -> Color {
                Color::$name
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Indigo,
    Violet,
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print only first letter in lower case
        match self {
            Color::Red => write!(f, "r"),
            Color::Orange => write!(f, "o"),
            Color::Yellow => write!(f, "y"),
            Color::Green => write!(f, "g"),
            Color::Blue => write!(f, "b"),
            Color::Indigo => write!(f, "i"),
            Color::Violet => write!(f, "v"),
        }
    }
}

decl_domains!(Red, Orange, Yellow, Green, Blue, Indigo, Violet);
