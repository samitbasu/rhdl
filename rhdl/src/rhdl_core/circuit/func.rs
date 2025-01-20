use crate::rhdl_core::{
    build_rtl_flow_graph, compile_design,
    digital_fn::{DigitalFn2, NoKernel3},
    hdl::{
        ast::{continuous_assignment, function_call, id, Direction, Module},
        builder::generate_verilog,
    },
    rtl::Object,
    CircuitDescriptor, ClockReset, CompilationMode, Digital, DigitalFn, HDLDescriptor, Kind,
    RHDLError, Synchronous, SynchronousDQ, SynchronousIO,
};

use super::hdl_backend::maybe_port_wire;

#[derive(Clone)]
pub struct Func<I: Digital, O: Digital> {
    module: Object,
    update: fn(ClockReset, I) -> O,
}

impl<I: Digital, O: Digital> SynchronousIO for Func<I, O> {
    type I = I;
    type O = O;
    type Kernel = NoKernel3<ClockReset, I, (), (O, ())>;
}

impl<I: Digital, O: Digital> SynchronousDQ for Func<I, O> {
    type D = ();
    type Q = ();
}

impl<I: Digital, O: Digital> Func<I, O> {
    pub fn new<T>() -> Result<Self, RHDLError>
    where
        T: DigitalFn,
        T: DigitalFn2<A0 = ClockReset, A1 = I, O = O>,
    {
        let module = compile_design::<T>(CompilationMode::Synchronous)?;
        let update = T::func();
        Ok(Self { module, update })
    }
}

impl<I: Digital, O: Digital> Synchronous for Func<I, O> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, clock_reset: ClockReset, input: Self::I, _state: &mut Self::S) -> Self::O {
        (self.update)(clock_reset, input)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            flow_graph: build_rtl_flow_graph(&self.module),
            rtl: Some(self.module.clone()),
            children: Default::default(),
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        let module_name = &descriptor.unique_name;
        let mut module = Module {
            name: module_name.clone(),
            description: self.description(),
            ..Default::default()
        };
        module.ports = [
            maybe_port_wire(Direction::Input, 2, "clock_reset"),
            maybe_port_wire(Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(Direction::Output, Self::O::bits(), "o"),
        ]
        .into_iter()
        .flatten()
        .collect();
        let verilog = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
        // Call the verilog function with (clock_reset, i, q), if they exist.
        let clock_reset = Some(id("clock_reset"));
        let i_bind = (Self::I::bits() != 0).then(|| id("i"));
        let fn_call = function_call(
            &verilog.name,
            vec![clock_reset, i_bind].into_iter().flatten().collect(),
        );
        let fn_call = continuous_assignment("o", fn_call);
        module.statements.push(fn_call);
        module.functions.push(verilog);
        Ok(HDLDescriptor {
            name: module_name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
