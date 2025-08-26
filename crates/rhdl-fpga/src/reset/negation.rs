//!# Reset Negation Core
//!
//! Because the [Reset] type is meant to signal an
//! active-high reset, if you need a reset that is
//! active-low, you will need a [ResetNegation] core.
//! This core is simply a not gate, but it can unwrap
//! the [ResetN] type and create a new [Reset] type (which signifies)
//! an active-high reset.  This is sometimes needed for
//! interfacing to external systems that provide a active-low reset.
//!
//!# Schematic
//!
//! Here is the schematic symbol for the [ResetNegation]
//! core.
//!
#![doc = badascii_doc::badascii_formal!(r"
          Reset            
       ++Negation+-+       
       |    +      |       
       |    |\     |       
resetN++--->| +○+--+>reset
       |    |/     +       
       |    +      |       
       |           |       
       +-----------+       
"
)]
//!
//!# Example
//!
//! Here is a simple example of using the [ResetNegation] core.
//!
//!```
#![doc = include_str!("../../examples/reset_negation.rs")]
//!```
//!
//!With the trace
#![doc = include_str!("../../doc/reset_negation.md")]
//!
use rhdl::{
    core::{hdl::ast::unary, rtl::spec::AluUnary},
    prelude::*,
};

#[derive(PartialEq, Debug, Clone, Default)]
/// The [ResetNegation] core.  Both the input and
/// output signals must belong to the same clock
/// domain `C`.
pub struct ResetNegation<C: Domain> {
    _c: std::marker::PhantomData<C>,
}

impl<C: Domain> CircuitDQ for ResetNegation<C> {
    type D = ();
    type Q = ();
}

impl<C: Domain> CircuitIO for ResetNegation<C> {
    type I = Signal<ResetN, C>;
    type O = Signal<Reset, C>;
    type Kernel = NoKernel2<Self::I, (), (Self::O, ())>;
}

impl<C: Domain> Circuit for ResetNegation<C> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn description(&self) -> String {
        format!(
            "Reset inversion (active low to active high) in domain {:?}",
            C::color()
        )
    }

    fn sim(&self, input: Self::I, _state: &mut Self::S) -> Self::O {
        trace_push_path("reset_negation");
        let out = if input.val().raw() {
            reset(false)
        } else {
            reset(true)
        };
        trace("input", &input);
        trace("output", &out);
        trace_pop_path();
        signal(out)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.into(),
            input_kind: <Self::I as Timed>::static_kind(),
            output_kind: <Self::O as Timed>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: rhdl::core::ntl::builder::circuit_black_box(self, name)?,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        module.ports = vec![
            port("i", Direction::Input, HDLKind::Wire, unsigned_width(1)),
            port("o", Direction::Output, HDLKind::Wire, unsigned_width(1)),
        ];
        module
            .statements
            .push(continuous_assignment("o", unary(AluUnary::Not, id("i"))));
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
