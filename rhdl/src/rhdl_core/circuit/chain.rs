use crate::rhdl_core::ntl;
use crate::rhdl_core::{
    digital_fn::NoKernel3,
    hdl::ast::{
        component_instance, connection, id, unsigned_width, Declaration, Direction, HDLKind, Module,
    },
    rtl::object::RegisterKind,
    trace_pop_path, trace_push_path, CircuitDescriptor, ClockReset, Digital, HDLDescriptor, Kind,
    Synchronous, SynchronousDQ, SynchronousIO,
};
use std::collections::BTreeMap;

use super::hdl_backend::maybe_port_wire;

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

    fn sim(
        &self,
        clock_reset: crate::rhdl_core::ClockReset,
        input: Self::I,
        state: &mut Self::S,
    ) -> Self::O {
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

    fn descriptor(
        &self,
        name: &str,
    ) -> Result<crate::rhdl_core::CircuitDescriptor, crate::rhdl_core::RHDLError> {
        let a_name = format!("{name}_a");
        let b_name = format!("{name}_b");
        let desc_a = self.a.descriptor(&a_name)?;
        let desc_b = self.b.descriptor(&b_name)?;
        let mut builder = ntl::Builder::new(name);
        let input_kind: RegisterKind = <A as SynchronousIO>::I::static_kind().into();
        let output_kind: RegisterKind = <B as SynchronousIO>::O::static_kind().into();
        // The inputs to the circuit are [cr, I], the output is [O]
        // Allocate these as inputs to the netlist
        let top_cr = builder.add_input(2);
        let top_i = builder.add_input(input_kind.len());
        let top_o = builder.allocate_outputs(output_kind.len());
        // Link in the A and B children
        let a_offset = builder.link(&desc_a.ntl);
        let b_offset = builder.link(&desc_b.ntl);
        // Connect the clock and reset to the A and B netlists.
        for ((tcr, acr), bcr) in top_cr
            .iter()
            .zip(&desc_a.ntl.inputs[0])
            .zip(&desc_b.ntl.inputs[0])
        {
            builder.copy_from_to(*tcr, acr.offset(a_offset));
            builder.copy_from_to(*tcr, bcr.offset(b_offset));
        }
        // Connect the input of the NTL to the input of the first circuit
        for (ti, ai) in top_i.iter().zip(&desc_a.ntl.inputs[1]) {
            builder.copy_from_to(*ti, ai.offset(a_offset));
        }
        // Connect the circuit A to the input of circuit B
        for (ao, bi) in desc_a.ntl.outputs.iter().zip(&desc_b.ntl.inputs[1]) {
            builder.copy_from_to(ao.offset(a_offset), bi.offset(b_offset));
        }
        // Connec the output of circuit B to the NTL output
        for (to, bo) in top_o.iter().zip(&desc_b.ntl.outputs) {
            builder.copy_from_to(bo.offset(b_offset), *to)
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

    fn hdl(
        &self,
        name: &str,
    ) -> Result<crate::rhdl_core::HDLDescriptor, crate::rhdl_core::RHDLError> {
        let mut module = Module {
            name: name.into(),
            description: self.description(),
            ..Default::default()
        };
        let input_kind = <A as SynchronousIO>::I::static_kind();
        let pipe_kind = <A as SynchronousIO>::O::static_kind();
        module.ports = [
            maybe_port_wire(Direction::Input, 2, "clock_reset"),
            maybe_port_wire(Direction::Input, <A as SynchronousIO>::I::bits(), "i"),
            maybe_port_wire(Direction::Output, <B as SynchronousIO>::O::bits(), "o"),
        ]
        .into_iter()
        .flatten()
        .collect();
        module.declarations.push(Declaration {
            kind: HDLKind::Wire,
            name: "pipe".into(),
            width: unsigned_width(pipe_kind.bits()),
            alias: None,
        });
        let a_name = &format!("{name}_a");
        let b_name = &format!("{name}_b");
        // Add the two child components.
        let a_input_binding = if input_kind.is_empty() {
            None
        } else {
            Some(connection("i", id("i")))
        };
        let cr_binding = Some(connection("clock_reset", id("clock_reset")));
        let b_output_binding = Some(connection("o", id("o")));
        let a_p_binding = Some(connection("o", id("pipe")));
        let b_i_binding = Some(connection("i", id("pipe")));
        let a_instance = component_instance(
            a_name,
            "a",
            [cr_binding.clone(), a_input_binding.clone(), a_p_binding]
                .into_iter()
                .flatten()
                .collect(),
        );
        let b_instance = component_instance(
            b_name,
            "b",
            [cr_binding, b_i_binding, b_output_binding]
                .into_iter()
                .flatten()
                .collect(),
        );
        let a_hdl = self.a.hdl(a_name)?;
        let b_hdl = self.b.hdl(b_name)?;
        module.statements.extend([a_instance, b_instance]);
        Ok(HDLDescriptor {
            name: name.into(),
            body: module,
            children: BTreeMap::from_iter(vec![(a_name.into(), a_hdl), (b_name.into(), b_hdl)]),
        })
    }
}
