use crate::BitZ;

pub trait Tristate: Clone + Copy + Default {
    fn bits() -> usize;
}

impl<const N: usize> Tristate for BitZ<N> {
    fn bits() -> usize {
        N
    }
}

impl Tristate for () {
    fn bits() -> usize {
        0
    }
}

impl<T: Tristate> Tristate for (T,) {
    fn bits() -> usize {
        T::bits()
    }
}

impl<T0: Tristate, T1: Tristate> Tristate for (T0, T1) {
    fn bits() -> usize {
        T0::bits() + T1::bits()
    }
}

impl<T0: Tristate, T1: Tristate, T2: Tristate> Tristate for (T0, T1, T2) {
    fn bits() -> usize {
        T0::bits() + T1::bits() + T2::bits()
    }
}

impl<T0: Tristate, T1: Tristate, T2: Tristate, T3: Tristate> Tristate for (T0, T1, T2, T3) {
    fn bits() -> usize {
        T0::bits() + T1::bits() + T2::bits() + T3::bits()
    }
}

impl<T0: Tristate, T1: Tristate, T2: Tristate, T3: Tristate, T4: Tristate> Tristate
    for (T0, T1, T2, T3, T4)
{
    fn bits() -> usize {
        T0::bits() + T1::bits() + T2::bits() + T3::bits() + T4::bits()
    }
}
