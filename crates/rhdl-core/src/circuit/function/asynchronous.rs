use crate::{
    Circuit, CircuitDQ, CircuitDescriptor, CircuitIO, CompilationMode, Digital, DigitalFn,
    HDLDescriptor, Kind, RHDLError, Timed,
    circuit::circuit_descriptor::CircuitType,
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
    module: Object,
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
        let module = compile_design::<T>(CompilationMode::Asynchronous)?;
        let update = T::func();
        Ok(Self { module, update })
    }
}

impl<I: Timed, O: Timed> Circuit for AsyncFunc<I, O> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, input: Self::I, _state: &mut Self::S) -> Self::O {
        (self.update)(input)
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            ntl: build_ntl_from_rtl(&self.module),
            rtl: Some(self.module.clone()),
            children: Default::default(),
            circuit_type: CircuitType::Asynchronous,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        let module_name = &descriptor.unique_name;
        let ports = [
            maybe_port_wire(vlog::Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, Self::O::bits(), "o"),
        ];
        let i_bind = (Self::I::bits() != 0).then(|| format_ident!("i"));
        let module_name_ident = format_ident!("{}", module_name);
        let function_def = descriptor
            .rtl
            .as_ref()
            .ok_or(RHDLError::FunctionNotSynthesizable {
                name: descriptor.unique_name.clone(),
            })?
            .as_vlog()?;
        let function_name = format_ident!("{}", function_def.name);
        let module: vlog::ModuleDef = parse_quote! {
            module #module_name_ident(#(#ports),*);
                assign o = #function_name(#i_bind);
                #function_def
            endmodule
        };
        Ok(HDLDescriptor {
            name: module_name.into(),
            modules: module.into(),
        })
    }
}
