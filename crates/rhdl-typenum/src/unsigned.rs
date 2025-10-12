/// Trait for type-level unsigned integers.
pub trait Unsigned: Copy + Default + 'static {
    /// The value of the type-level integer as a `usize`.
    const USIZE: usize = 0;
    /// Create a new instance of the type-level integer.
    fn new() -> Self {
        Self::default()
    }
}
