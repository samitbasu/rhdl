use syn::visit_mut::VisitMut;
use syn::{parse_quote, Arm, Expr, Lit, LitInt, Pat};

pub struct CustomSuffix;

impl VisitMut for CustomSuffix {
    fn visit_expr_mut(&mut self, node: &mut Expr) {
        if let Expr::Lit(expr) = &node {
            if let Lit::Int(int) = &expr.lit {
                let suffix = int.suffix().replace('_', "");
                let unsuffixed: LitInt = syn::parse_str(int.base10_digits()).unwrap();
                let suffix_width: String = suffix.chars().skip(1).collect();
                if let Ok(suffix_width_digits) = suffix_width.parse::<usize>() {
                    if suffix.starts_with('u') {
                        *node = parse_quote!(rhdl_bits::Bits::<#suffix_width_digits>(#unsuffixed))
                    } else if suffix.starts_with('i') {
                        *node =
                            parse_quote!(rhdl_bits::SignedBits::<#suffix_width_digits>(#unsuffixed))
                    }
                }
            }
        }
        syn::visit_mut::visit_expr_mut(self, node);
    }
    fn visit_pat_mut(&mut self, node: &mut Pat) {
        if let Pat::Lit(lit) = node {
            if let Lit::Int(int) = &lit.lit {
                let suffix = int.suffix().replace('_', "");
                let unsuffixed: LitInt = syn::parse_str(int.base10_digits()).unwrap();
                let suffix_width: String = suffix.chars().skip(1).collect();
                if let Ok(suffix_width_digits) = suffix_width.parse::<usize>() {
                    if suffix.starts_with('u') {
                        *node = parse_quote!(rhdl_bits::Bits::<#suffix_width_digits>(#unsuffixed))
                    } else if suffix.starts_with('i') {
                        *node =
                            parse_quote!(rhdl_bits::SignedBits::<#suffix_width_digits>(#unsuffixed))
                    }
                }
            }
        }
        syn::visit_mut::visit_pat_mut(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_custom_suffix() {
        let num = 0xdedbeef;
        let test_code = quote! {
            fn update() {
                let b = 0x4313_u8;
                let j = 342;
                let i = 0x432_u8;
                let a = 54_234_i14;
                let p = 0o644_u12;
                let h = 0b1010110_u_10;
                let h = 0b1010110_u_10;
                let p = 0b110011_i15;
                let q: u8 = 4;
                let z = a.c;
                let w = (a, a);
                a.c[1] = q + 3;
                a.c = [0; 3];
                a.c = [1, 2, 3];
                let q = (1, (0, 5), 6);
                let (q0, (q1, q1b), q2): (u8, (u8, u8), u16) = q; // Tuple destructuring
                a.a = 2 + 3 + q1;
                let z;
                if 1 > 3 {
                    z = 2_u4;
                    z = 0x432_u10;
                } else {
                    z = 5;
                }
                match z {
                    1_u4 => {
                        z = 2_u4;
                        z = 0x432_u10;
                    }
                    2_u4 => {
                        z = 5;
                    }
                }
                let p = 0b110011_i15;
            }
        };
        let mut item = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        CustomSuffix.visit_item_fn_mut(&mut item);
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }
}
