use crate::{
    Circuit, CircuitDQ, CircuitDescriptor, CircuitIO, HDLDescriptor, RHDLError,
    circuit::descriptor::Descriptor,
    types::digital::Digital,
    types::path::{Path, bit_range},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rhdl_vlog as vlog;

/// Build run time description of a circuit
pub fn build_asynchronous_descriptor<C: Circuit>(
    circuit: &C,
    name: &str,
) -> Result<Descriptor, RHDLError> {
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
    for (ndx, child_desc) in circuit.children().enumerate() {
        let child_desc = child_desc?;
        if child.output_kind.is_empty() {
            continue;
        }
    }

    let child_decls = circuit
        .children()
        .filter(|desc| !desc.map(|d| d.output_kind.is_empty()).unwrap_or(false))
        .enumerate()
        .map(|(ndx, desc)| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind, &child_path)?;
            let (q_range, _) = bit_range(q_kind, &child_path)?;
            let input_binding = vlog::maybe_connect("i", "d", d_range);
            let output_binding = vlog::maybe_connect("o", "q", q_range);
            let bindings = [input_binding, output_binding];
            let bindings = bindings.iter().flatten();
            let component_name = format_ident!("{}", descriptor.unique_name);
            let component_instance = format_ident!("c{ndx}");
            Ok(quote! {
                #component_name #component_instance(
                    #(#bindings),*
                );
            })
        })
        .collect::<Result<Vec<TokenStream>, RHDLError>>()?;
    let kernel = descriptor
        .rtl
        .as_ref()
        .ok_or(RHDLError::FunctionNotSynthesizable {
            name: descriptor.unique_name.clone(),
        })?
        .as_vlog()?;
    // Call the verilog function with (i, q), if they exist.
    let i_bind = (circuit_input.bits() != 0).then(|| format_ident!("i"));
    let q_bind = (q_kind.bits() != 0).then(|| format_ident!("q"));
    let kernel_name = format_ident!("{}", kernel.name);
    let module_ident = format_ident!("{}", descriptor.unique_name);
    let output_range: vlog::BitRange = (0..outputs).into();
    let d_bind = (d_kind.bits() != 0).then(|| {
        let d_range: vlog::BitRange = (outputs..(d_kind.bits() + outputs)).into();
        quote! {assign d = od[#d_range];}
    });
    let module: vlog::ModuleDef = vlog::parse_quote_miette! {
        module #module_ident(#(#ports),*);
            #(#declarations;)*
            assign o = od[#output_range];
            #(#child_decls;)*
            #d_bind
            assign od = #kernel_name(#i_bind, #q_bind);
            #kernel
        endmodule
    }?;
    let mut module_list: vlog::ModuleList = module.into();
    for child in descriptor.children.values() {
        let child_hdl = build_asynchronous_hdl(child)?;
        module_list.modules.extend(child_hdl.modules);
    }
    Ok(HDLDescriptor {
        name: descriptor.unique_name.clone(),
        modules: module_list,
    })
}
