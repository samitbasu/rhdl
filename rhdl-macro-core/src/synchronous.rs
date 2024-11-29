use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

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
                #(self.#component_name.init(),)*
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

            #hdl_fn

            #sim_fn
        }
    };

    Ok(quote! {
        #synchronous_impl
    })
}

#[cfg(test)]
mod test {
    use crate::utils::assert_tokens_eq;

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
        let output = derive_synchronous(decl).unwrap();
        let expected = quote!(
            #[derive(Debug, Clone, PartialEq, Digital, Copy)]
            pub struct Q<const N: usize> {
                strobe: <DFF<Bits<N>> as rhdl::core::SynchronousIO>::O,
                value: <Constant<Bits<N>> as rhdl::core::SynchronousIO>::O,
            }
            #[derive(Debug, Clone, PartialEq, Digital, Copy)]
            pub struct D<const N: usize> {
                strobe: <DFF<Bits<N>> as rhdl::core::SynchronousIO>::I,
                value: <Constant<Bits<N>> as rhdl::core::SynchronousIO>::I,
            }
            impl<const N: usize> rhdl::core::Synchronous for Strobe<N> {
                type Q = StrobeQ<N>;
                type D = StrobeD<N>;
                type S = (
                    Self::Q,
                    <DFF<Bits<N>> as rhdl::core::Synchronous>::S,
                    <Constant<Bits<N>> as rhdl::core::Synchronous>::S,
                );
                type Update = pushd<N>;
                const UPDATE: fn(rhdl::core::ClockReset, Self::I, Self::Q) -> (Self::O, Self::D) =
                    pushd::<N>;
                fn name(&self) -> &'static str {
                    stringify!(Strobe)
                }
                fn descriptor(&self) -> rhdl::core::CircuitDescriptor {
                    let mut ret = rhdl::core::synchronous_root_descriptor(self);
                    ret.add_child(stringify!(strobe), &self.strobe);
                    ret.add_child(stringify!(value), &self.value);
                    ret
                }
                fn hdl(&self) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
                    let mut ret = rhdl::core::root_hdl(self, kind)?;
                    ret.add_child(stringify!(strobe), &self.strobe, kind)?;
                    ret.add_child(stringify!(value), &self.value, kind)?;
                    Ok(ret)
                }
                fn sim(
                    &self,
                    clock: rhdl::core::ClockReset,
                    input: <Self as SynchronousIO>::I,
                    state: &mut Self::S,
                    io: &mut Self::Z,
                ) -> <Self as SynchronousIO>::O {
                    for _ in 0..rhdl::core::MAX_ITERS {
                        let prev_state = state.clone();
                        let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
                        rhdl::core::note_push_path(stringify!(strobe));
                        state.0.strobe = self.strobe.sim(
                            clock_reset,
                            internal_inputs.strobe,
                            &mut state.1,
                            &mut io.strobe,
                        );
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(value));
                        state.0.value = self.value.sim(
                            clock_reset,
                            internal_inputs.value,
                            &mut state.2,
                            &mut io.value,
                        );
                        rhdl::core::note_pop_path();
                        if state == &prev_state {
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
        let output = derive_synchronous(decl).unwrap();
        let expected = quote!(
            #[derive(Debug, Clone, PartialEq, Digital,  Copy)]
            pub struct PushQ {
                    strobe: <Strobe<32> as rhdl::core::SynchronousIO>::O,
                    value: <Constant<Bits<8>> as rhdl::core::SynchronousIO>::O,
                    buf_z: <ZDriver<8> as rhdl::core::SynchronousIO>::O,
                    side: <DFF<Side> as rhdl::core::SynchronousIO>::O,
                    latch: <DFF<Bits<8>> as rhdl::core::SynchronousIO>::O,
                }
                #[derive(Debug, Clone, PartialEq, Digital, opy)]
                pub struct PushD {
                    strobe: <Strobe<32> as rhdl::core::SynchronousIO>::I,
                    value: <Constant<Bits<8>> as rhdl::core::SynchronousIO>::I,
                    buf_z: <ZDriver<8> as rhdl::core::SynchronousIO>::I,
                    side: <DFF<Side> as rhdl::core::SynchronousIO>::I,
                    latch: <DFF<Bits<8>> as rhdl::core::SynchronousIO>::I,
                }
                #[derive(Debug, Clone, PartialEq, Copy)]
                pub struct PushZ {
                    strobe: <Strobe<32> as rhdl::core::Synchronous>::Z,
                    value: <Constant<Bits<8>> as rhdl::core::Synchronous>::Z,
                    buf_z: <ZDriver<8> as rhdl::core::Synchronous>::Z,
                    side: <DFF<Side> as rhdl::core::Synchronous>::Z,
                    latch: <DFF<Bits<8>> as rhdl::core::Synchronous>::Z,
                }
                impl rhdl::core::Notable for PushZ {
                    fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                        self.strobe.note((key,stringify!(strobe)), &mut writer);
                        self.value.note((key,stringify!(value)), &mut writer);
                        self.buf_z.note((key,stringify!(buf_z)), &mut writer);
                        self.side.note((key,stringify!(side)), &mut writer);
                        self.latch.note((key,stringify!(latch)), &mut writer);
                    }
                }
                impl rhdl::core::Tristate for PushZ {
                    const N: usize = <Strobe<32> as rhdl::core::Synchronous>::Z::N +
                     <Constant<Bits<8>> as rhdl::core::Synchronous>::Z::N +
                      <ZDriver<8> as rhdl::core::Synchronous>::Z::N +
                       <DFF<Side> as rhdl::core::Synchronous>::Z::N +
                        <DFF<Bits<8>> as rhdl::core::Synchronous>::Z::N +
                         0;
                }
                impl rhdl::core::Synchronous for Push {
                    type Q = PushQ;
                    type D = PushD;
                    type Z = PushZ;
                    type S = (
                        Self::Q,
                        <Strobe<32> as rhdl::core::Synchronous>::S,
                        <Constant<Bits<8>> as rhdl::core::Synchronous>::S,
                        <ZDriver<8> as rhdl::core::Synchronous>::S,
                        <DFF<Side> as rhdl::core::Synchronous>::S,
                        <DFF<Bits<8>> as rhdl::core::Synchronous>::S,
                    );
                    type Update = pushd;
                    const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = pushd;
                fn name(&self) -> &'static str {
                    stringify!(Push)
                }
                fn descriptor(&self) -> rhdl::core::CircuitDescriptor {
                    let mut ret = rhdl::core::root_descriptor(self);
                    ret.add_child(stringify!(strobe), &self.strobe);
                    ret.add_child(stringify!(value), &self.value);
                    ret.add_child(stringify!(buf_z), &self.buf_z);
                    ret.add_child(stringify!(side), &self.side);
                    ret.add_child(stringify!(latch), &self.latch);
                    ret
                }
                fn as_hdl(
                    &self,
                    kind: rhdl::core::HDLKind,
                ) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
                    let mut ret = rhdl::core::root_hdl(self, kind)?;
                    ret.add_child(stringify!(strobe), &self.strobe, kind)?;
                    ret.add_child(stringify!(value), &self.value, kind)?;
                    ret.add_child(stringify!(buf_z), &self.buf_z, kind)?;
                    ret.add_child(stringify!(side), &self.side, kind)?;
                    ret.add_child(stringify!(latch), &self.latch, kind)?;
                    Ok(ret)
                }
                fn sim(
                    &self,
                    input: <Self as SynchronousIO>::I,
                    state: &mut Self::S,
                    io: &mut Self::Z,
                ) -> <Self as SynchronousIO>::O {
                    for _ in 0..rhdl::core::MAX_ITERS {
                        let prev_state = state.clone();
                        let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
                        rhdl::core::note_push_path(stringify!(strobe));
                        state
                            .0
                            .strobe = self
                            .strobe
                            .sim(internal_inputs.strobe, &mut state.1, &mut io.strobe);
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(value));
                        state
                            .0
                            .value = self
                            .value
                            .sim(internal_inputs.value, &mut state.2, &mut io.value);
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(buf_z));
                        state
                            .0
                            .buf_z = self
                            .buf_z
                            .sim(internal_inputs.buf_z, &mut state.3, &mut io.buf_z);
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(side));
                        state
                            .0
                            .side = self.side.sim(internal_inputs.side, &mut state.4, &mut io.side);
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(latch));
                        state
                            .0
                            .latch = self
                            .latch
                            .sim(internal_inputs.latch, &mut state.5, &mut io.latch);
                        rhdl::core::note_pop_path();
                        if state == &prev_state {
                            return outputs;
                        }
                    }
                    panic!("Simulation did not converge");
                }
            }
        );
        assert_tokens_eq(&expected, &output);
    }
}
