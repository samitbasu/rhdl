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

use quote::format_ident;
use rhdl::{
    core::{AsyncKind, ScopedName},
    prelude::*,
};
use syn::parse_quote;

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
    type Kernel = NoCircuitKernel<Self::I, (), (Self::O, ())>;
}

impl<C: Domain> Circuit for ResetNegation<C> {
    type S = ();

    fn init(&self) -> Self::S {}

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

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError> {
        let name = scoped_name.to_string();
        Descriptor::<AsyncKind> {
            name: scoped_name,
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            kernel: None,
            netlist: None,
            hdl: Some(self.hdl(&name)?),
            _phantom: std::marker::PhantomData,
        }
        .with_netlist_black_box()
    }
}

impl<C: Domain> ResetNegation<C> {
    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = format_ident!("{}", name);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_name(input wire [0:0] i, output wire [0:0] o);
                assign o = ~i;
            endmodule
        };
        Ok(HDLDescriptor {
            name: name.into(),
            modules: module.into(),
        })
    }
}
