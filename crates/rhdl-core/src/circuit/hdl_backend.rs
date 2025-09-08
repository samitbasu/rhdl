use std::collections::BTreeMap;

use crate::Digital;
use crate::types::path::Path;
use crate::types::path::bit_range;
use crate::{Circuit, HDLDescriptor, RHDLError, Synchronous};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rhdl_vlog as vlog;
use syn::parse_quote;
use syn::parse_quote_spanned;
use syn::spanned::Spanned;

pub fn build_hdl<C: Circuit>(
    circuit: &C,
    name: &str,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor(name)?;
    let outputs = C::O::bits();
    let ports = [
        vlog::maybe_port_wire(vlog::Direction::Input, C::I::bits(), "i"),
        vlog::maybe_port_wire(vlog::Direction::Output, C::O::bits(), "o"),
    ];
    let declarations = [
        vlog::maybe_decl_wire(C::O::bits() + C::D::bits(), "od"),
        vlog::maybe_decl_wire(C::D::bits(), "d"),
        vlog::maybe_decl_wire(C::Q::bits(), "q"),
    ];
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
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
            let component_name = format_ident!("{}", descriptor.unique_name);
            let component_instance = format_ident!("c{ndx}");
            Ok(quote! {
                #component_name #component_instance(
                    #input_binding
                    #output_binding
                );
            })
        })
        .collect::<Result<Vec<TokenStream>, RHDLError>>()?;
    let kernel = descriptor
        .rtl
        .as_ref()
        .ok_or(RHDLError::FunctionNotSynthesizable)?
        .as_vlog()?;
    // Call the verilog function with (i, q), if they exist.
    let i_bind = (C::I::bits() != 0).then(|| format_ident!("i"));
    let q_bind = (C::Q::bits() != 0).then(|| format_ident!("q"));
    let kernel_name = format_ident!("{}", kernel.name);
    let module_ident = format_ident!("{}", descriptor.unique_name);
    let output_range: vlog::BitRange = (0..outputs).into();
    let d_bind = (C::D::bits() != 0).then(|| {
        let d_range: vlog::BitRange = (outputs..(C::D::bits() + outputs)).into();
        quote! {assign d = od[#d_range];}
    });
    let module: vlog::ModuleDef = parse_quote! {
        module #module_ident(#(#ports),*);
            #(#declarations;)*
            assign o = od[#output_range];
            #(#child_decls;)*
            #d_bind
            assign od = #kernel_name(#i_bind, #q_bind);
            #kernel
        endmodule
    };
    Ok(HDLDescriptor {
        name: descriptor.unique_name.into(),
        body: module,
        children,
    })
}

// There is a fair amount of overlap between this function and the previous one.  In principle,
// it should be possible to factor out the common bits and DRY up the code.
pub fn build_synchronous_hdl<C: Synchronous>(
    circuit: &C,
    name: &str,
    children: BTreeMap<String, HDLDescriptor>,
) -> Result<HDLDescriptor, RHDLError> {
    let descriptor = circuit.descriptor(name)?;
    let outputs = C::O::bits();
    let ports = [
        vlog::maybe_port_wire(vlog::Direction::Input, 2, "clock_reset"),
        vlog::maybe_port_wire(vlog::Direction::Input, C::I::bits(), "i"),
        vlog::maybe_port_wire(vlog::Direction::Output, C::O::bits(), "o"),
    ];
    let ports = ports.iter().flatten();
    let declarations = [
        vlog::maybe_decl_wire(C::O::bits() + C::D::bits(), "od"),
        vlog::maybe_decl_wire(C::D::bits(), "d"),
        vlog::maybe_decl_wire(C::Q::bits(), "q"),
    ];
    let d_kind = C::D::static_kind();
    let q_kind = C::Q::static_kind();
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
            let component_name = format_ident!("{}", descriptor.unique_name);
            let component_instance = format_ident!("c{ndx}");
            Ok(quote! {
                #component_name #component_instance(
                    .clock_reset(clock_reset),
                    #input_binding,
                    #output_binding
                )
            })
        })
        .collect::<Result<Vec<TokenStream>, RHDLError>>()?;
    let kernel = descriptor
        .rtl
        .as_ref()
        .ok_or(RHDLError::FunctionNotSynthesizable)?
        .as_vlog()?;
    // Call the verilog function with (clock_reset, i, q), if they exist.
    let i_bind = (C::I::bits() != 0).then(|| format_ident!("i"));
    let q_bind = (C::Q::bits() != 0).then(|| format_ident!("q"));
    let args = [Some(format_ident!("clock_reset")), i_bind, q_bind];
    let args = args.iter().flatten();
    let kernel_name = format_ident!("{}", kernel.name);
    let module_ident = format_ident!("{}", descriptor.unique_name);
    let output_range: vlog::BitRange = (0..outputs).into();
    let d_bind = (C::D::bits() != 0).then(|| {
        let d_range: vlog::BitRange = (outputs..(C::D::bits() + outputs)).into();
        quote! {assign d = od[#d_range];}
    });
    let module: vlog::ModuleDef = parse_quote! {
        module #module_ident(#(#ports),*);
            #(#declarations;)*
            assign o = od[#output_range];
            #(#child_decls;)*
            #d_bind
            assign od = #kernel_name(#(#args),*);
            #kernel
        endmodule
    };
    Ok(HDLDescriptor {
        name: descriptor.unique_name.into(),
        body: module,
        children,
    })
}
