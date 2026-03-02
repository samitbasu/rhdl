//! Synchronous circuit composition
//!
//! This struct allows you to chain two synchronous circuits together.
//! ! Given two circuits A and B, where the output type of A matches the input type of B,
//! you can create a new circuit that represents the composition of A followed by B.
//!
#![doc = badascii_doc::badascii!(r"
  +---+Chain<A, B> +---------+  
  |                          |  
  +  I   +----+  P  +----+   |  
+------->|    +---->|    | O +  
  |  cr  | A  |     | B  +----->
+----+-->|    |  +->|    |   +  
  +  |   +----+  |  +----+   |  
  |  +-----------+           |  
  +--------------------------+  
")]
//!
//! The input type of the composed circuit is the input type of A,
//! and the output type is the output type of B.
use quote::{format_ident, quote};
use rhdl_vlog::declaration;
use syn::parse_quote;

use crate::RHDLError;
use crate::circuit::descriptor::{Descriptor, SyncKind};
use crate::circuit::schematic::builder::{QueryPortSet, SchematicBuilder};
use crate::circuit::schematic::{CanonicalPath, Schematic};
use crate::circuit::scoped_name::ScopedName;
use crate::{
    ClockReset, Digital, HDLDescriptor, Kind, Synchronous, SynchronousDQ, SynchronousIO,
    digital_fn::NoSynchronousKernel, trace_pop_path, trace_push_path,
};
use rhdl_vlog as vlog;
use rhdl_vlog::{maybe_port_wire, unsigned_width};

/// Chain two synchronous circuits together.
/// Given two circuits A and B, where the output type of A matches the input type of B,
/// create a new circuit that represents the composition of A followed by B.
pub struct Chain<A, B> {
    a: A,
    b: B,
}

impl<A, B> Chain<A, B> {
    /// Create a new [Chain] circuit from the given circuits A and B.
    #[must_use]
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: Synchronous, B: Synchronous> SynchronousIO for Chain<A, B> {
    type I = <A as SynchronousIO>::I;
    type O = <B as SynchronousIO>::O;
    type Kernel = NoSynchronousKernel<ClockReset, Self::I, (), (Self::O, ())>;
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

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<SyncKind>, RHDLError> {
        let a_descriptor = self.a.descriptor(scoped_name.with("a"))?;
        let b_descriptor = self.b.descriptor(scoped_name.with("b"))?;
        let name = scoped_name.to_string();
        Ok(Descriptor::<SyncKind> {
            name: scoped_name,
            type_name: std::any::type_name::<Self>(),
            input_kind: a_descriptor.input_kind,
            output_kind: b_descriptor.output_kind,
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            kernel: None,
            schematic: Some(self.schematic(&name, &a_descriptor, &b_descriptor)?),
            hdl: Some(self.hdl(&name, &a_descriptor, &b_descriptor)?),
            _phantom: std::marker::PhantomData,
        })
    }

    fn children(
        &self,
        parent_scope: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
        std::iter::once(self.a.descriptor(parent_scope.with("a")))
            .chain(std::iter::once(self.b.descriptor(parent_scope.with("b"))))
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
        a_descriptor: &Descriptor<SyncKind>,
        b_descriptor: &Descriptor<SyncKind>,
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
        let a_ident = format_ident!("{}", a_descriptor.name.to_string());
        let b_ident = format_ident!("{}", b_descriptor.name.to_string());
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

    fn schematic(
        &self,
        name: &str,
        a_descriptor: &Descriptor<SyncKind>,
        b_descriptor: &Descriptor<SyncKind>,
    ) -> Result<Schematic, RHDLError> {
        let mut builder = SchematicBuilder::synchronous::<Self>(name)?;
        let cr_kind = ClockReset::static_kind();
        let input_kind = a_descriptor.input_kind;
        let output_kind = b_descriptor.output_kind;
        let child_a = builder.import(a_descriptor.schematic()?.clone());
        let child_b = builder.import(b_descriptor.schematic()?.clone());
        builder.select_and_link(
            QueryPortSet::Input { index: 0 },
            QueryPortSet::ChildInput {
                child_index: child_a,
                index: 0,
            },
            cr_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::Input { index: 0 },
            QueryPortSet::ChildInput {
                child_index: child_b,
                index: 0,
            },
            cr_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::Input { index: 1 },
            QueryPortSet::ChildInput {
                child_index: child_a,
                index: 1,
            },
            input_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::ChildOutput {
                child_index: child_a,
            },
            QueryPortSet::ChildInput {
                child_index: child_b,
                index: 1,
            },
            a_descriptor.output_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::ChildOutput {
                child_index: child_b,
            },
            QueryPortSet::Output,
            output_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default(),
        )?;
        Ok(builder.build())
    }
}
