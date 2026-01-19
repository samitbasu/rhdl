#![allow(dead_code)]
use rhdl::{
    core::{ScopedName, SyncKind},
    prelude::*,
};

struct Foo;

impl Foo {
    // ANCHOR: descriptor
    pub fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError>
// ANCHOR_END: descriptor
    {
        let _ = scoped_name;
        todo!()
    }
}

pub mod children {
    use rhdl::{
        core::{ScopedName, SyncKind},
        prelude::*,
    };

    // ANCHOR: children
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        // snip...
        /// Iterate over the child circuits of this circuit.
        fn children(
            &self,
            _parent_scope: &ScopedName,
        ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
            std::iter::empty()
        }
    }
    // ANCHOR_END: children
}

pub mod sim {
    use rhdl::{
        core::{ScopedName, SyncKind},
        prelude::*,
    };

    // ANCHOR: sim-signature
    pub trait Synchronous: 'static + Sized + SynchronousIO {
        // State storage type
        type S: PartialEq + Clone;
        // snip...
        //                 👇 - extra argument
        fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O;
        // snip...
    }
    // ANCHOR_END: sim-signature
}
