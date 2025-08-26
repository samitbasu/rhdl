use seq_macro::seq;

pub trait Digit: Copy + Default + 'static {
    const DIGIT_USIZE: usize = 0;
}

// Derive the 10 digits
seq!(N in 0..=9 {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
    pub struct D~N;

    impl D~N {
        pub fn new() -> Self {
            Self
        }
    }

    impl Digit for D~N {
        const DIGIT_USIZE: usize = N;
    }
});
