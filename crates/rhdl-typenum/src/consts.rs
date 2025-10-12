use seq_macro::seq;

use super::prelude::Unsigned;

seq!(N in 1..=128 {
    /// Type-level unsigned integer constant.
    #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Debug, Default)]
    pub struct U~N;

    impl Unsigned for U~N {
        const USIZE: usize = N;
    }
});
