use quote::{format_ident, quote};
use rhdl_vlog::declaration;
use syn::parse_quote;

use crate::ntl;
use crate::{
    CircuitDescriptor, ClockReset, Digital, HDLDescriptor, Kind, Synchronous, SynchronousDQ,
    SynchronousIO, digital_fn::NoKernel3, trace_pop_path, trace_push_path,
};
use rhdl_vlog as vlog;
use rhdl_vlog::{maybe_port_wire, unsigned_width};
use std::collections::BTreeMap;

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

    fn description(&self) -> String {
        format!(
            "series synchronous circuit of {} and {}",
            self.a.description(),
            self.b.description()
        )
    }

    fn descriptor(&self, name: &str) -> Result<crate::CircuitDescriptor, crate::RHDLError> {
        let a_name = format!("{name}_a");
        let b_name = format!("{name}_b");
        let desc_a = self.a.descriptor(&a_name)?;
        let desc_b = self.b.descriptor(&b_name)?;
        let mut builder = ntl::Builder::new(name);
        let input_kind: Kind = <A as SynchronousIO>::I::static_kind();
        let output_kind: Kind = <B as SynchronousIO>::O::static_kind();
        // The inputs to the circuit are [cr, I], the output is [O]
        // Allocate these as inputs to the netlist
        let top_cr = builder.add_input(ClockReset::static_kind());
        let top_i = builder.add_input(input_kind);
        let top_o = builder.allocate_outputs(output_kind);
        // Link in the A and B children
        let a_offset = builder.import(&desc_a.ntl);
        let b_offset = builder.import(&desc_b.ntl);
        // Connect the clock and reset to the A and B netlists.
        for ((tcr, acr), bcr) in top_cr
            .iter()
            .zip(&desc_a.ntl.inputs[0])
            .zip(&desc_b.ntl.inputs[0])
        {
            builder.copy_from_to(*tcr, a_offset(acr.into()));
            builder.copy_from_to(*tcr, b_offset(bcr.into()));
        }
        // Connect the input of the NTL to the input of the first circuit
        for (ti, ai) in top_i.iter().zip(&desc_a.ntl.inputs[1]) {
            builder.copy_from_to(*ti, a_offset(ai.into()));
        }
        // Connect the circuit A to the input of circuit B
        for (ao, bi) in desc_a.ntl.outputs.iter().zip(&desc_b.ntl.inputs[1]) {
            builder.copy_from_to(a_offset(*ao), b_offset(bi.into()));
        }
        // Connec the output of circuit B to the NTL output
        for (to, bo) in top_o.iter().zip(&desc_b.ntl.outputs) {
            builder.copy_from_to(b_offset(*bo), *to)
        }
        let desc = CircuitDescriptor {
            unique_name: name.into(),
            input_kind: desc_a.input_kind,
            output_kind: desc_b.output_kind,
            q_kind: Kind::Empty,
            d_kind: Kind::Empty,
            ntl: builder.build(ntl::builder::BuilderMode::Synchronous)?,
            rtl: None,
            children: BTreeMap::from_iter(vec![(a_name, desc_a), (b_name, desc_b)]),
        };
        Ok(desc)
    }

    fn hdl(&self, name: &str) -> Result<crate::HDLDescriptor, crate::RHDLError> {
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
        let a_name = format!("{}_a", name);
        let b_name = format!("{}_b", name);
        let a_ident = format_ident!("{}_a", name);
        let b_ident = format_ident!("{}_b", name);
        let a_input_binding = if input_kind.is_empty() {
            quote! {}
        } else {
            quote! {.i(i)}
        };
        let module_ident = format_ident!("{name}");
        let module: vlog::ModuleDef = parse_quote! {
            module #module_ident(input wire [1:0] clock_reset, #(#ports),*);
                #pipe
                #a_ident a(.clock_reset(clock_reset), .o(pipe), #a_input_binding);
                #b_ident b(.clock_reset(clock_reset), .i(pipe), .o(o));
            endmodule
        };
        let a_hdl = self.a.hdl(&a_name)?;
        let b_hdl = self.b.hdl(&b_name)?;
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: BTreeMap::from_iter(vec![(a_name, a_hdl), (b_name, b_hdl)]),
        })
    }
}
