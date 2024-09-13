use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Attribute, Data, DeriveInput, Expr, ExprPath};

use crate::utils::is_auto_dq_from_attributes;

pub fn derive_circuit(input: TokenStream) -> syn::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    derive_circuit_struct(decl)
}

pub struct FieldSet<'a> {
    component_name: Vec<syn::Ident>,
    component_ty: Vec<&'a syn::Type>,
}

impl<'a> TryFrom<&'a syn::Fields> for FieldSet<'a> {
    type Error = syn::Error;

    fn try_from(fields: &'a syn::Fields) -> syn::Result<Self> {
        let mut component_name = Vec::new();
        let mut component_ty = Vec::new();
        for field in fields.iter() {
            component_name.push(field.ident.clone().ok_or_else(|| {
                syn::Error::new(field.span(), "Circuit components (fields) must have names")
            })?);
            component_ty.push(&field.ty);
        }
        Ok(FieldSet {
            component_name,
            component_ty,
        })
    }
}

fn define_descriptor_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn descriptor(&self) -> Result<rhdl::core::CircuitDescriptor, rhdl::core::RHDLError> {
            let mut ret = rhdl::core::root_descriptor(self)?;
            #(ret.add_child(stringify!(#component_name), &self.#component_name)?;)*
            Ok(ret)
        }
    }
}

fn define_hdl_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    quote! {
        fn as_hdl(&self, kind: rhdl::core::HDLKind) -> Result<rhdl::core::HDLDescriptor, rhdl::core::RHDLError> {
            let mut ret = rhdl::core::root_hdl(self, kind)?;
            #(ret.add_child(stringify!(#component_name), &self.#component_name, kind)?;)*
            Ok(ret)
        }
    }
}

