use crate::{
    ClockReset, Digital, HDLDescriptor, Kind, RHDLError, Synchronous, SynchronousDQ, SynchronousIO,
    circuit::{circuit_descriptor::CircuitType, descriptor::Descriptor, scoped_name::ScopedName},
    digital_fn::NoKernel3,
};

use quote::format_ident;
use rhdl_vlog as vlog;
use syn::parse_quote;

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

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor, RHDLError> {
        let name = scoped_name.to_string();
        let module_ident = format_ident!("{name}");
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident;
            endmodule
        };
        // let ntl = ntl::Builder::new(name);
        // It's not exactly clear what the build mode of the NetList Builder
        // should be.  We do not know if the phantom is in a synchronous or
        // asynchronous context, and ideally it shouldn't matter.  So we use
        // the synchronous mode so that the inputs have at least place holders
        // for the clock reset and input vecs (both empty)
        Ok(Descriptor {
            name: scoped_name,
            input_kind: Kind::Empty,
            output_kind: Kind::Empty,
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            circuit_type: CircuitType::Asynchronous,
            kernel: None,
            netlist: None,
            hdl: Some(HDLDescriptor {
                name: name.to_string(),
                modules: module.into(),
            }),
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
