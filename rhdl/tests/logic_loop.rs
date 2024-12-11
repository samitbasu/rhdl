// This component contains an intentional logic loop.
use rhdl::prelude::*;

mod inverter {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    pub struct U;

    impl SynchronousIO for U {
        type I = bool;
        type O = bool;
        type Kernel = inverter;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn inverter(_cr: ClockReset, i: bool, _q: ()) -> (bool, ()) {
        (!i, ())
    }
}

#[derive(Clone, Debug, Synchronous, Default)]
pub struct U {
    left: inverter::U,
    right: inverter::U,
}

#[derive(Digital, Default)]
pub struct D {
    left: bool,
    right: bool,
}

#[derive(Digital, Default)]
pub struct Q {
    left: bool,
    right: bool,
}

impl SynchronousIO for U {
    type I = bool;
    type O = bool;
    type Kernel = logic_loop;
}

impl SynchronousDQ for U {
    type D = D;
    type Q = Q;
}

#[kernel]
pub fn logic_loop(_cr: ClockReset, i: bool, q: Q) -> (bool, D) {
    let mut d = D::default();
    if i {
        d.left = q.right;
        d.right = q.left;
    }
    (q.left, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_loop() -> miette::Result<()> {
        let uut = U::default();
        assert!(uut.flow_graph("uut").is_err());
        Ok(())
    }
}