fn define_sim_fn(field_set: &FieldSet) -> TokenStream {
    let component_name = &field_set.component_name;
    let component_index = (1..=component_name.len())
        .map(syn::Index::from)
        .collect::<Vec<_>>();
    quote! {
        fn sim(&self, input: <Self as CircuitIO>::I, state: &mut Self::S , io: &mut Self::Z ) -> <Self as CircuitIO>::O {
            rhdl::core::note("input", input);
            for _ in 0..rhdl::core::MAX_ITERS {
                let prev_state = state.clone();
                let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
                #(
                    rhdl::core::note_push_path(stringify!(#component_name));
                    state.0.#component_name =
                    self.#component_name.sim(internal_inputs.#component_name, &mut state.#component_index, &mut io.#component_name);
                    rhdl::core::note_pop_path();
                )*
                if state == &prev_state {
                    rhdl::core::note("outputs", outputs);
                    return outputs;
                }
            }
            panic!("Simulation did not converge");
        }
    }
}

fn extract_kernel_name_from_attributes(attrs: &[Attribute]) -> syn::Result<Option<ExprPath>> {
    for attr in attrs {
        if attr.path().is_ident("rhdl") {
            let Expr::Assign(assign) = attr.parse_args::<Expr>()? else {
                return Err(syn::Error::new(
                    attr.span(),
                    "Expected rhdl attribute to be of the form #[rhdl(kernel = name)]",
                ));
            };
            let Expr::Path(path) = *assign.left else {
                return Err(syn::Error::new(
                    assign.left.span(),
                    "Expected rhdl attribute to be of the form #[rhdl(kernel = name)]",
                ));
            };
            if !path.path.is_ident("kernel") {
                return Err(syn::Error::new(
                    path.span(),
                    "Expected rhdl attribute to be of the form #[rhdl(kernel = name)]",
                ));
            }
            let Expr::Path(expr_path) = *assign.right else {
                return Err(syn::Error::new(
                    assign.right.span(),
                    "Expected rhdl attribute to be of the form #[rhdl(kernel = name)]",
                ));
            };
            return Ok(Some(expr_path));
        }
    }
    Ok(None)
}

fn derive_circuit_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    let auto_dq = is_auto_dq_from_attributes(&decl.attrs);
    let kernel_name = extract_kernel_name_from_attributes(&decl.attrs)?;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    let Data::Struct(s) = &decl.data else {
        return Err(syn::Error::new(
            decl.span(),
            "Circuit can only be derived for structs with named fields",
        ));
    };
    let field_set = FieldSet::try_from(&s.fields)?;
    let component_ty = &field_set.component_ty;
    let component_name = &field_set.component_name;
    let generics = &decl.generics;
    // Create a new struct by appending a Q to the name of the struct, and for each field, map
    // the type to <ty as rhdl::core::Circuit>::O,
    let new_struct_q = quote! {
        #[derive(Debug, Clone, PartialEq, Digital, Copy, Timed)]
        pub struct Q #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::O),*
        }
    };
    // Repeat with D and ::I
    let new_struct_d = quote! {
        #[derive(Debug, Clone, PartialEq, Digital, Copy, Timed)]
        pub struct D #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::CircuitIO>::I),*
        }
    };
    // Repeat again with Z and ::Z
    let name_z = format_ident!("{}Z", struct_name);
    let new_struct_z = quote!(
        #[derive(Debug, Clone, PartialEq, Copy, Default)]
        pub struct #name_z #generics #where_clause {
            #(#component_name: <#component_ty as rhdl::core::Circuit>::Z),*
        }
    );
    // Add an implementation of Notable for the Z struct.
    // Should be of the form:
    // impl rhdl::core::Notable for StructZ {
    // fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl NoteWriter) {
    //     self.field1.note((key, stringify!(field1)), &mut writer);
    //     self.field2.note((key, stringify!(field2)), &mut writer);
    //     // ...
    // }
    // }
    let component_name = &field_set.component_name;
    let notable_z_impl = quote! {
        impl #impl_generics rhdl::core::Notable for #name_z #ty_generics {
            fn note(&self, key: impl rhdl::core::NoteKey, mut writer: impl rhdl::core::NoteWriter) {
                #(self.#component_name.note((key, stringify!(#component_name)), &mut writer);)*
            }
        }
    };
    // Add an impl of rhdl::core::Tristate for the Z struct.  It should add the ::N constants of
    // each field.
    // Should be of the form:
    // impl rhdl::core::Tristate for StructZ {
    //     const N: usize = <Field1 as rhdl::core::Circuit>::Z::N + <Field2 as rhdl::core::Circuit>::Z::N + ...;
    // }
    let component_ty = &field_set.component_ty;
    let tristate_z_impl = quote! {
        impl #impl_generics rhdl::core::Tristate for #name_z #ty_generics {
            const N: usize = #(<#component_ty as rhdl::core::Circuit>::Z::N +)* 0;
        }
    };
    // Add a tuple of the states of the components
    let state_tuple = quote!((Self::Q, #(<#component_ty as rhdl::core::Circuit>::S),*));
    let descriptor_fn = define_descriptor_fn(&field_set);
    let hdl_fn = define_hdl_fn(&field_set);
    let sim_fn = define_sim_fn(&field_set);
    let name_fn = quote!(
        fn name(&self) -> &'static str {
            stringify!(#struct_name)
        }
    );
    let dq_section = if !auto_dq {
        quote! {}
    } else {
        quote! {
            #new_struct_q
            #new_struct_d

            impl #impl_generics rhdl::core::CircuitDQ for #struct_name #ty_generics #where_clause {
                type Q = Q #ty_generics;
                type D = D #ty_generics;
            }
        }
    };
    let circuit_impl = quote! {
        impl #impl_generics rhdl::core::Circuit for #struct_name #ty_generics #where_clause {
            type Z = #name_z #ty_generics;
            type S = #state_tuple;

            type Update = #kernel_name;

            const UPDATE: fn(<Self as CircuitIO>::I,
                <Self as CircuitDQ>::Q) -> (
                    <Self as CircuitIO>::O,
                    <Self as CircuitDQ>::D) = #kernel_name;

            #name_fn

            #descriptor_fn

            #hdl_fn

            #sim_fn
        }
    };

    Ok(quote! {
        #dq_section
        #new_struct_z
        #notable_z_impl
        #tristate_z_impl
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
            #[rhdl(kernel = pushd::<N>)]
            pub struct Strobe<const N: usize> {
                strobe: DFF<Bits<N>>,
                value: Constant<Bits<N>>,
            }
        );
        let output = derive_circuit(decl).unwrap();
        let expected = quote!(
            #[derive(Debug, Clone, PartialEq, Digital, Copy)]
            pub struct StrobeQ<const N: usize> {
                strobe: <DFF<Bits<N>> as rhdl::core::CircuitIO>::O,
                value: <Constant<Bits<N>> as rhdl::core::CircuitIO>::O,
            }
            #[derive(Debug, Clone, PartialEq, Digital, Copy)]
            pub struct StrobeD<const N: usize> {
                strobe: <DFF<Bits<N>> as rhdl::core::CircuitIO>::I,
                value: <Constant<Bits<N>> as rhdl::core::CircuitIO>::I,
            }
            #[derive(Debug, Clone, PartialEq, Copy)]
            pub struct StrobeZ<const N: usize> {
                strobe: <DFF<Bits<N>> as rhdl::core::Circuit>::Z,
                value: <Constant<Bits<N>> as rhdl::core::Circuit>::Z,
            }
            impl<const N: usize> rhdl::core::Notable for StrobeZ<N> {
                fn note(
                    &self,
                    key: impl rhdl::core::NoteKey,
                    mut writer: impl rhdl::core::NoteWriter,
                ) {
                    self.strobe.note((key, stringify!(strobe)), &mut writer);
                    self.value.note((key, stringify!(value)), &mut writer);
                }
            }
            impl<const N: usize> rhdl::core::Tristate for StrobeZ<N> {
                const N: usize = <DFF<Bits<N>> as rhdl::core::Circuit>::Z::N
                    + <Constant<Bits<N>> as rhdl::core::Circuit>::Z::N
                    + 0;
            }
            impl<const N: usize> rhdl::core::Circuit for Strobe<N> {
                type Q = StrobeQ<N>;
                type D = StrobeD<N>;
                type Z = StrobeZ<N>;
                type S = (
                    Self::Q,
                    <DFF<Bits<N>> as rhdl::core::Circuit>::S,
                    <Constant<Bits<N>> as rhdl::core::Circuit>::S,
                );
                type Update = pushd<N>;
                const UPDATE: fn(Self::I, Self::Q) -> (Self::O, Self::D) = pushd::<N>;
                fn name(&self) -> &'static str {
                    stringify!(Strobe)
                }
                fn descriptor(&self) -> rhdl::core::CircuitDescriptor {
                    let mut ret = rhdl::core::root_descriptor(self);
                    ret.add_child(stringify!(strobe), &self.strobe);
                    ret.add_child(stringify!(value), &self.value);
                    ret
                }
                fn as_hdl(
                    &self,
                    kind: rhdl::core::HDLKind,
                ) -> anyhow::Result<rhdl::core::HDLDescriptor> {
                    let mut ret = rhdl::core::root_hdl(self, kind)?;
                    ret.add_child(stringify!(strobe), &self.strobe, kind)?;
                    ret.add_child(stringify!(value), &self.value, kind)?;
                    Ok(ret)
                }
                fn sim(
                    &self,
                    input: <Self as CircuitIO>::I,
                    state: &mut Self::S,
                    io: &mut Self::Z,
                ) -> <Self as CircuitIO>::O {
                    rhdl::core::note("input", input);
                    for _ in 0..rhdl::core::MAX_ITERS {
                        let prev_state = state.clone();
                        let (outputs, internal_inputs) = Self::UPDATE(input, state.0);
                        rhdl::core::note_push_path(stringify!(strobe));
                        state.0.strobe =
                            self.strobe
                                .sim(internal_inputs.strobe, &mut state.1, &mut io.strobe);
                        rhdl::core::note_pop_path();
                        rhdl::core::note_push_path(stringify!(value));
                        state.0.value =
                            self.value
                                .sim(internal_inputs.value, &mut state.2, &mut io.value);
                        rhdl::core::note_pop_path();
                        if state == &prev_state {
                            rhdl::core::note("outputs", outputs);
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
            #[rhdl(kernel = pushd)]
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
            #[derive(Debug, Clone, PartialEq, Digital,  Copy)]
            pub struct PushQ {
                    strobe: <Strobe<32> as rhdl::core::CircuitIO>::O,
                    value: <Constant<Bits<8>> as rhdl::core::CircuitIO>::O,
                    buf_z: <ZDriver<8> as rhdl::core::CircuitIO>::O,
                    side: <DFF<Side> as rhdl::core::CircuitIO>::O,
                    latch: <DFF<Bits<8>> as rhdl::core::CircuitIO>::O,
                }
                #[derive(Debug, Clone, PartialEq, Digital, opy)]
                pub struct PushD {
                    strobe: <Strobe<32> as rhdl::core::CircuitIO>::I,
                    value: <Constant<Bits<8>> as rhdl::core::CircuitIO>::I,
                    buf_z: <ZDriver<8> as rhdl::core::CircuitIO>::I,
                    side: <DFF<Side> as rhdl::core::CircuitIO>::I,
                    latch: <DFF<Bits<8>> as rhdl::core::CircuitIO>::I,
                }
                #[derive(Debug, Clone, PartialEq, Copy)]
                pub struct PushZ {
                    strobe: <Strobe<32> as rhdl::core::Circuit>::Z,
                    value: <Constant<Bits<8>> as rhdl::core::Circuit>::Z,
                    buf_z: <ZDriver<8> as rhdl::core::Circuit>::Z,
                    side: <DFF<Side> as rhdl::core::Circuit>::Z,
                    latch: <DFF<Bits<8>> as rhdl::core::Circuit>::Z,
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
                    const N: usize = <Strobe<32> as rhdl::core::Circuit>::Z::N +
                     <Constant<Bits<8>> as rhdl::core::Circuit>::Z::N +
                      <ZDriver<8> as rhdl::core::Circuit>::Z::N +
                       <DFF<Side> as rhdl::core::Circuit>::Z::N +
                        <DFF<Bits<8>> as rhdl::core::Circuit>::Z::N +
                         0;
                }
                impl rhdl::core::Circuit for Push {
                    type Q = PushQ;
                    type D = PushD;
                    type Z = PushZ;
                    type S = (
                        Self::Q,
                        <Strobe<32> as rhdl::core::Circuit>::S,
                        <Constant<Bits<8>> as rhdl::core::Circuit>::S,
                        <ZDriver<8> as rhdl::core::Circuit>::S,
                        <DFF<Side> as rhdl::core::Circuit>::S,
                        <DFF<Bits<8>> as rhdl::core::Circuit>::S,
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
                ) -> anyhow::Result<rhdl::core::HDLDescriptor> {
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
                    input: <Self as CircuitIO>::I,
                    state: &mut Self::S,
                    io: &mut Self::Z,
                ) -> <Self as CircuitIO>::O {
                    rhdl::core::note("input", input);
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
                            rhdl::core::note("outputs", outputs);
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
