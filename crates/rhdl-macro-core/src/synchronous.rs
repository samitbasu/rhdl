use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::FieldSet;

pub fn derive_synchronous(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_synchronous_struct(decl)
}

fn define_descriptor_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn descriptor(&self, name: &str) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
            use std::collections::BTreeMap;
            let mut children: BTreeMap<String, CircuitDescriptor> = BTreeMap::new();
            #(children.insert(stringify!(#component_name).to_string(),
                self.#component_name.descriptor(
                    &format!("{name}_{}", stringify!(#component_name))
                )?
            );)*
            rhdl::core::build_synchronous_descriptor::<Self>(name, children)
        }
    }
}

fn define_hdl_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn hdl(&self, name: &str) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
            use std::collections::BTreeMap;
            let mut children: BTreeMap<String, HDLDescriptor> = BTreeMap::new();
            #(children.insert(stringify!(#component_name).to_string(),
                self.#component_name.hdl(
                    &format!("{name}_{}", stringify!(#component_name))
                )?);)*
            rhdl::core::build_synchronous_hdl(self, name, children)
        }
    }
}

fn define_init_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn init(&self) -> Self::S {
            (
                <<Self as rhdl::core::SynchronousDQ>::Q as rhdl::core::Digital>::dont_care(),
                #(self.#component_name.init()),*
            )
        }
    }
}

fn define_sim_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    let component_index = (1..=component_name.len())
        .map(syn::Index::from)
        .collect::<Vec<_>>();
    quote! {
        fn sim(&self, clock_reset: rhdl::core::ClockReset, input: <Self as SynchronousIO>::I, state: &mut Self::S ) -> <Self as SynchronousIO>::O {
            let update_fn = <<Self as SynchronousIO>::Kernel as DigitalFn3>::func();
            rhdl::core::trace("input", &input);
            for _ in 0..rhdl::core::MAX_ITERS {
                let prev_state = state.clone();
                let (outputs, internal_inputs) = update_fn(clock_reset, input, state.0);
                #(
                    rhdl::core::trace_push_path(stringify!(#component_name));
                    state.0.#component_name =
                    self.#component_name.sim(clock_reset, internal_inputs.#component_name, &mut state.#component_index);
                    rhdl::core::trace_pop_path();
                )*
                if state == &prev_state {
                    rhdl::core::trace("outputs", &outputs);
                    return outputs;
                }
            }
            panic!("Simulation did not converge");
        }
    }
}

fn derive_synchronous_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Struct(s) = &decl.data else {
        return Err(syn::Error::new(
            decl.span(),
            "Synchronous can only be derived for structs with named fields",
        ));
    };
    let field_set = FieldSet::try_from(&s.fields)?;
    let component_ty = &field_set.component_ty;
    // Add a tuple of the states of the components
    let state_tuple = quote!((Self::Q, #(<#component_ty as rhdl::core::Synchronous>::S),*));
    let init_fn = define_init_fn(&field_set);
    let descriptor_fn = define_descriptor_fn(&field_set);
    let hdl_fn = define_hdl_fn(&field_set);
    let sim_fn = define_sim_fn(&field_set);
    let synchronous_impl = quote! {
        impl #impl_generics rhdl::core::Synchronous for #struct_name #ty_generics #where_clause {
            type S = #state_tuple;

            #init_fn

            #descriptor_fn

            //#hdl_fn

            #sim_fn
        }
    };

    Ok(quote! {
        #synchronous_impl
    })
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;

    use super::*;

    #[test]
    fn test_template_synchronous_derive() {
        let decl = quote!(
            #[rhdl(kernel = pushd::<N>)]
            pub struct Strobe<const N: usize> {
                strobe: DFF<Bits<N>>,
                value: Constant<Bits<N>>,
            }
        );
        let output = derive_synchronous(decl).unwrap().to_string();
        let expected = expect_file!["expect/template_synchronous_derive.expect"];
        expected.assert_eq(&output);
    }

    #[test]
    fn test_synchronous_derive() {
        let decl = quote!(
            #[rhdl(kernel = pushd)]
            pub struct Push {
                strobe: Strobe<32>,
                value: Constant<Bits<8>>,
                buf_z: ZDriver<8>,
                side: DFF<Side>,
                latch: DFF<Bits<8>>,
            }
        );
        let output = derive_synchronous(decl).unwrap().to_string();
        let expected = expect_file!["expect/synchronous_derive.expect"];
        expected.assert_eq(&output);
    }
}
