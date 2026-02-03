use rhdl::{
    core::{AsyncKind, ScopedName},
    prelude::*,
};

// ANCHOR: circuit_trait
pub trait Circuit: 'static + CircuitIO + Sized {
    type S: Clone + PartialEq;

    fn init(&self) -> Self::S;
    fn sim(&self, input: Self::I, state: &mut Self::S) -> Self::O;
    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError>;
    fn children(
        &self,
        parent_scope: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<AsyncKind>, RHDLError>>;
}

// ANCHOR_END: circuit_trait
