//! Generate HDL (Verilog) for [Circuit](crate::Circuit)
//!
//! This module provides functionality to generate HDL (specifically Verilog)
//! representations of RHDL circuits.  You will generally not need to call these
//! functions directly.  Instead use the method `descriptor` on [Circuit](crate::Circuit).
//! The returned [Descriptor](crate::circuit::descriptor::Descriptor) will contain
//! the HDL description of the circuit.
//!
//! See the [book] for more details on HDL generation in RHDL.
use crate::{
    Circuit, CircuitDQ, CircuitIO, CompilationMode, HDLDescriptor, Kind, RHDLError,
    circuit::{
        descriptor::{AsyncKind, Descriptor},
        scoped_name::ScopedName,
    },
    compile_design,
    ntl::{self, from_rtl::build_ntl_from_rtl},
    rtl,
    types::{
        digital::Digital,
        path::{Path, bit_range},
    },
};
use quote::{format_ident, quote};
use rhdl_vlog as vlog;

fn build_circuit_hdl<C: Circuit>(
    scoped_name: &ScopedName,
    kernel: &rtl::Object,
    children: &[Descriptor<AsyncKind>],
) -> Result<HDLDescriptor, RHDLError> {
    let local_name = scoped_name.to_string();
    let circuit_output = <C as CircuitIO>::O::static_kind();
    let circuit_input = <C as CircuitIO>::I::static_kind();
    let d_kind = <C as CircuitDQ>::D::static_kind();
    let q_kind = <C as CircuitDQ>::Q::static_kind();
    let outputs = circuit_output.bits();
    let ports = [
        vlog::maybe_port_wire(vlog::Direction::Input, circuit_input.bits(), "i"),
        vlog::maybe_port_wire(vlog::Direction::Output, circuit_output.bits(), "o"),
    ];
    let declarations = [
        vlog::maybe_decl_wire(circuit_output.bits() + d_kind.bits(), "od"),
        vlog::maybe_decl_wire(d_kind.bits(), "d"),
        vlog::maybe_decl_wire(q_kind.bits(), "q"),
    ];
    let mut child_decls = Vec::new();
    let mut child_hdls = Vec::new();
    for (ndx, child_desc) in children.iter().enumerate() {
        if child_desc.output_kind.is_empty() {
            continue;
        }
        child_hdls.push(child_desc.hdl()?.modules.clone());
        let local_name = child_desc.name.last().unwrap();
        let child_path = Path::default().field(local_name);
        let (d_range, _) = bit_range(d_kind, &child_path)?;
        let (q_range, _) = bit_range(q_kind, &child_path)?;
        let input_binding = vlog::maybe_connect("i", "d", d_range);
        let output_binding = vlog::maybe_connect("o", "q", q_range);
        let bindings = [input_binding, output_binding];
        let bindings = bindings.iter().flatten();
        let component_name = format_ident!("{}", child_desc.name.to_string());
        let component_instance = format_ident!("c{ndx}");
        child_decls.push(quote! {
            #component_name #component_instance(
                #(#bindings),*
            );
        });
    }
    let kernel = kernel.as_vlog()?;
    // Call the verilog function with (i, q), if they exist.
    let i_bind = (circuit_input.bits() != 0).then(|| format_ident!("i"));
    let q_bind = (q_kind.bits() != 0).then(|| format_ident!("q"));
    let kernel_name = format_ident!("{}", kernel.name);
    let module_ident = format_ident!("{local_name}");
    let output_range: vlog::BitRange = (0..outputs).into();
    let d_bind = (d_kind.bits() != 0).then(|| {
        let d_range: vlog::BitRange = (outputs..(d_kind.bits() + outputs)).into();
        quote! {assign d = od[#d_range];}
    });
    let modules: vlog::ModuleList = vlog::parse_quote_miette! {
        module #module_ident(#(#ports),*);
            #(#declarations;)*
            assign o = od[#output_range];
            #(#child_decls;)*
            #d_bind
            assign od = #kernel_name(#i_bind, #q_bind);
            #kernel
        endmodule
        #(#child_hdls)*
    }?;
    Ok(HDLDescriptor {
        name: local_name,
        modules,
    })
}

fn build_circuit_netlist<C: Circuit>(
    scoped_name: &ScopedName,
    kernel: &rtl::Object,
    children: &[Descriptor<AsyncKind>],
) -> Result<ntl::Object, RHDLError> {
    let name = scoped_name.to_string();
    // Build the netlist
    // First construct the netlist for the update function
    let update_netlist = build_ntl_from_rtl(kernel);
    // Create a manual builder for the top level netlist
    let mut builder = ntl::builder::Builder::new(&name);
    let output_kind: Kind = C::O::static_kind();
    if output_kind.is_empty() {
        return Err(RHDLError::NoOutputsError);
    }
    let input_kind: Kind = C::I::static_kind();
    let top_i = builder.add_input(input_kind);
    let top_o = builder.allocate_outputs(output_kind);
    let update_register_offset = builder.import(&update_netlist);
    // Link the module input to the input of the update function
    for (&top_i_bit, &update_i_bit) in top_i.iter().zip(&update_netlist.inputs[0]) {
        builder.copy_from_to(top_i_bit, update_register_offset(update_i_bit.into()));
    }
    // Link up the output bits from the update_netlist
    for (&top_o_bit, &update_o_bit) in top_o.iter().zip(&update_netlist.outputs) {
        builder.copy_from_to(update_register_offset(update_o_bit), top_o_bit);
    }
    // Get the "D" vector by skipping the first |O| bits, and pre-map them into their new addresses
    let d_vec = update_netlist
        .outputs
        .iter()
        .skip(output_kind.bits())
        .map(|op| update_register_offset(*op))
        .collect::<Vec<_>>();
    // Get the "Q" vector by remapping the 2nd input to the update function.
    // Note that the update function signature for a synchronous function is (ClockReset, I, Q) -> (O, D)
    let q_vec = update_netlist.inputs[1]
        .iter()
        .map(|op| update_register_offset(op.into()))
        .collect::<Vec<_>>();
    // Create the inputs for the children by splitting bits off of the d_index
    for child_descriptor in children {
        let child_name = child_descriptor.name.last().unwrap();
        // Compute the bit range for this child's input based on its name
        let child_path = Path::default().field(child_name);
        let (output_bit_range, _) = bit_range(C::D::static_kind(), &child_path)?;
        let (input_bit_range, _) = bit_range(C::Q::static_kind(), &child_path)?;
        let netlist =
            child_descriptor
                .netlist
                .as_ref()
                .ok_or(RHDLError::FunctionNotSynthesizable {
                    name: child_descriptor.name.to_string(),
                })?;
        // Merge the child's netlist into ours
        let child_offset = builder.import(netlist);
        // Connect the child's input registers to the given bits of the D register
        for (&d_bit, child_i) in d_vec[output_bit_range.clone()]
            .iter()
            .zip(&netlist.inputs[0])
        {
            builder.copy_from_to(d_bit, child_offset(child_i.into()));
        }
        // Connect the childs output registers to the given bits of the Q register
        for (&q_bit, &child_o) in q_vec[input_bit_range.clone()].iter().zip(&netlist.outputs) {
            builder.copy_from_to(child_offset(child_o), q_bit);
        }
    }
    builder.build(ntl::builder::BuilderMode::Asynchronous)
}

/// Build run time description of a circui, where the circuit is
/// named by `scoped_name`.
pub fn build_asynchronous_descriptor<C: Circuit>(
    circuit: &C,
    scoped_name: ScopedName,
) -> Result<Descriptor<AsyncKind>, RHDLError> {
    let kernel = compile_design::<C::Kernel>(CompilationMode::Asynchronous)?;
    let children = circuit
        .children(&scoped_name)
        .collect::<Result<Vec<Descriptor<AsyncKind>>, RHDLError>>()?;
    let hdl = build_circuit_hdl::<C>(&scoped_name, &kernel, &children)?;
    let netlist = build_circuit_netlist::<C>(&scoped_name, &kernel, &children)?;
    let circuit_output = <C as CircuitIO>::O::static_kind();
    let circuit_input = <C as CircuitIO>::I::static_kind();
    let d_kind = <C as CircuitDQ>::D::static_kind();
    let q_kind = <C as CircuitDQ>::Q::static_kind();
    Ok(Descriptor {
        name: scoped_name,
        input_kind: circuit_input,
        output_kind: circuit_output,
        d_kind,
        q_kind,
        kernel: Some(kernel),
        hdl: Some(hdl),
        netlist: Some(netlist),
        _phantom: std::marker::PhantomData,
    })
}
