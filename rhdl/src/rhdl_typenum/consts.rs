use super::{
    digits::*,
    unsigned::{T_, U_},
};
use seq_macro::seq;

pub type U0 = T_;

seq!(N in 1..=9 {
    pub type U~N = U_<T_, D~N>;
});

seq!(M in 1..=9 {
    seq!(N in 0..=9 {
        pub type U~M~N = U_<U_<T_, D~M>, D~N>;
    });
});

seq!(K in 1..=9 {
    seq!(M in 0..=9 {
        seq!(N in 0..=9 {
            pub type U~K~M~N = U_<U_<U_<T_,D~K>,D~M>,D~N>;
        });
    });
});
