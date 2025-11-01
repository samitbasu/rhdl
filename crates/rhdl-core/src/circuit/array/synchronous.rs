//! Support for arrays of [Synchronous](crate::Synchronous)
//!
//! In RHDL, there is a blanket implementation of [Synchronous](crate::Synchronous) for
//! an array of `impl Synchronous`.  This allows you to create arrays of synchronous circuits
//! and use them as a single synchronous circuit.  The array circuit will have inputs and
//! outputs that are arrays of the inputs and outputs of the individual circuits.  The
//! clock and reset are automatically fed to each individual circuit.
//!
#![doc = badascii_doc::badascii!(r"
            [0]   I   +-------+   O    +          
         +----------->| C 0   +------->|          
         |            +-------+        |          
         |               ^ cr          |          
         |     +---------+             |          
         |  [1]   I   +-------+   O    |          
         +----------->| C 1   +------->|          
         |     +      +-------+        |          
         |     |        ^ cr           |          
 [I;N]   |     +--------+ .            | [O;N]    
+--------+     |          .            +--------->
         |     |          .            |          
         |     +                       |          
         | [N-1]  I   +-------+   O    |          
         +----------->| C N-1 +------->|          
               +      +-------+        +          
               |          ^ cr                    
+--------------+----------+                       
")]
//!
//! Here, there are `N` instances of circuit `C`, each taking an input of type `I`
//! and producing an output of type `O`.  The array circuit takes an input
//! of type `[I; N]` (an array of `N` inputs of type `I`) and produces an output
//! of type `[O; N]` (an array of `N` outputs of type `O`).  The
//! clock and reset signals are provided to each individual circuit automatically.
//!
use crate::{
    ClockReset, Digital, HDLDescriptor, Kind, RHDLError, Synchronous, SynchronousDQ, SynchronousIO,
    circuit::{
        descriptor::{Descriptor, SyncKind},
        scoped_name::ScopedName,
    },
    digital_fn::NoKernel3,
    ntl, trace_pop_path, trace_push_path,
    types::path::{Path, bit_range},
};
use quote::{format_ident, quote};
use rhdl_vlog as vlog;
use syn::parse_quote;

// Blanket implementation for an array of synchronous circuits.
impl<T: SynchronousIO, const N: usize> SynchronousIO for [T; N] {
    type I = [T::I; N];
    type O = [T::O; N];
    type Kernel = NoKernel3<ClockReset, Self::I, Self::Q, (Self::O, Self::D)>;
}

impl<T: SynchronousIO, const N: usize> SynchronousDQ for [T; N] {
    type D = ();
    type Q = ();
}

const ARRAY_ENTRIES: &[&str] = &[
    "[0]", "[1]", "[2]", "[3]", "[4]", "[5]", "[6]", "[7]", "[8]", "[9]", "[10]", "[11]", "[12]",
    "[13]", "[14]", "[15]", "[XX]",
];

impl<T: Synchronous, const N: usize> Synchronous for [T; N] {
    type S = [T::S; N];

    fn init(&self) -> Self::S {
        core::array::from_fn(|i| self[i].init())
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("array");
        let mut output = [T::O::dont_care(); N];
        for i in 0..N {
            trace_push_path(ARRAY_ENTRIES[i.min(16)]);
            output[i] = self[i].sim(clock_reset, input[i], &mut state[i]);
            trace_pop_path();
        }
        trace_pop_path();
        output
    }

    // This requires a custom implementation because the default implementation
    // assumes that the children of the current circuit are named with field names
    // as part of a struct.
    fn descriptor(
        &self,
        scoped_name: ScopedName,
    ) -> Result<Descriptor<SyncKind>, crate::RHDLError> {
        let children = self
            .children(&scoped_name)
            .collect::<Result<Vec<_>, RHDLError>>()?;
        let name = scoped_name.to_string();
        Ok(Descriptor {
            name: scoped_name,
            input_kind: Self::I::static_kind(),
            output_kind: Self::O::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            kernel: None,
            hdl: Some(hdl::<T, N>(&name, &children)?),
            netlist: Some(netlist::<T, N>(&name, &children)?),
            _phantom: std::marker::PhantomData,
        })
    }

