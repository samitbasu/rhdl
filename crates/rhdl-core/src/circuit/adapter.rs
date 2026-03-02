//! An adapter to use [Synchronous](crate::Synchronous) circuits in [Circuit](crate::Circuit) contexts.
//!
//! An adapter allows you to wrap a synchronous circuit so that it can be used
//! in an asynchronous context.  The adapter maps the (implicit) clock and reset
//! signasl of the synchronous circuit to explicit inputs of the adapter circuit.
//! This allows you to integrate synchronous circuits into larger asynchronous
//! designs.
//!
//! # Motivation
//!
//! Note that in the canonical diagram of a synchronous circuit, the clock and
//! reset signals are injected into the design and into each of the children.  
#![doc = badascii_doc::badascii!(r"
        +---------------------------------------------------------------+        
        |   +--+ SynchronousIO::I           SynchronousIO::O +--+       |        
  input |   v               +-----------------------+           v       | output 
 +----->+------------------>|input            output+-------------------+------->
        | +---------------->|c&r      Kernel        |                   |        
        | |            +--->|q                     d+-----+             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
        | |            |    +-----------------------+     |             |        
        | |q.child_1+> +----+o        child_1      i|<----+ <+d.child_1 |        
        | +-----------+|+-->|c&r                    |     |             |        
        | |            |    +-----------------------+     |             |        
        | |            |                                  |             |        
  clock | |            |    +-----------------------+     |             |        
& reset | |q.child_2+> +----+o        child_2      i|<----+ <+d.child_2 |        
 +------+-+---------------->|c&r                    |                   |        
  (c&r) |                   +-----------------------+                   |        
        +---------------------------------------------------------------+        
")]
//! In particular, note that the input type [SynchronousIO::I](crate::SynchronousIO::I) of the synchronous
//! circuit does not explicitly include the clock and reset lines.  This makes it easier to compose synchronous
//! circuits together, since you don't need to explicitly feed the clock and reset signals into each
//! child.  
//!
//! Asynchronous [Circuit](crate::Circuit) circuits, on the other hand, have no implicit signals injected,
//! and all input and output signals must appear in [CircuitIO::I](crate::CircuitIO::I) and [CircuitIO::O](crate::CircuitIO::O).
//! Therefore, to use a synchronous circuit in an asynchronous context, we need to somehow make the
//! clock and reset signals explicit.  This is what the [Adapter](Adapter) struct does.
//!
#![doc = badascii_doc::badascii!(r"
                     +--+Adapter+-------------------------------+               
        Adapter::I   |                                          |               
          .input     |   SynchronousIO::I       SynchronousIO::O|               
    +--------------+ |         + (T)                     + (U)  |               
      signal<T,D>  | +         v       +-------------+   v      +  Adapter::O   
                   +------------------>| Synchronous +------------------------->
                     + +-------------->|   Circuit   |          +  signal<U,D>  
        Adapter::I   + |               +-------------+          |               
          .clock +-----+                                        |               
          .reset     +                           Domain D       |               
signal<ClockReset,D> +------------------------------------------+               
")]
//!
//! Note that the adapter also assigns a domain D to the clock and reset signals, as well
//! as to the input and output signals of the circuit.  By using the adapter, you are promising
//! that the input signals are synchronous with the provided clock and reset signals!  And that
//! the clock and reset are also synchronous.  This is important for correct operation of the
//! synchronous circuit.  If that requirement is violated, then you may end up with undefined behavior
//! and/or data corruption!!
//!  
use crate::{
    Circuit, CircuitDQ, CircuitIO, ClockReset, Digital, DigitalFn, Domain, HDLDescriptor, Kind,
    RHDLError, Signal, Synchronous, Timed,
    bitx::BitX,
    circuit::{
        descriptor::{AsyncKind, Descriptor, SyncKind},
        schematic::{
            CanonicalPath, Schematic,
            builder::{QueryPortSet, SchematicBuilder},
        },
        scoped_name::ScopedName,
    },
    digital_fn::NoCircuitKernel,
    types::{kind::Field, signal::signal},
};

use quote::{format_ident, quote};
use rhdl_vlog::{self as vlog, maybe_port_wire};
use syn::parse_quote;

/// An adapter allows you to use a Synchronous circuit in an Asynchronous context.
///
#[derive(Clone)]
pub struct Adapter<C: Synchronous, D: Domain> {
    circuit: C,
    domain: std::marker::PhantomData<D>,
}

impl<C: Synchronous, D: Domain> Adapter<C, D> {
    /// Create a new adapter wrapping the given synchronous circuit.
    #[must_use]
    pub fn new(circuit: C) -> Self {
        Self {
            circuit,
            domain: Default::default(),
        }
    }
}

impl<C: Synchronous + Default, D: Domain> Default for Adapter<C, D> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

/// The input type for the Adapter circuit.
///
/// This type combines the clock and reset signals with the input
/// signals for the underlying synchronous circuit.
///
/// Each is also promoted to be a [Signal](crate::Signal) in the given domain D.
///
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AdapterInput<I: Digital, D: Domain> {
    /// The clock and reset signals.
    pub clock_reset: Signal<ClockReset, D>,
    /// The input signals for the underlying synchronous circuit.
    pub input: Signal<I, D>,
}

impl<I: Digital, D: Domain> Timed for AdapterInput<I, D> {}

