pub trait BitWidth {}

pub struct W<const N: usize>;

seq_macro::seq!(N in 1..=128 {
    #(
        impl BitWidth for W<N> {}
    )*
});
