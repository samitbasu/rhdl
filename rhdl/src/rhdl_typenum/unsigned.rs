use seq_macro::seq;

pub trait Unsigned: Copy + Default + 'static {
    const USIZE: usize = 0;
    fn new() -> Self {
        Self::default()
    }
}
