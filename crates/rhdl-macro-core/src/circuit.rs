use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

use crate::utils::FieldSet;

pub fn derive_circuit(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_circuit_struct(decl)
}

fn define_descriptor_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn descriptor(&self, name: &str) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
            use std::collections::BTreeMap;
            let mut children : BTreeMap<String, rhdl::core::CircuitDescriptor> = BTreeMap::new();
            #(children.insert(stringify!(#component_name).to_string(),
                self.#component_name.descriptor(
                    &format!("{name}_{}", stringify!(#component_name))
                )?
            );)*
            rhdl::core::build_descriptor::<Self>(name, children)
        }
    }
}

fn define_hdl_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn hdl(&self, name: &str) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
            use std::collections::BTreeMap;
            let mut children : BTreeMap<String, rhdl::core::HDLDescriptor> = BTreeMap::new();
            #(children.insert(stringify!(#component_name).to_string(),
                self.#component_name.hdl(
                    &format!("{name}_{}", stringify!(#component_name))
                )?
            );)*
            rhdl::core::build_hdl(self, name, children)
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
                    self.#component_name.sim(internal_inputs.#component_name, &mut state.#component_index);
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
                #(self.#component_name.init(),)*
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
    let descriptor_fn = define_descriptor_fn(&field_set);
    let hdl_fn = define_hdl_fn(&field_set);
    let sim_fn = define_sim_fn(&field_set);
    let init_fn = define_init_fn(&field_set);
    let circuit_impl = quote! {
        impl #impl_generics rhdl::core::Circuit for #struct_name #ty_generics #where_clause {
            type S = #state_tuple;

            #init_fn

            #descriptor_fn

            #hdl_fn

            #sim_fn
        }
    };

    Ok(quote! {
        #circuit_impl
    })
}

