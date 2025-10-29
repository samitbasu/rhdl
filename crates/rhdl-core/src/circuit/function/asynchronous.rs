use crate::{
    Circuit, CircuitDQ, CircuitIO, CompilationMode, Digital, DigitalFn, HDLDescriptor, Kind,
    RHDLError, Timed,
    circuit::{circuit_descriptor::CircuitType, descriptor::Descriptor, scoped_name::ScopedName},
    compile_design,
    digital_fn::{DigitalFn1, NoKernel2},
    ntl::from_rtl::build_ntl_from_rtl,
    rtl::Object,
};

use quote::format_ident;
use rhdl_vlog::{self as vlog, maybe_port_wire};
use syn::parse_quote;

#[derive(Clone)]
pub struct AsyncFunc<I: Timed, O: Timed> {
    kernel: Object,
    update: fn(I) -> O,
}

impl<I: Timed, O: Timed> CircuitIO for AsyncFunc<I, O> {
    type I = I;
    type O = O;
    type Kernel = NoKernel2<I, (), (O, ())>;
}

impl<I: Timed, O: Timed> CircuitDQ for AsyncFunc<I, O> {
    type D = ();
    type Q = ();
}

impl<I: Timed, O: Timed> AsyncFunc<I, O> {
    pub fn new<T>() -> Result<Self, RHDLError>
    where
        T: DigitalFn,
        T: DigitalFn1<A0 = I, O = O>,
    {
        let kernel = compile_design::<T>(CompilationMode::Asynchronous)?;
        let update = T::func();
        Ok(Self { kernel, update })
    }
}

impl<I: Timed, O: Timed> Circuit for AsyncFunc<I, O> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, input: Self::I, _state: &mut Self::S) -> Self::O {
        (self.update)(input)
    }

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor, RHDLError> {
        let module_name = scoped_name.to_string();
        let ports = [
            maybe_port_wire(vlog::Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, Self::O::bits(), "o"),
        ];
        let i_bind = (Self::I::bits() != 0).then(|| format_ident!("i"));
        let module_name_ident = format_ident!("{}", module_name);
        let function_def = self.kernel.as_vlog()?;
        let function_name = format_ident!("{}", function_def.name);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_name_ident(#(#ports),*);
                assign o = #function_name(#i_bind);
                #function_def
            endmodule
        };
        Ok(Descriptor {
            name: scoped_name,
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            netlist: Some(build_ntl_from_rtl(&self.kernel)),
            kernel: Some(self.kernel.clone()),
            circuit_type: CircuitType::Asynchronous,
            hdl: Some(HDLDescriptor {
                name: module_name,
                modules: module.into(),
            }),
        })
    }
}
