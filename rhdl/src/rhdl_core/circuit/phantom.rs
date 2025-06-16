use crate::prelude::{
    CircuitDescriptor, ClockReset, Digital, HDLDescriptor, Kind, Module, NoKernel3, RHDLError,
    Synchronous, SynchronousDQ, SynchronousIO,
};

impl<T: Digital + 'static> Synchronous for std::marker::PhantomData<T> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(
        &self,
        _clock_reset: crate::prelude::ClockReset,
        _input: Self::I,
        _state: &mut Self::S,
    ) -> Self::O {
    }

    fn description(&self) -> String {
        format!("Phantom (type only) component: {:?}", T::static_kind())
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: format!("{name}_phantom"),
            input_kind: Kind::Empty,
            output_kind: Kind::Empty,
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: crate::core::ntl::object::Object::default(),
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = self.descriptor(name)?.unique_name;
        let module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}

impl<T: Digital + 'static> SynchronousIO for std::marker::PhantomData<T> {
    type I = ();
    type O = ();
    type Kernel = NoKernel3<ClockReset, (), (), ((), ())>;
}

impl<T: Digital + 'static> SynchronousDQ for std::marker::PhantomData<T> {
    type D = ();
    type Q = ();
}
