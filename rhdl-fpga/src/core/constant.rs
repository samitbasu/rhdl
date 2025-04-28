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
use rhdl::{
    core::{
        hdl::ast::{
            bit_string, continuous_assignment, port, signed_width, unsigned_width, Direction,
            Module,
        },
        types::bit_string::BitString,
    },
    prelude::*,
};

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
        let mut flow_graph = FlowGraph::default();
        let my_val = &self.value.typed_bits().bits;
        let driver = my_val.iter().map(|b| {
            flow_graph.new_component_with_optional_location(ComponentKind::Constant(*b), 1, None)
        });
        flow_graph.output = driver.collect();
        flow_graph.inputs = vec![vec![], vec![]];
        Ok(CircuitDescriptor {
            unique_name: format!("{name}_const_{:?}", self.value.typed_bits()),
            input_kind: Kind::Empty,
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            flow_graph,
            children: Default::default(),
            rtl: None,
        })
    }
}

impl<T: Digital> DigitalFn for Constant<T> {}

impl<T: Digital> Constant<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor(name)?.unique_name;
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        let output_bits = T::bits();
        let output_width = if T::static_kind().is_signed() {
            signed_width(output_bits)
        } else {
            unsigned_width(output_bits)
        };
        let bs: BitString = self.value.typed_bits().into();
        module.ports = vec![
            port(
                "clock_reset",
                Direction::Input,
                rhdl::core::hdl::ast::HDLKind::Wire,
                unsigned_width(2),
            ),
            port(
                "o",
                Direction::Output,
                rhdl::core::hdl::ast::HDLKind::Wire,
                output_width,
            ),
        ];
        module
            .statements
            .push(continuous_assignment("o", bit_string(&bs)));
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}
