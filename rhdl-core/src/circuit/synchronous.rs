use crate::{
    crusty::timing::CostGraph, error::RHDLError, types::reset::Reset, util::hash_id,
    CircuitDescriptor, Clock, Digital, DigitalFn, HDLDescriptor, HDLKind, Tristate,
};

pub type SynchronousUpdateFn<C> = fn(
    bool,
    <C as SynchronousIO>::I,
    <C as SynchronousDQ>::Q,
) -> (<C as SynchronousIO>::O, <C as SynchronousDQ>::D);

pub trait SynchronousIO: 'static + Sized + Clone {
    type I: Digital;
    type O: Digital;
}

pub trait SynchronousDQ: 'static + Sized + Clone {
    type D: Digital;
    type Q: Digital;
}

pub trait Synchronous: 'static + Sized + Clone + SynchronousIO + SynchronousDQ {
    type Z: Tristate;

    type Update: DigitalFn;

    const UPDATE: SynchronousUpdateFn<Self> = |_, _, _| unimplemented!();

    type S: Digital;

    fn sim(
        &self,
        clock: Clock,
        reset: Reset,
        input: Self::I,
        state: &mut Self::S,
        io: &mut Self::Z,
    ) -> Self::O;

    fn name(&self) -> &'static str;

    fn descriptor(&self) -> CircuitDescriptor {
        synchronous_root_descriptor(self)
    }

    fn as_hdl(&self, kind: HDLKind) -> Result<HDLDescriptor, RHDLError>;

    // auto derived
    // First is 0, then 0 + c0::NumZ, then 0 + c0::NumZ + c1::NumZ, etc
    fn z_offsets() -> impl Iterator<Item = usize> {
        std::iter::once(0)
    }
    // Auto derived
    //    fn timing(out_path: &crate::types::path::Path) -> CostGraph;
}

pub fn synchronous_root_descriptor<C: Synchronous>(circuit: &C) -> CircuitDescriptor {
    CircuitDescriptor {
        unique_name: format!(
            "{}_{:x}",
            circuit.name(),
            hash_id(std::any::TypeId::of::<C>())
        ),
        input_kind: C::I::static_kind(),
        output_kind: C::O::static_kind(),
        d_kind: C::D::static_kind(),
        q_kind: C::Q::static_kind(),
        num_tristate: C::Z::N,
        update_schematic: None,
        tristate_offset_in_parent: 0,
        children: Default::default(),
    }
}
