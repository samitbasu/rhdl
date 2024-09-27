use rhdl::prelude::*;

#[derive(Clone, Debug, Synchronous, Default)]
#[rhdl(kernel=inverter)]
pub struct U {}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
}

impl SynchronousDQ for U {
    type D = ();
    type Q = ();
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
        let fg = &uut.descriptor()?.flow_graph;
        let mut file = std::fs::File::create("inverter.dot").unwrap();
        write_dot(fg, &mut file).unwrap();
        Ok(())
    }
}
