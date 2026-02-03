use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::FieldSet;

pub fn derive_synchronous(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_synchronous_struct(decl)
}

fn define_children_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn children(&self, parent_scope: &rhdl::core::ScopedName) -> impl Iterator<Item = Result<rhdl::core::Descriptor<rhdl::core::SyncKind>, rhdl::core::RHDLError>> {
            [
                #(Synchronous::descriptor(&self.#component_name, parent_scope.with(stringify!(#component_name)))),*
            ].into_iter()
        }
    }
}

fn define_init_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn init(&self) -> Self::S {
            (
                <<Self as rhdl::core::SynchronousDQ>::Q as rhdl::core::Digital>::dont_care(),
                #(Synchronous::init(&self.#component_name)),*
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
                    Synchronous::sim(&self.#component_name, clock_reset, internal_inputs.#component_name, &mut state.#component_index);
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
    let children_fn = define_children_fn(&field_set);
    let sim_fn = define_sim_fn(&field_set);
    let synchronous_impl = quote! {
        impl #impl_generics rhdl::core::Synchronous for #struct_name #ty_generics #where_clause {
            type S = #state_tuple;

            #init_fn

            #children_fn

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
