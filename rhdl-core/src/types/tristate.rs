use crate::Notable;

pub trait Tristate: Clone + Copy + Default + Notable {
    const N: usize;
}

impl Tristate for () {
    const N: usize = 0;
}

impl<A: Tristate, B: Tristate> Tristate for (A, B) {
    const N: usize = A::N + B::N;
}
