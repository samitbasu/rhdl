use crate::{
    CircuitDescriptor, ClockReset, Digital, HDLDescriptor, Kind, RHDLError, Synchronous,
    SynchronousDQ, SynchronousIO, digital_fn::NoKernel3, hdl::ast::Module, ntl,
};

impl<T: Digital + 'static> Synchronous for std::marker::PhantomData<T> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(
        &self,
        _clock_reset: crate::ClockReset,
        _input: Self::I,
        _state: &mut Self::S,
    ) -> Self::O {
    }

    fn description(&self) -> String {
        format!("Phantom (type only) component: {:?}", T::static_kind())
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let ntl = ntl::Builder::new(name);
        // It's not exactly clear what the build mode of the NetList Builder
        // should be.  We do not know if the phantom is in a synchronous or
        // asynchronous context, and ideally it shouldn't matter.  So we use
        // the synchronous mode so that the inputs have at least place holders
        // for the clock reset and input vecs (both empty)
        Ok(CircuitDescriptor {
            unique_name: format!("{name}_phantom"),
            input_kind: Kind::Empty,
            output_kind: Kind::Empty,
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
            ntl: ntl.build(ntl::builder::BuilderMode::Synchronous)?,
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
