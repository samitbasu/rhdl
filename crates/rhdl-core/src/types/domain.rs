/// Domain types represent different timing domains in the design.
/// Each domain type implements the Domain trait, which provides a color
/// marker for the domain.
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

/// Color markers for different time domains.
///
/// There is no a priori meaning to the colors; they are simply
/// distinct markers to differentiate domains.  You can organize
/// them however you like.  One option is to organize based on speed,
/// so that "Red" is the fastest domain, "Orange" is slower, and so on.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// Red domain
    Red,
    /// Orange domain
    Orange,
    /// Yellow domain
    Yellow,
    /// Green domain
    Green,
    /// Blue domain
    Blue,
    /// Indigo domain
    Indigo,
    /// Violet domain
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