#[cfg(test)]
mod test {
    use crate::utils::assert_tokens_eq;

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
        let expected = quote!(
            impl<const N: usize> rhdl::core::Circuit for Strobe<N> {
                type S = (
                    Self::Q,
                    <DFF<Bits<N>> as rhdl::core::Circuit>::S,
                    <Constant<Bits<N>> as rhdl::core::Circuit>::S,
                );
                fn init(&self) -> Self::S {
                    (
                        <<Self as rhdl::core::CircuitDQ>::Q as rhdl::core::Digital>::dont_care(),
                        self.strobe.init(),
                        self.value.init(),
                    )
                }
                fn descriptor(
                    &self,
                    name: &str,
                ) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
                    use std::collections::BTreeMap;
                    let mut children: BTreeMap<String, rhdl::core::CircuitDescriptor> =
                        BTreeMap::new();
                    children.insert(
                        stringify!(strobe).to_string(),
                        self.strobe
                            .descriptor(&format!("{name}_{}", stringify!(strobe)))?,
                    );
                    children.insert(
                        stringify!(value).to_string(),
                        self.value
                            .descriptor(&format!("{name}_{}", stringify!(value)))?,
                    );
                    rhdl::core::build_descriptor::<Self>(name, children)
                }
                fn hdl(
                    &self,
                    name: &str,
                ) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
                    use std::collections::BTreeMap;
                    let mut children: BTreeMap<String, rhdl::core::HDLDescriptor> = BTreeMap::new();
                    children.insert(
                        stringify!(strobe).to_string(),
                        self.strobe.hdl(&format!("{name}_{}", stringify!(strobe)))?,
                    );
                    children.insert(
                        stringify!(value).to_string(),
                        self.value.hdl(&format!("{name}_{}", stringify!(value)))?,
                    );
                    rhdl::core::build_hdl(self, name, children)
                }
                fn sim(
                    &self,
                    input: <Self as rhdl::core::CircuitIO>::I,
                    state: &mut Self::S,
                ) -> <Self as CircuitIO>::O {
                    let update_fn =
                        <<Self as rhdl::core::CircuitIO>::Kernel as rhdl::core::DigitalFn2>::func();
                    rhdl::core::trace("input", &input);
                    for _ in 0..rhdl::core::MAX_ITERS {
                        let prev_state = state.clone();
                        let (outputs, internal_inputs) = update_fn(input, state.0);
                        rhdl::core::trace_push_path(stringify!(strobe));
                        state.0.strobe = self.strobe.sim(internal_inputs.strobe, &mut state.1);
                        rhdl::core::trace_pop_path();
                        rhdl::core::trace_push_path(stringify!(value));
                        state.0.value = self.value.sim(internal_inputs.value, &mut state.2);
                        rhdl::core::trace_pop_path();
                        if state == &prev_state {
                            rhdl::core::trace("outputs", &outputs);
                            return outputs;
                        }
                    }
                    panic!("Simulation did not converge");
                }
            }
        );
        assert_tokens_eq(&expected, &output);
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
        let expected = quote!(
        impl rhdl::core::Circuit for Push {
            type S = (
                Self::Q,
                <Strobe<32> as rhdl::core::Circuit>::S,
                <Constant<Bits<8>> as rhdl::core::Circuit>::S,
                <ZDriver<8> as rhdl::core::Circuit>::S,
                <DFF<Side> as rhdl::core::Circuit>::S,
                <DFF<Bits<8>> as rhdl::core::Circuit>::S,
            );
            fn init(&self) -> Self::S {
                (
                    <<Self as rhdl::core::CircuitDQ>::Q as rhdl::core::Digital>::dont_care(),
                    self.strobe.init(),
                    self.value.init(),
                    self.buf_z.init(),
                    self.side.init(),
                    self.latch.init(),
                )
            }
            fn descriptor(
                &self,
                name: &str,
            ) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
                use std::collections::BTreeMap;
                let mut children: BTreeMap<String, rhdl::core::CircuitDescriptor> = BTreeMap::new();
                children.insert(
                    stringify!(strobe).to_string(),
                    self.strobe
                        .descriptor(&format!("{name}_{}", stringify!(strobe)))?,
                );
                children.insert(
                    stringify!(value).to_string(),
                    self.value
                        .descriptor(&format!("{name}_{}", stringify!(value)))?,
                );
                children.insert(
                    stringify!(buf_z).to_string(),
                    self.buf_z
                        .descriptor(&format!("{name}_{}", stringify!(buf_z)))?,
                );
                children.insert(
                    stringify!(side).to_string(),
                    self.side
                        .descriptor(&format!("{name}_{}", stringify!(side)))?,
                );
                children.insert(
                    stringify!(latch).to_string(),
                    self.latch
                        .descriptor(&format!("{name}_{}", stringify!(latch)))?,
                );
                rhdl::core::build_descriptor::<Self>(name, children)
            }
            fn hdl(&self, name: &str) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
                use std::collections::BTreeMap;
                let mut children: BTreeMap<String, rhdl::core::HDLDescriptor> = BTreeMap::new();
                children.insert(
                    stringify!(strobe).to_string(),
                    self.strobe.hdl(&format!("{name}_{}", stringify!(strobe)))?,
                );
                children.insert(
                    stringify!(value).to_string(),
                    self.value.hdl(&format!("{name}_{}", stringify!(value)))?,
                );
                children.insert(
                    stringify!(buf_z).to_string(),
                    self.buf_z.hdl(&format!("{name}_{}", stringify!(buf_z)))?,
                );
                children.insert(
                    stringify!(side).to_string(),
                    self.side.hdl(&format!("{name}_{}", stringify!(side)))?,
                );
                children.insert(
                    stringify!(latch).to_string(),
                    self.latch.hdl(&format!("{name}_{}", stringify!(latch)))?,
                );
                rhdl::core::build_hdl(self, name, children)
            }
            fn sim(
                &self,
                input: <Self as rhdl::core::CircuitIO>::I,
                state: &mut Self::S,
            ) -> <Self as CircuitIO>::O {
                let update_fn = <<Self as rhdl::core::CircuitIO>::Kernel as rhdl::core::DigitalFn2>::func();
                rhdl::core::trace("input", &input);
                for _ in 0..rhdl::core::MAX_ITERS {
                    let prev_state = state.clone();
                    let (outputs, internal_inputs) = update_fn(input, state.0);
                    rhdl::core::trace_push_path(stringify!(strobe));
                    state.0.strobe = self.strobe.sim(internal_inputs.strobe, &mut state.1);
                    rhdl::core::trace_pop_path();
                    rhdl::core::trace_push_path(stringify!(value));
                    state.0.value = self.value.sim(internal_inputs.value, &mut state.2);
                    rhdl::core::trace_pop_path();
                    rhdl::core::trace_push_path(stringify!(buf_z));
                    state.0.buf_z = self.buf_z.sim(internal_inputs.buf_z, &mut state.3);
                    rhdl::core::trace_pop_path();
                    rhdl::core::trace_push_path(stringify!(side));
                    state.0.side = self.side.sim(internal_inputs.side, &mut state.4);
                    rhdl::core::trace_pop_path();
                    rhdl::core::trace_push_path(stringify!(latch));
                    state.0.latch = self.latch.sim(internal_inputs.latch, &mut state.5);
                    rhdl::core::trace_pop_path();
                    if state == &prev_state {
                        rhdl::core::trace("outputs", &outputs);
                        return outputs;
                    }
                }
                panic!("Simulation did not converge");
            }
        });
        assert_tokens_eq(&expected, &output);
    }
}
