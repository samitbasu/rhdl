use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput};

fn derive_digital_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    match &decl.data {
        Data::Struct(s) => match s.fields {
            syn::Fields::Named(_) => derive_digital_named_struct(decl),
            _ => Err(syn::Error::new_spanned(
                decl,
                "DigitalStruct can only be derived for named structs",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            decl,
            "DigitalStruct can only be derived for structs",
        )),
    }
}

fn derive_digital_named_struct(decl: DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &decl.ident;
    decl.generics
        .params
        .iter()
        .for_each(|x| println!("{:?}", x));
    Ok(quote!())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_derive_digital_struct() {
        let input: DeriveInput = syn::parse_quote! {
            #[derive(DigitalStruct)]
            struct Foo {
                a: u8,
                b: u8,
            }
        };
        let output = derive_digital_struct(input).unwrap();
        println!("{}", output);
    }

    #[test]
    fn test_derive_digital_struct_with_generics() {
        let input: DeriveInput = syn::parse_quote! {
            #[derive(DigitalStruct)]
            struct Foo<T, S> {
                a: S,
                b: T,
                c: [u8; 3],
            }
        };
        /*

        The type function for this should look something like:

        fn Foo_hdl_ty() -> Ty {
            // Collect generic parameters
            let generic_types = vec!["T", "S"];
            Ty::Struct(TyMap {
                name: "Foo",
                fields: vec![
                    ("a", Ty::Var(1)),
                    ("b", Ty::Var(0)),
                    ("c", Ty::Array(vec![Ty::Const(Bits::Unsigned(8)); 3])),
                ],
                generic_types: generic_types,
            })
        }

         */

        let output = derive_digital_struct(input).unwrap();
        println!("{}", output);
    }
}
