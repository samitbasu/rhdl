use evalexpr::ContextWithMutableVariables;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Ident, LitInt, Token, parse::Parse, punctuated::Punctuated, visit_mut::VisitMut};

#[derive(Debug, PartialEq, Eq)]
struct RangedDecl {
    name: Ident,
    in_token: Token![in],
    start: LitInt,
    range_token: Token![..],
    end: LitInt,
}

impl Parse for RangedDecl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(RangedDecl {
            name: input.parse()?,
            in_token: input.parse()?,
            start: input.parse()?,
            range_token: input.parse()?,
            end: input.parse()?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ForDecls {
    for_token: Token![for],
    decls: Punctuated<RangedDecl, Token![,]>,
}

impl Parse for ForDecls {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(ForDecls {
            for_token: input.parse()?,
            decls: Punctuated::parse_separated_nonempty(input)?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WhereClause {
    where_token: Token![where],
    filter: syn::Expr,
}

impl Parse for WhereClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(WhereClause {
            where_token: input.parse()?,
            filter: input.parse()?,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ConstImplInput {
    for_decls: ForDecls,
    where_clause: Option<WhereClause>,
    item: syn::Item,
}

impl Parse for ConstImplInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let for_decls: ForDecls = input.parse()?;
        let where_clause: Option<WhereClause> = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };
        let item: syn::Item = input.parse()?;
        Ok(ConstImplInput {
            for_decls,
            where_clause,
            item,
        })
    }
}

struct VariableState {
    name: Ident,
    start: i64,
    end: i64,
    value: i64,
}

struct VariablesState {
    variables: Vec<VariableState>,
    finished: bool,
}

impl FromIterator<(Ident, i64, i64)> for VariablesState {
    fn from_iter<T: IntoIterator<Item = (Ident, i64, i64)>>(iter: T) -> Self {
        let variables: Vec<VariableState> = iter
            .into_iter()
            .map(|(name, start, end)| VariableState {
                name,
                start,
                end,
                value: start,
            })
            .collect();
        VariablesState {
            variables,
            finished: false,
        }
    }
}

impl Iterator for VariablesState {
    type Item = Vec<(Ident, i64)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.variables.is_empty() {
            return None;
        }

        // Return current state
        let current = self
            .variables
            .iter()
            .map(|v| (v.name.clone(), v.value))
            .collect();

        // Advance to next combination (like odometer)
        for i in (0..self.variables.len()).rev() {
            let var = &mut self.variables[i];
            if var.value < var.end - 1 {
                // Note: end is exclusive
                var.value += 1;
                // Reset all variables to the right back to their start values
                for j in i + 1..self.variables.len() {
                    self.variables[j].value = self.variables[j].start;
                }
                return Some(current);
            }
        }

        // If we get here, all variables are at their maximum values
        self.finished = true;
        Some(current)
    }
}

struct ConstImplGenerator {
    variables: Vec<(Ident, i64, i64)>,
    where_clause: Option<String>,
    item: syn::Item,
}

impl TryFrom<ConstImplInput> for ConstImplGenerator {
    type Error = syn::Error;

    fn try_from(input: ConstImplInput) -> Result<Self, Self::Error> {
        let mut variables = Vec::new();
        for decl in input.for_decls.decls {
            let start = decl.start.base10_parse::<i64>()?;
            let end = decl.end.base10_parse::<i64>()?;
            if start >= end {
                return Err(syn::Error::new(
                    decl.start.span(),
                    "Start of range must be less than end",
                ));
            }
            variables.push((decl.name, start, end));
        }
        let where_clause = input.where_clause.map(|wc| {
            let filter = &wc.filter;
            quote::quote!(#filter).to_string()
        });
        Ok(ConstImplGenerator {
            variables,
            where_clause,
            item: input.item,
        })
    }
}

struct Substitutor<'a> {
    values: &'a Vec<(Ident, i64)>,
}

impl VisitMut for Substitutor<'_> {
    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        let syn::Expr::Path(path_expr) = i else {
            return syn::visit_mut::visit_expr_mut(self, i);
        };
        for (name, value) in self.values {
            if path_expr.path.is_ident(name) {
                let value = syn::Index::from(*value as usize);
                *i = syn::parse_quote! { #value };
                return;
            }
        }
        return syn::visit_mut::visit_expr_mut(self, i);
    }
}

impl ToTokens for ConstImplGenerator {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let states = self.variables.iter().cloned().collect::<VariablesState>();
        // Iterate over all combinations
        for combination in states {
            // Create a context for evaluating where clauses
            let mut context = evalexpr::HashMapContext::new();
            for (name, value) in &combination {
                context
                    .set_value(name.to_string(), evalexpr::Value::Int(*value))
                    .unwrap();
            }
            // Check where clauses
            if let Some(where_clause) = &self.where_clause {
                let result = evalexpr::eval_boolean_with_context(where_clause, &context)
                    .expect("Failed to evaluate where clause");
                if !result {
                    continue;
                }
            }
            let mut substitutor = Substitutor {
                values: &combination,
            };
            let mut body = self.item.clone();
            substitutor.visit_item_mut(&mut body);
            tokens.extend(quote::quote!(#body));
        }
    }
}

pub fn const_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<ConstImplInput>(input)?;
    let generator = ConstImplGenerator::try_from(input)?;
    Ok(generator.to_token_stream())
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;

    #[test]
    fn test_cartesian_product_iterator() {
        // Create some test variables
        let variables = [
            (syn::parse_str("N").unwrap(), 1, 4),
            (syn::parse_str("M").unwrap(), 2, 4),
        ];

        let iterator = variables.into_iter().collect::<VariablesState>();
        let results: Vec<_> = iterator.collect();

        // Should generate: (N=1,M=2), (N=1,M=3), (N=2,M=2), (N=2,M=3), (N=3,M=2), (N=3,M=3)
        assert_eq!(results.len(), 6);

        // Check first combination
        assert_eq!(results[0][0].1, 1); // N = 1
        assert_eq!(results[0][1].1, 2); // M = 2

        // Check last combination
        assert_eq!(results[5][0].1, 3); // N = 3
        assert_eq!(results[5][1].1, 3); // M = 3
    }

    #[test]
    fn test_parse_const_impl() {
        let input = quote! {
            for N in 1..5, M in 1..8 where N + M < 10, N - M > 0
                fn add() {
                    println("N = {}, M = {}", N, M);
                }
        };
        let parsed: ConstImplInput = syn::parse2(input).expect("Failed to parse");
        let generator: ConstImplGenerator = parsed.try_into().expect("Failed to convert");
        let ts = generator.to_token_stream();
        let pretty = prettyplease::unparse(&syn::parse2::<syn::File>(ts).unwrap());
        let expect = expect_test::expect![[r#"
            fn add() {
                println("N = {}, M = {}", 2, 1);
            }
            fn add() {
                println("N = {}, M = {}", 3, 1);
            }
            fn add() {
                println("N = {}, M = {}", 3, 2);
            }
            fn add() {
                println("N = {}, M = {}", 4, 1);
            }
            fn add() {
                println("N = {}, M = {}", 4, 2);
            }
            fn add() {
                println("N = {}, M = {}", 4, 3);
            }
        "#]];
        expect.assert_eq(&pretty);
    }
}
