use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::FieldSet;

pub fn derive_circuit(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_circuit_struct(decl)
}

fn define_children_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn children(&self, parent_scope: &rhdl::core::ScopedName) -> impl Iterator<Item = Result<rhdl::core::Descriptor<rhdl::core::AsyncKind>, rhdl::core::RHDLError>> {
            [
                #(Circuit::descriptor(&self.#component_name, parent_scope.with(stringify!(#component_name)))),*
            ].into_iter()
        }
    }
}

fn define_sim_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    let component_index = (1..=component_name.len())
        .map(syn::Index::from)
        .collect::<Vec<_>>();
    quote! {
        fn sim(&self, input: <Self as rhdl::core::CircuitIO>::I, state: &mut Self::S ) -> <Self as CircuitIO>::O {
            let update_fn = <<Self as rhdl::core::CircuitIO>::Kernel as rhdl::core::DigitalFn2>::func();
            rhdl::core::trace("input", &input);
            for _ in 0..rhdl::core::MAX_ITERS {
                let prev_state = state.clone();
                let (outputs, internal_inputs) = update_fn(input, state.0);
                #(
                    rhdl::core::trace_push_path(stringify!(#component_name));
                    state.0.#component_name =
                    Circuit::sim(&self.#component_name, internal_inputs.#component_name, &mut state.#component_index);
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

fn define_init_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn init(&self) -> Self::S {
            (
                <<Self as rhdl::core::CircuitDQ>::Q as rhdl::core::Digital>::dont_care(),
                #(Circuit::init(&self.#component_name)),*
            )
        }
    }
}

fn derive_circuit_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Struct(s) = &decl.data else {
        return Err(syn::Error::new(
            decl.span(),
            "Circuit can only be derived for structs with named fields",
        ));
    };
    let field_set = FieldSet::try_from(&s.fields)?;
    let component_ty = &field_set.component_ty;
    // Add a tuple of the states of the components
    let state_tuple = quote!((Self::Q, #(<#component_ty as rhdl::core::Circuit>::S),*));
    let children_fn = define_children_fn(&field_set);
    let sim_fn = define_sim_fn(&field_set);
    let init_fn = define_init_fn(&field_set);
    let circuit_impl = quote! {
        impl #impl_generics rhdl::core::Circuit for #struct_name #ty_generics #where_clause {
            type S = #state_tuple;

            #init_fn

            #children_fn

            #sim_fn
        }
    };

    Ok(quote! {
        #circuit_impl
    })
}

#[cfg(test)]
mod test {
    use expect_test::expect_file;

    use crate::utils::pretty_print;

    use super::*;

    #[test]
    fn test_template_circuit_derive() {
        let decl = quote!(
            pub struct Strobe<const N: usize> {
                strobe: DFF<Bits<N>>,
                value: Constant<Bits<N>>,
            }
        );
        let output = derive_circuit(decl).unwrap();
        let expected = expect_file!["expect/test_template_circuit_derive.expect"];
        expected.assert_eq(pretty_print(&output).as_str());
    }

    #[test]
    fn test_circuit_derive() {
        let decl = quote!(
            pub struct Push {
                strobe: Strobe<32>,
                value: Constant<Bits<8>>,
                buf_z: ZDriver<8>,
                side: DFF<Side>,
                latch: DFF<Bits<8>>,
            }
        );
        let output = derive_circuit(decl).unwrap();
        let expected = expect_file!["expect/circuit_derive.expect"];
        expected.assert_eq(pretty_print(&output).as_str());
    }
}
