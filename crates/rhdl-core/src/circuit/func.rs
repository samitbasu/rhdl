use quote::format_ident;
use syn::parse_quote;

use crate::{
    CircuitDescriptor, ClockReset, CompilationMode, Digital, DigitalFn, HDLDescriptor, Kind,
    RHDLError, Synchronous, SynchronousDQ, SynchronousIO, compile_design,
    digital_fn::{DigitalFn2, NoKernel3},
    hdl::builder::generate_verilog,
    ntl::from_rtl::build_ntl_from_rtl,
    rtl::Object,
    trace, trace_pop_path, trace_push_path,
};

use rhdl_vlog::{self as vlog, maybe_port_wire};

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
    pub fn try_new<T>() -> Result<Self, RHDLError>
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
        trace_push_path("func");
        trace("input", &input);
        let output = (self.update)(clock_reset, input);
        trace("output", &output);
        trace_pop_path();
        output
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            rtl: Some(self.module.clone()),
            children: Default::default(),
            ntl: build_ntl_from_rtl(&self.module),
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let descriptor = self.descriptor(name)?;
        let module_name = &descriptor.unique_name;
        let module_ident = format_ident!("{}", module_name);
        let ports = [
            maybe_port_wire(vlog::Direction::Input, 2, "clock_reset"),
            maybe_port_wire(vlog::Direction::Input, Self::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, Self::O::bits(), "o"),
        ];
        let function_def = generate_verilog(descriptor.rtl.as_ref().unwrap())?;
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
        Ok(HDLDescriptor {
            name: module_name.into(),
            body: module,
            children: Default::default(),
        })
    }
}
