//! Generate HDL (Verilog) for [Circuit](crate::Circuit)
//!
//! This module provides functionality to generate HDL (specifically Verilog)
//! representations of RHDL circuits.  You will generally not need to call these
//! functions directly.  Instead use the method `descriptor` on [Circuit](crate::Circuit).
//! The returned [Descriptor](crate::circuit::descriptor::Descriptor) will contain
//! the HDL description of the circuit.
//!
//! See the [book] for more details on HDL generation in RHDL.
use std::sync::Arc;

use crate::{
    Circuit, CircuitDQ, CircuitIO, CompilationMode, HDLDescriptor, RHDLError,
    circuit::{
        descriptor::{AsyncKind, Descriptor},
        schematic::circuit::build_schematic,
        scoped_name::ScopedName,
    },
    compiler::driver::{compile_design_stage1, compile_design_stage2},
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

/// Build run time description of a circui, where the circuit is
/// named by `scoped_name`.
pub fn build_asynchronous_descriptor<C: Circuit>(
    circuit: &C,
    scoped_name: ScopedName,
) -> Result<Descriptor<AsyncKind>, RHDLError> {
    let rhif = compile_design_stage1::<C::Kernel>(CompilationMode::Asynchronous)?;
    let kernel = compile_design_stage2(Arc::clone(&rhif))?;
    let children = circuit
        .children(&scoped_name)
        .collect::<Result<Vec<Descriptor<AsyncKind>>, RHDLError>>()?;
    let hdl = build_circuit_hdl::<C>(&scoped_name, &kernel, &children)?;
    let schematic = build_schematic::<C>(&scoped_name, rhif, &children)?;
    let circuit_output = <C as CircuitIO>::O::static_kind();
    let circuit_input = <C as CircuitIO>::I::static_kind();
    let d_kind = <C as CircuitDQ>::D::static_kind();
    let q_kind = <C as CircuitDQ>::Q::static_kind();
    Ok(Descriptor {
        name: scoped_name,
        type_name: std::any::type_name::<C>(),
        input_kind: circuit_input,
        output_kind: circuit_output,
        d_kind,
        q_kind,
        kernel: Some(kernel),
        hdl: Some(hdl),
        schematic: Some(schematic),
        _phantom: std::marker::PhantomData,
    })
}
