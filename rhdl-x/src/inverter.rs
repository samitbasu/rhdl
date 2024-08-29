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
pub fn inverter(_reset: bool, i: bool, _q: ()) -> (bool, ()) {
    (!i, ())
}

#[test]
fn test_inverter_flow_graph() -> miette::Result<()> {
    let uut = U::default();
    let descriptor = uut.descriptor()?;
    let fg = build_synchronous_flow_graph(&descriptor)?;
    let mut file = std::fs::File::create("inverter.dot").unwrap();
    write_dot(&fg, &mut file).unwrap();
    Ok(())
}
