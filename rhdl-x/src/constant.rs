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
pub struct U<T: Digital> {
    value: T,
}

impl<T: Digital> U<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Digital> SynchronousIO for U<T> {
    type I = ();
    type O = T;
    type Kernel = dummy<T>;
}

impl<T: Digital> SynchronousDQ for U<T> {
    type D = ();
    type Q = ();
}

#[kernel]
pub fn dummy<T: Digital>(_cr: ClockReset, _i: (), _q: ()) -> (T, ()) {
    (T::init(), ())
}

impl<T: Digital> Synchronous for U<T> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, _clock_reset: ClockReset, _input: Self::I, _state: &mut Self::S) -> Self::O {
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

impl<T: Digital> DigitalFn for U<T> {}

impl<T: Digital> U<T> {
    fn as_verilog(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor(name)?.unique_name;
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        let output_bits = T::bits().saturating_sub(1);
        let output_width = if T::static_kind().is_signed() {
            signed_width(output_bits)
        } else {
            unsigned_width(output_bits)
        };
        let bs: BitString = self.value.typed_bits().into();
        module.ports = vec![
            port(
                "clock",
                Direction::Input,
                rhdl::core::hdl::ast::HDLKind::Wire,
                unsigned_width(1),
            ),
            port(
                "reset",
                Direction::Input,
                rhdl::core::hdl::ast::HDLKind::Wire,
                unsigned_width(1),
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
