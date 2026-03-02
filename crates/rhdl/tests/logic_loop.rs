// This component contains an intentional logic loop.
use rhdl::prelude::*;

mod simplest {
    use rhdl::prelude::*;

    #[derive(Clone, Debug, Synchronous, Default)]
    pub struct U;

    impl SynchronousIO for U {
        type I = bool;
        type O = bool;
        type Kernel = simplest;
    }

    impl SynchronousDQ for U {
        type D = ();
        type Q = ();
    }

    #[kernel]
    pub fn simplest(_cr: ClockReset, i: bool, _q: ()) -> (bool, ()) {
        (i, ())
    }
}

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

#[derive(PartialEq, Default, Clone, Copy, Digital)]
pub struct D {
    left: bool,
    right: bool,
}

#[derive(PartialEq, Default, Clone, Copy, Digital)]
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
mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use miette::IntoDiagnostic;
    use rhdl_core::circuit::schematic::error::SchematicICE;

    #[test]
    fn test_logic_loop() -> miette::Result<()> {
        let uut = U::default();
        let Err(RHDLError::SchematicConstructionError(ice)) = uut.descriptor("uut".into()) else {
            panic!("Expected this to fail with a logic loop error");
        };
        let SchematicICE::LogicLoop { filename: _ } = ice.as_ref() else {
            panic!("Expected this to be a logic loop error");
        };
        Ok(())
    }

    #[test]
    fn test_simplest_flow() -> miette::Result<()> {
        let pass_through =
            compile_design_stage1::<simplest::simplest>(CompilationMode::Synchronous)?;
        std::fs::write("simplest.rhif", format!("{:#?}", pass_through)).into_diagnostic()?;
        let uut = simplest::U;
        let _descriptor = uut.descriptor("uut".into())?;
        Ok(())
    }
}
