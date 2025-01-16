use seq_macro::seq;

use crate::digits::*;
use crate::{UInt, UTerm};

pub type U0 = UTerm;

seq!(N in 1..=9 {
    pub type U~N = UInt<UTerm, D~N>;
});

seq!(M in 1..=9 {
    seq!(N in 0..=9 {
        pub type U~M~N = UInt<UInt<UTerm, D~M>, D~N>;
    });
});

seq!(K in 1..=9 {
    seq!(M in 0..=9 {
        seq!(N in 0..=9 {
            pub type U~K~M~N = UInt<UInt<UInt<UTerm,D~K>,D~M>,D~N>;
        });
    });
});
