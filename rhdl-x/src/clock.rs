use rhdl_macro::Digital;

// New type for the clock
#[derive(Debug, Clone, PartialEq, Digital, Default, Copy)]
pub struct Clock(pub bool);

pub fn clock() -> impl Iterator<Item = Clock> {
    std::iter::once(Clock(true))
        .chain(std::iter::once(Clock(false)))
        .cycle()
}
