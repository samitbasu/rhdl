use crate::{
    CircuitDescriptor, HDLDescriptor, RHDLError,
    types::path::{Path, bit_range},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rhdl_vlog::{self as vlog, parse_quote_miette};
use syn::parse_quote;

/// Build an HDL description of a synchronous circuit
///
/// This function is not typically called directly.  It is used by the `Synchronous` proc-macro
/// to construct the HDL description of a synchronous circuit.
pub fn build_synchronous_hdl(descriptor: &CircuitDescriptor) -> Result<HDLDescriptor, RHDLError> {
    let circuit_output = descriptor.output_kind;
    let circuit_input = descriptor.input_kind;
    let d_kind = descriptor.d_kind;
    let q_kind = descriptor.q_kind;
    let outputs = circuit_output.bits();
    let ports = [
        vlog::maybe_port_wire(vlog::Direction::Input, 2, "clock_reset"),
        vlog::maybe_port_wire(vlog::Direction::Input, circuit_input.bits(), "i"),
        vlog::maybe_port_wire(vlog::Direction::Output, circuit_output.bits(), "o"),
    ];
    let ports = ports.iter().flatten();
    let declarations = [
        vlog::maybe_decl_wire(circuit_output.bits() + d_kind.bits(), "od"),
        vlog::maybe_decl_wire(d_kind.bits(), "d"),
        vlog::maybe_decl_wire(q_kind.bits(), "q"),
    ];
    let child_decls = descriptor
        .children
        .iter()
        .filter(|(_, desc)| !desc.output_kind.is_empty())
        .enumerate()
        .map(|(ndx, (local_name, descriptor))| {
            let child_path = Path::default().field(local_name);
            let (d_range, _) = bit_range(d_kind, &child_path)?;
            let (q_range, _) = bit_range(q_kind, &child_path)?;
            let input_binding = vlog::maybe_connect("i", "d", d_range);
            let output_binding = vlog::maybe_connect("o", "q", q_range);
            let bindings = [
                Some(parse_quote! {.clock_reset(clock_reset)}),
                input_binding,
                output_binding,
            ];
            let bindings = bindings.iter().flatten();
            let component_name = format_ident!("{}", descriptor.unique_name);
            let component_instance = format_ident!("c{ndx}");
            Ok(quote! {
                #component_name #component_instance(
                    #(#bindings),*
                )
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
    // Call the verilog function with (clock_reset, i, q), if they exist.
    let i_bind = (circuit_input.bits() != 0).then(|| format_ident!("i"));
    let q_bind = (q_kind.bits() != 0).then(|| format_ident!("q"));
    let args = [Some(format_ident!("clock_reset")), i_bind, q_bind];
    let args = args.iter().flatten();
    let kernel_name = format_ident!("{}", kernel.name);
    let module_ident = format_ident!("{}", descriptor.unique_name);
    let output_range: vlog::BitRange = (0..outputs).into();
    let d_bind = (d_kind.bits() != 0).then(|| {
        let d_range: vlog::BitRange = (outputs..(d_kind.bits() + outputs)).into();
        quote! {assign d = od[#d_range];}
    });
    let module: vlog::ModuleDef = parse_quote_miette! {
        module #module_ident(#(#ports),*);
            #(#declarations;)*
            assign o = od[#output_range];
            #(#child_decls;)*
            #d_bind
            assign od = #kernel_name(#(#args),*);
            #kernel
        endmodule
    }?;
    let mut module_list: vlog::ModuleList = module.into();
    for child in descriptor.children.values() {
        let child_hdl = build_synchronous_hdl(child)?;
        module_list.modules.extend(child_hdl.modules);
    }
    Ok(HDLDescriptor {
        name: descriptor.unique_name.clone(),
        modules: module_list,
    })
}
