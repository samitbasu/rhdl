use quote::{format_ident, quote};
use rhdl_vlog::declaration;
use syn::parse_quote;

use crate::circuit::circuit_descriptor::CircuitType;
use crate::circuit::descriptor::Descriptor;
use crate::{
    ClockReset, Digital, HDLDescriptor, Kind, Synchronous, SynchronousDQ, SynchronousIO,
    digital_fn::NoKernel3, trace_pop_path, trace_push_path,
};
use crate::{RHDLError, ntl};
use rhdl_vlog as vlog;
use rhdl_vlog::{maybe_port_wire, unsigned_width};

#[derive(Clone)]
pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Synchronous, B: Synchronous> SynchronousIO for Chain<A, B> {
    type I = <A as SynchronousIO>::I;
    type O = <B as SynchronousIO>::O;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

impl<A: Synchronous, B: Synchronous> SynchronousDQ for Chain<A, B> {
    type D = ();
    type Q = ();
}

impl<A: Synchronous, B: Synchronous, P: Digital> Synchronous for Chain<A, B>
where
    A: SynchronousIO<O = P>,
    B: SynchronousIO<I = P>,
{
    type S = (A::S, B::S);

    fn init(&self) -> Self::S {
        (self.a.init(), self.b.init())
    }

    fn sim(&self, clock_reset: crate::ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("chain");
        trace_push_path("a");
        let p = self.a.sim(clock_reset, input, &mut state.0);
        trace_pop_path();
        trace_push_path("b");
        let o = self.b.sim(clock_reset, p, &mut state.1);
        trace_pop_path();
        trace_pop_path();
        o
    }

    fn descriptor(&self, name: &str) -> Result<Descriptor, RHDLError> {
        let a_descriptor = self.a.descriptor(&format!("{name}_a"))?;
        let b_descriptor = self.b.descriptor(&format!("{name}_b"))?;
        Ok(Descriptor {
            name: name.into(),
            input_kind: a_descriptor.input_kind,
            output_kind: b_descriptor.output_kind,
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            circuit_type: CircuitType::Synchronous,
            kernel: None,
            netlist: Some(self.netlist(name, &a_descriptor, &b_descriptor)?),
            hdl: Some(self.hdl(name, &a_descriptor, &b_descriptor)?),
        })
    }

    fn children(&self) -> impl Iterator<Item = Result<Descriptor, RHDLError>> {
        std::iter::once(self.a.descriptor("a")).chain(std::iter::once(self.b.descriptor("b")))
    }
}

impl<A: Synchronous, B: Synchronous, P: Digital> Chain<A, B>
where
    A: SynchronousIO<O = P>,
    B: SynchronousIO<I = P>,
{
    fn hdl(
        &self,
        name: &str,
        a_descriptor: &Descriptor,
        b_descriptor: &Descriptor,
    ) -> Result<HDLDescriptor, RHDLError> {
        let ports = [
            maybe_port_wire(vlog::Direction::Input, <A as SynchronousIO>::I::bits(), "i"),
            maybe_port_wire(
                vlog::Direction::Output,
                <B as SynchronousIO>::O::bits(),
                "o",
            ),
        ];
        let input_kind = <A as SynchronousIO>::I::static_kind();
        let pipe_kind = <A as SynchronousIO>::O::static_kind();
        let pipe = declaration(
            vlog::HDLKind::Wire,
            Some(unsigned_width(pipe_kind.bits())),
            "pipe",
        );
        let a_ident = format_ident!("{}_a", a_descriptor.name);
        let b_ident = format_ident!("{}_b", b_descriptor.name);
        let a_input_binding = if input_kind.is_empty() {
            quote! {}
        } else {
            quote! {.i(i)}
        };
        let module_ident = format_ident!("{name}");
        let a_hdl = a_descriptor.hdl()?;
        let b_hdl = b_descriptor.hdl()?;
        let a_modules = &a_hdl.modules;
        let b_modules = &b_hdl.modules;
        let module_list: vlog::ModuleList = parse_quote! {
            module #module_ident(input wire [1:0] clock_reset, #(#ports),*);
                #pipe
                #a_ident a(.clock_reset(clock_reset), .o(pipe), #a_input_binding);
                #b_ident b(.clock_reset(clock_reset), .i(pipe), .o(o));
            endmodule
            #a_modules
            #b_modules
        };
        Ok(HDLDescriptor {
            name: name.into(),
            modules: module_list,
        })
    }

    fn netlist(
        &self,
        name: &str,
        a_descriptor: &Descriptor,
        b_descriptor: &Descriptor,
    ) -> Result<ntl::Object, RHDLError> {
        let mut builder = ntl::Builder::new(name);
        let input_kind: Kind = <A as SynchronousIO>::I::static_kind();
        let output_kind: Kind = <B as SynchronousIO>::O::static_kind();
        // The inputs to the circuit are [cr, I], the output is [O]
        // Allocate these as inputs to the netlist
        let top_cr = builder.add_input(ClockReset::static_kind());
        let top_i = builder.add_input(input_kind);
        let top_o = builder.allocate_outputs(output_kind);
        // Link in the A and B children
        let a_netlist = a_descriptor.netlist()?;
        let b_netlist = b_descriptor.netlist()?;
        let a_offset = builder.import(a_netlist);
        let b_offset = builder.import(b_netlist);
        // Connect the clock and reset to the A and B netlists.
        for ((tcr, acr), bcr) in top_cr
            .iter()
            .zip(&a_netlist.inputs[0])
            .zip(&b_netlist.inputs[0])
        {
            builder.copy_from_to(*tcr, a_offset(acr.into()));
            builder.copy_from_to(*tcr, b_offset(bcr.into()));
        }
        // Connect the input of the NTL to the input of the first circuit
        for (ti, ai) in top_i.iter().zip(&a_netlist.inputs[1]) {
            builder.copy_from_to(*ti, a_offset(ai.into()));
        }
        // Connect the circuit A to the input of circuit B
        for (ao, bi) in a_netlist.outputs.iter().zip(&b_netlist.inputs[1]) {
            builder.copy_from_to(a_offset(*ao), b_offset(bi.into()));
        }
        // Connec the output of circuit B to the NTL output
        for (to, bo) in top_o.iter().zip(&b_netlist.outputs) {
            builder.copy_from_to(b_offset(*bo), *to)
        }
        builder.build(ntl::builder::BuilderMode::Synchronous)
    }
}
