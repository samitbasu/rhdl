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
}

impl SynchronousKernel for U {
    type Kernel = inverter;
}

#[kernel]
pub fn inverter(_cr: ClockReset, i: bool, _q: ()) -> (bool, ()) {
    (!i, ())
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
