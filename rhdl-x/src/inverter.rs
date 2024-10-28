use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, Default)]
pub struct U {}

impl SynchronousDQ for U {
    type D = ();
    type Q = ();
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
    type Kernel = inverter;
}

#[kernel]
pub fn inverter(_cr: ClockReset, i: bool, _q: ()) -> (bool, ()) {
    (!i, ())
}

#[kernel]
pub fn func_inverter(_cr: ClockReset, i: bool) -> bool {
    !i
}

pub type Uinv = Func<bool, bool>;

pub fn new() -> Result<Uinv, RHDLError> {
    Func::new::<func_inverter>()
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_inverter_flow_graph() -> miette::Result<()> {
        let uut = U::default();
        let fg = &uut.descriptor("uut")?.flow_graph;
        let mut file = std::fs::File::create("inverter.dot").unwrap();
        write_dot(fg, &mut file).unwrap();
        Ok(())
    }
}
