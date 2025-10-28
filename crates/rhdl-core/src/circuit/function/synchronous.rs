use quote::format_ident;
use syn::parse_quote;

use crate::{
    ClockReset, CompilationMode, Digital, DigitalFn, HDLDescriptor, Kind, RHDLError, Synchronous,
    SynchronousDQ, SynchronousIO,
    circuit::{circuit_descriptor::CircuitType, descriptor::Descriptor},
    compile_design,
    digital_fn::{DigitalFn2, NoKernel3},
    ntl::from_rtl::build_ntl_from_rtl,
    rtl::Object,
    trace, trace_pop_path, trace_push_path,
};

use rhdl_vlog::{self as vlog, maybe_port_wire};

#[derive(Clone)]
pub struct Func<I: Digital, O: Digital> {
    kernel: Object,
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
    pub fn try_new<T>() -> Result<Self, RHDLError>
    where
        T: DigitalFn,
        T: DigitalFn2<A0 = ClockReset, A1 = I, O = O>,
    {
        let kernel = compile_design::<T>(CompilationMode::Synchronous)?;
        let update = T::func();
        Ok(Self { kernel, update })
    }
}

impl<I: Digital, O: Digital> Synchronous for Func<I, O> {
    type S = ();

    fn init(&self) -> Self::S {}

    fn sim(&self, clock_reset: ClockReset, input: Self::I, _state: &mut Self::S) -> Self::O {
        trace_push_path("func");
        trace("input", &input);
        let output = (self.update)(clock_reset, input);
        trace("output", &output);
        trace_pop_path();
        output
    }

    fn descriptor(&self, name: &str) -> Result<Descriptor, RHDLError> {
        let module_name = name;
        let module_ident = format_ident!("{}", module_name);
        let ports = [
            maybe_port_wire(vlog::Direction::Input, 2, "clock_reset"),
            maybe_port_wire(vlog::Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, Self::O::bits(), "o"),
        ];
        let function_def = self.kernel.as_vlog()?;
        let func_name = format_ident!("{}", function_def.name);
        // Call the verilog function with (clock_reset, i, q), if they exist.
        let clock_reset = Some(format_ident!("clock_reset"));
        let i_bind = (Self::I::bits() != 0).then(|| format_ident!("i"));
        let fn_args = [clock_reset, i_bind];
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident(#(#ports),*);
                assign o = #func_name(#(#fn_args),*);
                #function_def
            endmodule
        };
        Ok(Descriptor {
            name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            kernel: Some(self.kernel.clone()),
            netlist: Some(build_ntl_from_rtl(&self.kernel)),
            circuit_type: CircuitType::Synchronous,
            hdl: Some(HDLDescriptor {
                name: name.to_string(),
                modules: module.into(),
            }),
        })
    }

    fn children(&self) -> impl Iterator<Item = Result<Descriptor, RHDLError>> {
        std::iter::empty()
    }
}
