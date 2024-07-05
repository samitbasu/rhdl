use crate::Notable;

pub trait Tristate: Clone + Copy + Default + Notable {
    const N: usize;
}

impl Tristate for () {
    const N: usize = 0;
}
