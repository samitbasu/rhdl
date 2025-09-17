//! Constant driver
//!
//! This core provides a constant value that can be provided
//! later in the compilation phase (i.e., not when `rustc` runs).
//!
//! The schematic symbol is simple:
#![doc = badascii_doc::badascii_formal!("
++Constant+-+    
|           | T  
|       val +--->
|           |    
+-----------+    
")]
//!
//! There is no timing information, the constant
//! core simply provides the constant value all the
//! time.
//!
//!# Example
//!
//! Here is an example of the constant being
//! used.
//!
//!```
#![doc = include_str!("../../examples/constant.rs")]
//!```
//!
//! The simulation trace is pretty boring.  
#![doc = include_str!("../../doc/constant.md")]
use quote::format_ident;
use rhdl::prelude::*;
use syn::parse_quote;

#[derive(Clone, Debug)]
/// The core to include for the constant driver
pub struct Constant<T: Digital> {
    value: T,
}

impl<T: Digital> Constant<T> {
    ///. Create a new constant driver with the provided value
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> SynchronousIO for Constant<T> {
    type I = ();
    type O = T;
    type Kernel = NoKernel3<ClockReset, (), (), (T, ())>;
}

impl<T: Digital> SynchronousDQ for Constant<T> {
    type D = ();
    type Q = ();
}

impl<T: Digital> Synchronous for Constant<T> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, _clock_reset: ClockReset, _input: Self::I, _state: &mut Self::S) -> Self::O {
        trace_push_path("constant");
        trace("value", &self.value);
        trace_pop_path();
        self.value
    }

    fn description(&self) -> String {
        format!("Constant: {:?}", self.value.typed_bits())
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        self.as_verilog(name)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: format!("{name}_const_{:?}", self.value.typed_bits()),
            input_kind: Kind::Empty,
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: constant(&self.value, name)?,
        })
    }
}

impl<T: Digital> DigitalFn for Constant<T> {}

impl<T: Digital> Constant<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor(name)?.unique_name;
        let module_ident = format_ident!("{}", module_name);
        let lit: vlog::LitVerilog = self.value.typed_bits().into();
        let bits: vlog::BitRange = (0..T::bits()).into();
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident(input wire [1:0] clock_reset, output wire [#bits] o);
                assign o = #lit;
            endmodule
        };
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}