    fn children(
        &self,
        parent_scope: &ScopedName,
    ) -> impl Iterator<Item = Result<Descriptor<SyncKind>, RHDLError>> {
        (0..N).map(move |i| self[i].descriptor(parent_scope.with(format!("c{i}"))))
    }
}

fn hdl<T: Synchronous, const N: usize>(
    name: &str,
    children: &[Descriptor<SyncKind>],
) -> Result<HDLDescriptor, RHDLError> {
    let module_name = &name;
    let module_ident = format_ident!("{module_name}");
    let i_kind = <[T; N] as SynchronousIO>::I::static_kind();
    let o_kind = <[T; N] as SynchronousIO>::O::static_kind();
    let ports = [
        vlog::maybe_port_wire(vlog::Direction::Input, 2, "clock_reset"),
        vlog::maybe_port_wire(vlog::Direction::Input, i_kind.bits(), "i"),
        vlog::maybe_port_wire(vlog::Direction::Output, o_kind.bits(), "o"),
    ];
    let ports = ports.iter().flatten();
    let mut child_hdls = vec![];
    let mut child_decls = vec![];
    for (ndx, child) in children.iter().enumerate() {
        let child_path = Path::default().index(ndx);
        let (i_range, _) = bit_range(i_kind, &child_path)?;
        let (o_range, _) = bit_range(o_kind, &child_path)?;
        let i_range_empty = i_range.is_empty();
        let o_range_empty = o_range.is_empty();
        let i_range_vlog: vlog::BitRange = i_range.into();
        let o_range_vlog: vlog::BitRange = o_range.into();
        let input_binding = (!i_range_empty).then(|| quote! {.i(i[#i_range_vlog])});
        let output_binding = (!o_range_empty).then(|| quote! {.o(o[#o_range_vlog])});
        let bindings = [
            Some(parse_quote! {.clock_reset(clock_reset)}),
            input_binding,
            output_binding,
        ];
        let component_name = format_ident!("{}", child.name.to_string());
        let instance_name = format_ident!("c{ndx}");
        let bindings = bindings.iter().flatten();
        child_decls.push(quote! { #component_name #instance_name(#(#bindings),*) });
        child_hdls.push(child.hdl()?.modules.clone());
    }
    let modules: vlog::ModuleList = parse_quote! {
        module #module_ident(#(#ports),*);
            #(#child_decls);*
        endmodule
        #(#child_hdls)*
    };
    Ok(HDLDescriptor {
        name: name.into(),
        modules,
    })
}

fn netlist<T: Synchronous, const N: usize>(
    name: &str,
    children: &[Descriptor<SyncKind>],
) -> Result<ntl::Object, RHDLError> {
    let mut builder = ntl::Builder::new(name);
    let cr_kind: Kind = ClockReset::static_kind();
    let input_kind: Kind = <[T; N] as SynchronousIO>::I::static_kind();
    let output_kind: Kind = <[T; N] as SynchronousIO>::O::static_kind();
    let tcr = builder.add_input(cr_kind);
    let ti = builder.add_input(input_kind);
    let to = builder.allocate_outputs(output_kind);
    for (i, child_descriptor) in children.iter().enumerate() {
        let child_path = Path::default().index(i);
        let (output_bit_range, _) = bit_range(output_kind, &child_path)?;
        let (input_bit_range, _) = bit_range(input_kind, &child_path)?;
        let child_netlist = child_descriptor.netlist()?;
        let offset = builder.import(child_netlist);
        let child_inputs = &child_netlist.inputs;
        for (&t, c) in tcr.iter().zip(&child_inputs[0]) {
            builder.copy_from_to(t, offset(c.into()));
        }
        for (&t, c) in ti[input_bit_range].iter().zip(&child_inputs[1]) {
            builder.copy_from_to(t, offset(c.into()));
        }
        for (&t, c) in to[output_bit_range].iter().zip(&child_netlist.outputs) {
            builder.copy_from_to(offset(*c), t);
        }
    }
    builder.build(ntl::builder::BuilderMode::Synchronous)
}
