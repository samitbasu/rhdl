pub trait Len {
    type Output: Unsigned;
    fn len(&self) -> Self::Output;
}

pub trait Digit: Copy + Default + 'static {
    const USIZE: usize = 0;
}

pub trait Unsigned: Copy + Default + 'static {
    const USIZE: usize = 0;
}

pub trait Trim {
    type Output: Unsigned;
    fn trim(&self) -> Self::Output;
}