impl<I: Digital, D: Domain> Digital for AdapterInput<I, D> {
    const BITS: usize = <Signal<ClockReset, D> as Digital>::BITS + <Signal<I, D> as Digital>::BITS;
    fn static_kind() -> crate::Kind {
        Kind::make_struct(
            "AdapterInput",
            vec![
                Field {
                    name: "clock_reset".to_string().into(),
                    kind: <Signal<ClockReset, D> as Digital>::static_kind(),
                },
                Field {
                    name: "input".to_string().into(),
                    kind: <Signal<I, D> as Digital>::static_kind(),
                },
            ]
            .into(),
        )
    }
    fn bin(self) -> Box<[BitX]> {
        let mut out = vec![];
        out.extend(self.clock_reset.bin());
        out.extend(self.input.bin());
        out.into()
    }
    fn dont_care() -> Self {
        Self {
            clock_reset: Signal::dont_care(),
            input: Signal::dont_care(),
        }
    }
}

impl<C: Synchronous, D: Domain> CircuitIO for Adapter<C, D> {
    type I = AdapterInput<C::I, D>;
    type O = Signal<C::O, D>;
    type Kernel = NoCircuitKernel<Self::I, (), (Self::O, ())>;
}

impl<C: Synchronous, D: Domain> CircuitDQ for Adapter<C, D> {
    type D = ();
    type Q = ();
}

impl<C: Synchronous, D: Domain> Circuit for Adapter<C, D> {
    type S = C::S;

    fn init(&self) -> Self::S {
        self.circuit.init()
    }

    fn sim(&self, input: AdapterInput<C::I, D>, state: &mut C::S) -> Signal<C::O, D> {
        let clock_reset = input.clock_reset.val();
        let input = input.input.val();
        let result = self.circuit.sim(clock_reset, input, state);
        signal(result)
    }

    fn descriptor(&self, scoped_name: ScopedName) -> Result<Descriptor<AsyncKind>, RHDLError> {
        let child_descriptor = self.circuit.descriptor(scoped_name.with("inner"))?;
        let name = scoped_name.to_string();
        Ok(Descriptor::<AsyncKind> {
            name: scoped_name,
            type_name: std::any::type_name::<Self>(),
            input_kind: <<Self as CircuitIO>::I as Digital>::static_kind(),
            output_kind: <<Self as CircuitIO>::O as Digital>::static_kind(),
            d_kind: <<Self as CircuitDQ>::D as Digital>::static_kind(),
            q_kind: <<Self as CircuitDQ>::Q as Digital>::static_kind(),
            kernel: None,
            hdl: Some(self.hdl(&name, &child_descriptor)?),
            schematic: Some(self.schematic(&name, &child_descriptor)?),
            _phantom: std::marker::PhantomData,
        })
    }

    fn children(
        &self,
        scoped_name: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<AsyncKind>, RHDLError>> {
        let inner = self
            .circuit
            .descriptor(scoped_name.with("inner"))
            .map(|inner| Descriptor::<AsyncKind> {
                name: scoped_name.with("inner"),
                type_name: inner.type_name,
                input_kind: inner.input_kind,
                output_kind: inner.output_kind,
                d_kind: inner.d_kind,
                q_kind: inner.q_kind,
                kernel: inner.kernel,
                hdl: inner.hdl,
                schematic: inner.schematic,
                _phantom: std::marker::PhantomData,
            });
        std::iter::once(inner)
    }
}

impl<C: Synchronous, D: Domain> DigitalFn for Adapter<C, D> {}

impl<C: Synchronous, D: Domain> Adapter<C, D> {
    fn hdl(
        &self,
        name: &str,
        child_descriptor: &Descriptor<SyncKind>,
    ) -> Result<HDLDescriptor, RHDLError> {
        let ports = [
            maybe_port_wire(vlog::Direction::Input, <Self as CircuitIO>::I::bits(), "i"),
            maybe_port_wire(vlog::Direction::Output, <Self as CircuitIO>::O::bits(), "o"),
        ];
        let child_hdl = child_descriptor.hdl()?;
        let name_ident = format_ident!("{name}");
        let input_connection = if !child_descriptor.input_kind.is_empty() {
            let lsb = syn::Index::from(2);
            let msb = syn::Index::from((2 + child_descriptor.input_kind.bits()).saturating_sub(1));
            quote! {, .i(i[#msb:#lsb])}
        } else {
            quote! {}
        };
        let child_unique_name_ident = format_ident!("{}", child_descriptor.name.to_string());
        let child_modules = &child_hdl.modules;
        let module_list: vlog::ModuleList = parse_quote! {
            module #name_ident(#(#ports),*);
                #child_unique_name_ident c(.clock_reset(i[1:0]) #input_connection, .o(o))
            endmodule
            #child_modules
        };
        Ok(HDLDescriptor {
            name: name.to_string(),
            modules: module_list,
        })
    }
    fn schematic(
        &self,
        name: &str,
        child_descriptor: &Descriptor<SyncKind>,
    ) -> Result<Schematic, RHDLError> {
        let cr_kind = ClockReset::static_kind();
        let mut builder = SchematicBuilder::circuit::<Self>(name)?;
        let child_index = builder.import(child_descriptor.schematic()?.clone());
        builder.select_and_link(
            QueryPortSet::Input { index: 0 },
            QueryPortSet::ChildInput {
                child_index,
                index: 0,
            },
            cr_kind,
            &CanonicalPath::default().field("clock_reset").signal_value(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::Input { index: 0 },
            QueryPortSet::ChildInput {
                child_index,
                index: 1,
            },
            child_descriptor.input_kind,
            &CanonicalPath::default().field("input").signal_value(),
            &CanonicalPath::default(),
        )?;
        builder.select_and_link(
            QueryPortSet::ChildOutput { child_index },
            QueryPortSet::Output,
            child_descriptor.output_kind,
            &CanonicalPath::default(),
            &CanonicalPath::default().signal_value(),
        )?;
        Ok(builder.build())
    }
}
