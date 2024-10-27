use rhdl::core::circuit::synchronous::SynchronousUpdateFn;
use rhdl::prelude::*;

#[kernel]
pub fn demo(_cr: ClockReset, i: b8) -> b8 {
    i << 1
}

#[derive(Clone)]
struct Func<I: Digital, O: Digital> {
    module: Object,
    update: fn(ClockReset, I) -> O,
}

impl<I: Digital, O: Digital> SynchronousIO for Func<I, O> {
    type I = I;
    type O = O;
}

impl<I: Digital, O: Digital> SynchronousDQ for Func<I, O> {
    type D = ();
    type Q = ();
}

impl<I: Digital, O: Digital> Func<I, O> {
    pub fn new<T: DigitalFn>(update: fn(ClockReset, I) -> O) -> Result<Self, RHDLError> {
        let module = compile_design::<T>(CompilationMode::Synchronous)?;
        Ok(Self { module, update })
    }
}

impl<I: Digital, O: Digital> Synchronous for Func<I, O> {
    type Z = ();
    type Update = ();
    const UPDATE: SynchronousUpdateFn<Self> = |_, _, _| unimplemented!();

    type S = ();

    fn sim(
        &self,
        clock_reset: ClockReset,
        input: Self::I,
        _state: &mut Self::S,
        _io: &mut Self::Z,
    ) -> Self::O {
        (self.update)(clock_reset, input)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            num_tristate: 0,
            tristate_offset_in_parent: 0,
            flow_graph: build_rtl_flow_graph(&self.module),
            rtl: Some(self.module.clone()),
            children: Default::default(),
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        let outputs = Self::O::bits();

        let module_name = &descriptor.unique_name;
        let mut module = Module {
            name: module_name.clone(),
            description: self.description(),
            ..Default::default()
        };
        module.ports = [
            
        ]

    }
}

#[test]
fn test_fn() {}
