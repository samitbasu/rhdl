use std::collections::BTreeMap;

use crate::{
    CircuitDescriptor, Digital, FlowGraph, Kind, Synchronous, SynchronousDQ, SynchronousIO,
    Tristate,
};

impl<A: Synchronous, B: Synchronous> SynchronousIO for (A, B) {
    type I = <A as SynchronousIO>::I;
    type O = <B as SynchronousIO>::O;
}

impl<A: Synchronous, B: Synchronous> SynchronousDQ for (A, B) {
    type D = ();

    type Q = ();
}

impl<A: Synchronous, B: Synchronous, ZC: Tristate, P: Digital> Synchronous for (A, B)
where
    A: Synchronous<Z = ZC>,
    A: SynchronousIO<O = P>,
    B: Synchronous<Z = ZC>,
    B: SynchronousIO<I = P>,
{
    type Z = ZC;

    type Update = ();

    type S = (A::S, B::S);

    fn sim(
        &self,
        clock_reset: crate::ClockReset,
        input: Self::I,
        state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O {
        let p = self.0.sim(clock_reset, input, &mut state.0, io);
        self.1.sim(clock_reset, p, &mut state.1, io)
    }

    fn description(&self) -> String {
        format!(
            "series synchronous circuit of {} and {}",
            self.0.description(),
            self.1.description()
        )
    }

    fn descriptor(&self, name: &str) -> Result<crate::CircuitDescriptor, crate::RHDLError> {
        let a_name = format!("{name}_0");
        let b_name = format!("{name}_1");
        let desc_a = self.0.descriptor(&a_name)?;
        let desc_b = self.1.descriptor(&b_name)?;
        let children = BTreeMap::from_iter(vec![(a_name, desc_a), (b_name, desc_b)]);
        let fg_0 = desc_a.flow_graph;
        let fg_1 = desc_b.flow_graph;
        let mut fg = desc_a.flow_graph;
        let b_remap = fg.merge(&fg_1);

        let desc = CircuitDescriptor {
            unique_name: name.into(),
            input_kind: desc_a.input_kind,
            output_kind: desc_b.output_kind,
            q_kind: Kind::Empty,
            d_kind: Kind::Empty,
            num_tristate: ZC::N,
            tristate_offset_in_parent: 0,
            flow_graph,
            rtl: None,
            children,
        };
        Ok(desc)
    }

    fn hdl(&self, name: &str) -> Result<crate::HDLDescriptor, crate::RHDLError> {
        todo!()
    }
}
