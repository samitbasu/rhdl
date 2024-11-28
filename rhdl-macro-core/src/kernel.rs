use std::collections::HashSet;

use inflections::Inflect;
use quote::{format_ident, quote};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, FnArg, Ident, Pat, PatType, Path,
    ReturnType, Type,
};

// use crate::suffix::CustomSuffix;
type TS = proc_macro2::TokenStream;
type Result<T> = syn::Result<T>;
// use syn::visit_mut::VisitMut;

// We need the same kind of scope tracking that is used in `infer_types.rs`.
// Basically, in any given scope, we need a list of the bindings that have
// been defined thus far.  That way, if we encounter a path that is not
// defined in the current scope, we can import it from the parent scope of the
// function.

#[derive(Copy, Clone, Debug, PartialEq)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

#[derive(Default)]
struct Scope {
    bindings: HashSet<Ident>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

pub struct Context {
    scopes: Vec<Scope>,
    active_scope: ScopeId,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            scopes: vec![Default::default()],
            active_scope: Default::default(),
        }
    }
}

fn ident_starts_with_capital_letter(i: &syn::Ident) -> bool {
    i.to_string()
        .chars()
        .next()
        .map(|x| x.is_uppercase())
        .unwrap_or(false)
}

fn pattern_has_bindings(pat: &syn::Pat) -> bool {
    match pat {
        Pat::Ident(x) => x.ident.to_string().is_snake_case(),
        Pat::Or(subs) => subs.cases.iter().any(pattern_has_bindings),
        Pat::Paren(pat) => pattern_has_bindings(&pat.pat),
        Pat::Slice(slice) => slice.elems.iter().any(pattern_has_bindings),
        Pat::Tuple(tuple) => tuple.elems.iter().any(pattern_has_bindings),
        Pat::Struct(struct_) => struct_.fields.iter().any(|x| pattern_has_bindings(&x.pat)),
        Pat::TupleStruct(tuple) => tuple.elems.iter().any(pattern_has_bindings),
        Pat::Type(ty) => pattern_has_bindings(&ty.pat),
        _ => false,
    }
}

//
// This is a kludge.  I do not know of any way to determine if
// an expression like j = Foo::Bar(3) is a function named Bar
// in the namespace of Foo, or if it is a variant tuple constructor
// for the variant Bar of the enum Foo.  In the later case, I will
// not be able to call <Foo as DigitalFn>::kernel_fn() to get the
// function details, since Foo is not a type.  As a result, I resort
// to the following hack.
//
// If the path is of the form Foo::Bar, and Foo and Bar are both
// capitalized, then we assume that it is a tuple struct variant
// in an Enum.  This could fail.  But I have no solution for that
// at the moment.
//
fn path_is_enum_tuple_struct_by_convention(path: &Path) -> bool {
    if path.segments.len() != 2 {
        return false;
    }
    let last = &path.segments[1];
    if last.ident == "UPDATE" {
        return false;
    }
    let second_to_last = &path.segments[0];
    ident_starts_with_capital_letter(&last.ident)
        && ident_starts_with_capital_letter(&second_to_last.ident)
}

fn split_path_into_base_and_variant(path: &Path) -> Result<(Path, Ident)> {
    let base = path
        .segments
        .iter()
        .take(path.segments.len() - 1)
        .cloned()
        .collect();
    let variant = path
        .segments
        .last()
        .ok_or(syn::Error::new(
            path.span(),
            "Empty path in rhdl kernel function",
        ))?
        .ident
        .clone();
    Ok((
        Path {
            leading_colon: None,
            segments: base,
        },
        variant,
    ))
}

fn rewrite_pattern_as_typed_bits(pat: &syn::Pat) -> syn::Result<TS> {
    match pat {
        Pat::Lit(lit) => match &lit.lit {
            syn::Lit::Bool(b) => Ok(quote! {bob.expr_lit_bool(#b)}),
            syn::Lit::Int(i) => Ok(quote! {bob.expr_lit_int(stringify!(#i))}),
            _ => Err(syn::Error::new(
                lit.span(),
                "Unsupported literal in rhdl kernel function",
            )),
        },
        _ => Ok(
            quote! { bob.expr_lit_typed_bits(rhdl::core::Digital::typed_bits(#pat), stringify!(#pat)) },
        ),
    }
}

fn rewrite_pattern_to_use_defaults_for_bindings(pat: &syn::Pat) -> TS {
    match pat {
        Pat::Ident(_) => {
            quote! { Default::default() }
        }
        Pat::Or(subs) => {
            let cases = subs
                .cases
                .iter()
                .map(rewrite_pattern_to_use_defaults_for_bindings);
            quote! { #(#cases) |* }
        }
        Pat::Paren(pat) => rewrite_pattern_to_use_defaults_for_bindings(&pat.pat),
        Pat::Slice(slice) => {
            let elems = slice
                .elems
                .iter()
                .map(rewrite_pattern_to_use_defaults_for_bindings);
            quote! { [#(#elems),*] }
        }
        Pat::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(rewrite_pattern_to_use_defaults_for_bindings);
            quote! { (#(#elems),*) }
        }
        Pat::Struct(struct_) => {
            let path = &struct_.path;
            let fields = struct_.fields.iter().map(|x| {
                let field_name = &x.member;
                let field_pat = rewrite_pattern_to_use_defaults_for_bindings(&x.pat);
                quote!( #field_name: #field_pat)
            });
            quote! { #path {#(#fields),*} }
        }
        Pat::TupleStruct(tuple) => {
            let path = &tuple.path;
            let elems = tuple
                .elems
                .iter()
                .map(rewrite_pattern_to_use_defaults_for_bindings);
            quote! { #path (#(#elems),*) }
        }
        Pat::Type(ty) => rewrite_pattern_to_use_defaults_for_bindings(&ty.pat),
        Pat::Wild(_) => quote! { Default::default() },
        _ => quote! { #pat },
    }
}

fn pat_is_none(pat: &syn::Pat) -> bool {
    if let syn::Pat::Ident(ident) = pat {
        ident.ident == "None"
    } else {
        false
    }
}

impl Context {
    fn new_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            bindings: HashSet::new(),
            children: Vec::new(),
            parent: self.active_scope,
        });
        self.scopes[self.active_scope.0].children.push(id);
        self.active_scope = id;
        id
    }
    fn end_scope(&mut self) {
        self.active_scope = self.scopes[self.active_scope.0].parent;
    }
    fn is_scoped_binding(&self, path: &Path) -> bool {
        if path.segments.len() != 1 {
            return false;
        }
        let ident = &path.segments[0].ident;
        let mut scope = self.active_scope;
        loop {
            if self.scopes[scope.0].bindings.contains(ident) {
                return true;
            }
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        false
    }
    fn add_scoped_binding(&mut self, pat: &Pat) -> Result<()> {
        match pat {
            Pat::Ident(ident) => {
                let name = &ident.ident;
                if ident.by_ref.is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Unsupported reference pattern in rhdl kernel function",
                    ));
                }
                self.scopes[self.active_scope.0]
                    .bindings
                    .insert(name.clone());
            }
            Pat::Tuple(tuple) => {
                for pat in tuple.elems.iter() {
                    self.add_scoped_binding(pat)?;
                }
            }
            Pat::Slice(slice) => {
                for pat in slice.elems.iter() {
                    self.add_scoped_binding(pat)?;
                }
            }
            Pat::Struct(structure) => {
                for field in structure.fields.iter() {
                    self.add_scoped_binding(&field.pat)?;
                }
            }
            Pat::TupleStruct(tuple) => {
                for pat in tuple.elems.iter() {
                    self.add_scoped_binding(pat)?;
                }
            }
            Pat::Or(or) => {
                for pat in or.cases.iter() {
                    self.add_scoped_binding(pat)?;
                }
            }
            Pat::Type(pat) => self.add_scoped_binding(&pat.pat)?,
            Pat::Wild(_) | Pat::Path(_) | Pat::Const(_) | Pat::Lit(_) => {}
            _ => {
                return Err(syn::Error::new(
                    pat.span(),
                    format!(
                        "Unsupported pattern {} in rhdl kernel function",
                        quote!(#pat)
                    ),
                ));
            }
        }
        Ok(())
    }
}

pub fn hdl_kernel(input: TS) -> Result<TS> {
    let input = syn::parse::<syn::ItemFn>(input.into())?;
    let mut context = Context::default();
    context.function(input)
}

// Convert a pattern that would appear in a function argument into an expression.
// Only supports idents and tuples of idents.
fn pattern_to_expr(pat: &syn::Pat) -> Result<TS> {
    match pat {
        syn::Pat::Ident(ident) => {
            let name = &ident.ident;
            Ok(quote! {#name})
        }
        syn::Pat::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(pattern_to_expr)
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {(#(#elems ,)*)})
        }
        _ => Err(syn::Error::new(
            pat.span(),
            "Unsupported pattern in rhdl kernel function",
        )),
    }
}

fn strip_mutability(pat: &syn::Pat) -> syn::Pat {
    let pat = pat.clone();
    match pat {
        syn::Pat::Ident(mut ident) => {
            ident.mutability = None;
            syn::Pat::Ident(ident)
        }
        syn::Pat::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(strip_mutability)
                .collect::<Punctuated<_, Comma>>();
            syn::Pat::Tuple(syn::PatTuple {
                attrs: tuple.attrs.clone(),
                paren_token: tuple.paren_token,
                elems,
            })
        }
        x => x,
    }
}

fn impl_digital_fnk_trait(function: &syn::ItemFn) -> Result<TS> {
    let orig_name = &function.sig.ident;
    let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();
    let args = &function.sig.inputs;
    let outer_args = args
        .iter()
        .cloned()
        .map(|x| match x {
            FnArg::Typed(PatType {
                attrs,
                pat,
                colon_token,
                ty,
            }) => FnArg::Typed(PatType {
                attrs,
                pat: Box::new(strip_mutability(&pat)),
                colon_token,
                ty,
            }),
            _ => x,
        })
        .collect::<Punctuated<_, Comma>>();
    let type_sig = args
        .iter()
        .map(|x| match x {
            FnArg::Typed(PatType { ty, .. }) => Ok(quote! {
                #ty
            }),
            x => Err(syn::Error::new(
                x.span(),
                "Unsupported argument type in rhdl kernel function",
            )),
        })
        .collect::<Result<Punctuated<_, Comma>>>()?;
    let arg_count = outer_args.len();
    let trait_name = format_ident!("DigitalFn{}", arg_count);
    let ret = &function.sig.output;
    let arg_maps = args
        .iter()
        .filter_map(|x| match x {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat) => Some(&pat.ty),
        })
        .enumerate()
        .map(|(ndx, arg)| {
            let ty_name = format_ident!("A{ndx}");
            quote! {
                type #ty_name = #arg;
            }
        });
    let ret_ty = match &ret {
        ReturnType::Default => quote! {()},
        ReturnType::Type(_, ty) => quote! {#ty},
    };
    let output_ty = quote! {
        type O = #ret_ty;
    };
    let ty_generics_as_turbo = ty_generics.as_turbofish();
    Ok(quote! {
        impl #impl_generics rhdl::core::digital_fn::#trait_name for #orig_name #ty_generics #where_clause {
            #(#arg_maps)*
            #output_ty
            fn func() -> fn(#type_sig) #ret {
                #orig_name #ty_generics_as_turbo
            }
        }
    })
}

// put the original function inside a wrapper function with the name of the original.
// Call the wrapped function 'inner'
// Capture the return of the wrapped function
// Call trace_path_push("function_name") before calling the wrapped function
// call trace_path_pop() after calling the wrapped function
// So it should go from this:
//
//  fn my_func(args) -> Ret {
//      body
//  }
//
// to
//
//  fn my_func(args) -> Ret {
//      fn inner(args) -> Ret { body }
//      note_push("my_func");
//      let ret = inner(args);
//      note_pop();
//      ret
//  }
//
fn trace_wrap_function(function: &syn::ItemFn) -> Result<TS> {
    let vis = &function.vis;
    let orig_name = &function.sig.ident;
    let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();
    let args = &function.sig.inputs;
    let outer_args = args
        .iter()
        .cloned()
        .map(|x| match x {
            FnArg::Typed(PatType {
                attrs,
                pat,
                colon_token,
                ty,
            }) => FnArg::Typed(PatType {
                attrs,
                pat: Box::new(strip_mutability(&pat)),
                colon_token,
                ty,
            }),
            _ => x,
        })
        .collect::<Punctuated<_, Comma>>();
    let call_args = args
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => Err(syn::Error::new(
                arg.span(),
                "Unsupported receiver in rhdl kernel function",
            )),
            syn::FnArg::Typed(pat) => {
                let pat = &pat.pat;
                pattern_to_expr(pat)
            }
        })
        .collect::<Result<Punctuated<_, Comma>>>()?;
    let ret = &function.sig.output;
    let body = function.block.clone();
    // CustomSuffix.visit_block_mut(&mut body);
    let ty_generics = ty_generics.as_turbofish();
    Ok(quote! {

        #vis fn #orig_name #impl_generics (#outer_args) #ret #where_clause {
            #[forbid(non_snake_case)]
            #[forbid(non_upper_case_globals)]
            #[forbid(unreachable_patterns)]
            //#[forbid(path_statements)]
            //#[forbid(unused_variables)]
            fn inner #impl_generics (#args) #ret #where_clause {
                #body
            }
            rhdl::core::trace_push_path(stringify!(#orig_name));
            let ret = inner #ty_generics (#call_args);
            rhdl::core::trace_pop_path();
            ret
        }
    })
}

impl Context {
    fn function(&mut self, function: syn::ItemFn) -> Result<TS> {
        let orig_name = &function.sig.ident;
        let vis = &function.vis;
        let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();
        let phantom_fields = function
            .sig
            .generics
            .params
            .iter()
            .enumerate()
            .filter_map(|(ndx, param)| {
                let ident = format_ident!("__phantom_{}", ndx);
                if let syn::GenericParam::Type(ty) = param {
                    let ty_name = &ty.ident;
                    Some(quote! {#ident: std::marker::PhantomData<#ty_name>})
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let name = &function.sig.ident;
        // Put the function arguments into the current scope
        for arg in function.sig.inputs.iter() {
            match arg {
                syn::FnArg::Receiver(_) => {
                    return Err(syn::Error::new(
                        arg.span(),
                        "Unsupported receiver in rhdl kernel function",
                    ))
                }
                syn::FnArg::Typed(pat) => {
                    self.add_scoped_binding(&pat.pat)?;
                }
            }
        }
        let block = self.block_inner(&function.block)?;
        let ret = match &function.sig.output {
            syn::ReturnType::Default => quote! {rhdl::core::Kind::Empty},
            syn::ReturnType::Type(_, ty) => {
                quote! {
                    <#ty as rhdl::core::Digital>::static_kind()
                }
            }
        };
        let args = &function
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Receiver(_) => Err(syn::Error::new(
                    arg.span(),
                    "Unsupported receiver in rhdl kernel function",
                )),
                syn::FnArg::Typed(pat) => {
                    let ty = &pat.ty;
                    let pat = self.pat(&pat.pat)?;
                    let kind = quote! {<#ty as rhdl::core::Digital>::static_kind()};
                    Ok(quote! { bob.type_pat(#pat, #kind)})
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let wrapped_function = trace_wrap_function(&function)?;
        let digital_fnk_impl = impl_digital_fnk_trait(&function)?;
        Ok(quote! {
            #wrapped_function

            #[allow(non_camel_case_types)]
            #vis struct #name #impl_generics {#(#phantom_fields,)*}

            #digital_fnk_impl

            impl #impl_generics rhdl::core::digital_fn::DigitalFn for #name #ty_generics #where_clause {
                fn kernel_fn() -> Option<rhdl::core::digital_fn::KernelFnKind> {
                    let bob = rhdl::core::ast_builder::ASTBuilder::default();
                    Some(bob.kernel_fn(
                        stringify!(#orig_name),
                        vec!{#(#args),*},
                        #ret,
                        #block,
                        std::any::TypeId::of::<#name #ty_generics>(),
                    ))
                }
            }
        })
    }

    fn block(&mut self, block: &syn::Block) -> Result<TS> {
        self.new_scope();
        let block = self.block_inner(block)?;
        self.end_scope();
        Ok(quote! {
            bob.block_expr(#block)
        })
    }

    fn block_inner(&mut self, block: &syn::Block) -> Result<TS> {
        let stmts = block
            .stmts
            .iter()
            .map(|x| self.stmt(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            bob.block(vec![#(#stmts),*],)
        })
    }

    fn stmt(&mut self, statement: &syn::Stmt) -> Result<TS> {
        match statement {
            syn::Stmt::Local(local) => self.stmt_local(local),
            syn::Stmt::Expr(expr, semi) => {
                let expr = self.expr(expr)?;
                if semi.is_some() {
                    Ok(quote! {
                        bob.semi_stmt(#expr)
                    })
                } else {
                    Ok(quote! {
                        bob.expr_stmt(#expr)
                    })
                }
            }
            _ => Err(syn::Error::new(
                statement.span(),
                "Unsupported statement type",
            )),
        }
    }

    fn stmt_local(&mut self, local: &syn::Local) -> Result<TS> {
        let pattern = self.pat(&local.pat)?;
        self.add_scoped_binding(&local.pat)?;
        let local_init = local
            .init
            .as_ref()
            .map(|x| self.expr(&x.expr))
            .transpose()?
            .map(|x| quote!(Some(#x)))
            .unwrap_or(quote! {None});
        Ok(quote! {
                bob.local_stmt(#pattern, #local_init)
        })
    }

    // TBD - we need to distinguish between something like:
    //  Foo::Bar(3)
    // and
    //  Foo::Bar(a)
    // Unfortunately, since `a` may be a constant at the outer scope,
    // this is not immediately obvious.

    // Use for patterns that are in let contexts.
    // These are infallible.
    fn pat(&mut self, pat: &syn::Pat) -> Result<TS> {
        match pat {
            syn::Pat::Ident(ident) => {
                let name = &ident.ident;
                let mutability = ident.mutability.is_some();
                if ident.by_ref.is_some() {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Unsupported reference pattern",
                    ));
                }
                Ok(quote! {
                    bob.ident_pat(stringify!(#name),#mutability)
                })
            }
            syn::Pat::TupleStruct(tuple) => {
                let path = self.path_inner(&tuple.path)?;
                let elems = tuple
                    .elems
                    .iter()
                    .map(|x| self.pat(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok(quote! {
                    bob.tuple_struct_pat(#path, vec![#(#elems),*])
                })
            }
            syn::Pat::Tuple(tuple) => {
                let elems = tuple
                    .elems
                    .iter()
                    .map(|x| self.pat(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok(quote! {
                    bob.tuple_pat(vec![#(#elems),*])
                })
            }
            syn::Pat::Slice(slice) => {
                let elems = slice
                    .elems
                    .iter()
                    .map(|x| self.pat(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok(quote! {
                    bob.slice_pat(vec![#(#elems),*])
                })
            }
            syn::Pat::Path(path) => {
                let path = self.path_inner(&path.path)?;
                Ok(quote! {
                    bob.path_pat(#path)
                })
            }
            syn::Pat::Struct(structure) => {
                let path = self.path_inner(&structure.path)?;
                let fields = structure
                    .fields
                    .iter()
                    .map(|x| self.field_pat(x))
                    .collect::<Result<Vec<_>>>()?;
                if structure.qself.is_some() {
                    return Err(syn::Error::new(
                        structure.span(),
                        "Unsupported qualified self in rhdl kernel function",
                    ));
                }
                let rest = structure.rest.is_some();
                Ok(quote! {
                    bob.struct_pat(#path, vec![#(#fields),*], #rest)
                })
            }
            syn::Pat::Type(pat) => {
                let ty = &pat.ty;
                let pat = self.pat(&pat.pat)?;
                let kind = quote! {<#ty as rhdl::core::Digital>::static_kind()};
                Ok(quote! {
                    bob.type_pat(#pat, #kind)
                })
            }
            syn::Pat::Lit(pat) => {
                let lit = self.lit_inner(pat)?;
                Ok(quote! {
                    bob.lit_pat(#lit)
                })
            }
            syn::Pat::Wild(_) => Ok(quote! {
                bob.wild_pat()
            }),
            _ => Err(syn::Error::new(pat.span(), "Unsupported pattern type")),
        }
    }

    fn field_pat(&mut self, expr: &syn::FieldPat) -> Result<TS> {
        let member = self.member(&expr.member)?;
        let pat = self.pat(&expr.pat)?;
        Ok(quote! {
            bob.field_pat(#member, #pat)
        })
    }

    fn expr(&mut self, expr: &syn::Expr) -> Result<TS> {
        match expr {
            syn::Expr::Lit(expr) => self.lit(expr),
            syn::Expr::Binary(expr) => self.binary(expr),
            syn::Expr::Unary(expr) => self.unary(expr),
            syn::Expr::Group(expr) => self.group(expr),
            syn::Expr::Paren(expr) => self.paren(expr),
            syn::Expr::Assign(expr) => self.assign(expr),
            syn::Expr::Path(expr) => self.path(&expr.path),
            syn::Expr::Struct(expr) => self.struct_ex(expr),
            syn::Expr::Block(expr) => self.block(&expr.block),
            syn::Expr::Field(expr) => self.field_expression(expr),
            syn::Expr::If(expr) => self.if_ex(expr),
            syn::Expr::Let(expr) => self.let_ex(expr),
            syn::Expr::Match(expr) => self.match_ex(expr),
            syn::Expr::Range(expr) => self.range(expr),
            syn::Expr::Try(expr) => self.try_ex(expr),
            syn::Expr::Return(expr) => self.ret(expr),
            syn::Expr::Tuple(expr) => self.tuple(expr),
            syn::Expr::Repeat(expr) => self.repeat(expr),
            syn::Expr::ForLoop(expr) => self.for_loop(expr),
            syn::Expr::While(expr) => self.while_loop(expr),
            syn::Expr::Call(expr) => self.call(expr),
            syn::Expr::Array(expr) => self.array(expr),
            syn::Expr::Index(expr) => self.index(expr),
            syn::Expr::MethodCall(expr) => self.method_call(expr),
            _ => Err(syn::Error::new(
                expr.span(),
                format!(
                    "Unsupported expression type {} in an rhdl kernel function",
                    quote!(#expr)
                ),
            )),
        }
    }

    fn method_call(&mut self, expr: &syn::ExprMethodCall) -> Result<TS> {
        let receiver = self.expr(&expr.receiver)?;
        let args = expr
            .args
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        let method = &expr.method;
        if ![
            "any",
            "all",
            "xor",
            "as_signed",
            "as_unsigned",
            "val",
            "resize",
        ]
        .contains(&method.to_string().as_str())
        {
            return Err(syn::Error::new(
                expr.span(),
                format!(
                    "Unsupported method call {} in an rhdl kernel function",
                    quote!(#expr)
                ),
            ));
        }
        let turbo = if let Some(x) = &expr.turbofish {
            if (x.args.len() != 1) || (method != "resize") {
                return Err(syn::Error::new(
                    x.span(),
                    "Unsupported turbofish in rhdl kernel function - only resize::<N>() is supported",
                ));
            }
            let x = x.args.iter().next().unwrap();
            quote!(Some({#x} as usize))
        } else {
            quote!(None)
        };
        Ok(quote! {
            bob.method_expr(#receiver, vec![#(#args),*], stringify!(#method), #turbo)
        })
    }

    fn index(&mut self, expr: &syn::ExprIndex) -> Result<TS> {
        let index = self.expr(&expr.index)?;
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.index_expr(#expr, #index)
        })
    }

    fn array(&mut self, expr: &syn::ExprArray) -> Result<TS> {
        let elems = expr
            .elems
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            bob.array_expr(vec![#(#elems),*])
        })
    }

    fn special_case_call(&mut self, expr: &syn::ExprCall) -> Result<Option<TS>> {
        let special_cases = [
            ("bits", quote!(bob.expr_bits)),
            ("signed", quote!(bob.expr_signed)),
            ("Ok", quote!(bob.expr_ok)),
            ("Err", quote!(bob.expr_err)),
            ("Some", quote!(bob.expr_some)),
            ("None", quote!(bob.expr_none)),
        ];
        let syn::Expr::Path(func_path) = expr.func.as_ref() else {
            return Err(syn::Error::new(
                expr.func.span(),
                "Unsupported function call in rhdl kernel function (only paths allowed here)",
            ));
        };
        let Some(name) = func_path.path.segments.last() else {
            return Ok(None);
        };
        if !name.arguments.is_empty() || expr.args.len() != 1 {
            return Ok(None);
        }
        if let Some((_, special_code)) = special_cases.iter().find(|(n, _)| name.ident == n) {
            let args = self.expr(&expr.args[0])?;
            return Ok(Some(quote! {
                #special_code(#args)
            }));
        }
        Ok(None)
    }

    // A call expression like `a = Foo(...)` can be either a variant
    // constructor, a tuple struct constructor or an actual function
    // call.  Use the `inspect_digital` function
    // to extract a call signature from the function.
    fn call(&mut self, expr: &syn::ExprCall) -> Result<TS> {
        let syn::Expr::Path(func_path) = expr.func.as_ref() else {
            return Err(syn::Error::new(
                expr.func.span(),
                "Unsupported function call in rhdl kernel function (only paths allowed here)",
            ));
        };

        if let Some(special_case) = self.special_case_call(expr)? {
            return Ok(special_case);
        }

        // There are 3 special cases not handled above
        // - note calls are removed since these do not generate any code
        // - default calls are replaced with the literal value of the argument using TypedBits and the Digital trait
        // - signal calls are replaced with a call to the appropriate form of the signal op
        if let Some(name) = func_path.path.segments.last() {
            if name.ident == "trace" {
                return Ok(quote! {
                    bob.block_expr(bob.block(vec![]))
                });
            } else if name.ident == "default" || name.ident == "maybe_init" {
                return Ok(quote! {
                    bob.lit_expr(
                        bob.expr_lit_typed_bits(
                            rhdl::core::Digital::typed_bits(#expr),
                            stringify!(#expr)
                        )
                    )
                });
            } else if name.ident == "signal" {
                let args = self.expr(&expr.args[0])?;
                if let syn::PathArguments::AngleBracketed(generics) = &name.arguments {
                    let Some(clock) = generics.args.iter().nth(1) else {
                        return Err(syn::Error::new(
                            expr.span(),
                            "Unsupported signal call in rhdl kernel function",
                        ));
                    };
                    let clock = quote!(<#clock as rhdl::core::Domain>::color());
                    return Ok(quote! {
                        bob.expr_signal(#args, Some(#clock))
                    });
                } else {
                    return Ok(quote! {
                        bob.expr_signal(#args, None)
                    });
                }
            }
        }
        let code = if !path_is_enum_tuple_struct_by_convention(&func_path.path) {
            // This is a function call
            self.get_code(&func_path.path)?
        } else {
            // This is an enum tuple struct... build one.
            // To do so, we split the path into the base and the variant, assuming that
            // the last segment is the variant.
            let base = func_path
                .path
                .segments
                .iter()
                .map(|x| quote!(#x))
                .take(func_path.path.segments.len() - 1)
                .collect::<Vec<_>>();
            let Some(variant) = func_path.path.segments.last() else {
                return Err(syn::Error::new(
                    func_path.path.span(),
                    "Empty path in rhdl kernel function",
                ));
            };
            let shell = quote! {
                <#(#base)::* as rhdl::core::Digital>::static_kind().enum_template(stringify!(#variant)).unwrap()
            };
            // This is a tuple struct constructor
            quote!(Some(rhdl::core::KernelFnKind::EnumTupleStructConstructor(#shell)))
        };
        let call_to_get_type = quote!(rhdl::core::digital_fn::inspect_digital(#func_path));
        let path = self.path_inner(&func_path.path)?;
        let args = expr
            .args
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            bob.call_expr(#path, vec![#(#args),*], #call_to_get_type, #code)
        })
    }

    fn get_code(&mut self, path: &Path) -> Result<TS> {
        if path.segments.is_empty() {
            return Err(syn::Error::new(
                path.span(),
                "Empty path in rhdl kernel function",
            ));
        }
        Ok(quote!(<#path as rhdl::core::digital_fn::DigitalFn>::kernel_fn()))
    }

    fn for_loop(&mut self, expr: &syn::ExprForLoop) -> Result<TS> {
        self.new_scope();
        let pat = self.pat(&expr.pat)?;
        self.add_scoped_binding(&expr.pat)?;
        let body = self.block_inner(&expr.body)?;
        let expr = self.expr(&expr.expr)?;
        self.end_scope();
        Ok(quote! {
            bob.for_expr(#pat, #expr, #body)
        })
    }

    fn while_loop(&mut self, expr: &syn::ExprWhile) -> Result<TS> {
        // In version 2.0...
        Err(syn::Error::new(
            expr.span(),
            "Unsupported while loop in rhdl kernel function",
        ))
    }

    fn repeat(&mut self, expr: &syn::ExprRepeat) -> Result<TS> {
        let expr_len = &expr.len;
        let len = quote!(<usize as rhdl::core::Digital>::typed_bits(#expr_len).as_i64().unwrap());
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.repeat_expr(#expr, #len)
        })
    }

    fn tuple(&mut self, expr: &syn::ExprTuple) -> Result<TS> {
        let elems = expr
            .elems
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            bob.tuple_expr(vec![#(#elems),*])
        })
    }

    fn group(&mut self, expr: &syn::ExprGroup) -> Result<TS> {
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.group_expr(#expr)
        })
    }

    fn paren(&mut self, expr: &syn::ExprParen) -> Result<TS> {
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.paren_expr(#expr)
        })
    }

    fn ret(&mut self, expr: &syn::ExprReturn) -> Result<TS> {
        let expr = expr
            .expr
            .as_ref()
            .map(|x| self.expr(x))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or_else(|| quote! {None});
        Ok(quote! {
            bob.return_expr(#expr)
        })
    }

    fn try_ex(&mut self, expr: &syn::ExprTry) -> Result<TS> {
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.expr_try(#expr)
        })
    }

    fn range(&mut self, expr: &syn::ExprRange) -> Result<TS> {
        let start = expr
            .start
            .as_ref()
            .map(|x| self.expr(x))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or_else(|| quote! {None});
        let end = expr
            .end
            .as_ref()
            .map(|x| self.expr(x))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or_else(|| quote! {None});
        let limits = match expr.limits {
            syn::RangeLimits::HalfOpen(_) => {
                quote!(bob.range_limits_half_open())
            }
            syn::RangeLimits::Closed(_) => quote!(bob.range_limits_closed()),
        };
        Ok(quote! {
            bob.range_expr(#start, #limits, #end)
        })
    }

    fn match_ex(&mut self, expr: &syn::ExprMatch) -> Result<TS> {
        let arms = expr
            .arms
            .iter()
            .map(|x| self.arm(x))
            .collect::<Result<Vec<_>>>()?;
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.match_expr(#expr, vec![#(#arms),*])
        })
    }

    fn arm(&mut self, arm: &syn::Arm) -> Result<TS> {
        self.new_scope();
        let pat = &arm.pat;
        let arm = if !pattern_has_bindings(pat) {
            let body = self.expr(&arm.body)?;
            if let syn::Pat::Wild(_) = &pat {
                quote! {bob.arm_wild(#body)}
            } else if pat_is_none(pat) {
                quote! {bob.arm_none(#body)}
            } else {
                let pat = rewrite_pattern_as_typed_bits(pat)?;
                quote! {bob.arm_constant(#pat, #body)}
            }
        } else {
            self.add_scoped_binding(pat)?;
            let body = self.expr(&arm.body)?;
            let mut discriminant = None;
            if let syn::Pat::Path(path) = pat {
                if path.path.is_ident("None") {
                    discriminant = Some(quote!(false.typed_bits()));
                }
            }
            if let syn::Pat::TupleStruct(path) = pat {
                if path.path.is_ident("Err") {
                    discriminant = Some(quote!(false.typed_bits()));
                } else if path.path.is_ident("Some") || path.path.is_ident("Ok") {
                    discriminant = Some(quote!(true.typed_bits()));
                }
            }
            if discriminant.is_none() {
                let pat_as_expr = rewrite_pattern_to_use_defaults_for_bindings(pat);
                discriminant = Some(quote!(rhdl::core::Digital::discriminant(#pat_as_expr)));
            }
            let inner = self.pat(pat)?;
            quote! {bob.arm_enum(#inner, #discriminant, #body)}
        };
        self.end_scope();
        Ok(arm)
    }

    fn let_ex(&mut self, expr: &syn::ExprLet) -> Result<TS> {
        let pattern = self.pat(&expr.pat)?;
        let value = self.expr(&expr.expr)?;
        Ok(quote! {
            bob.let_expr(#pattern, #value)
        })
    }

    fn if_ex(&mut self, expr: &syn::ExprIf) -> Result<TS> {
        let cond = self.expr(&expr.cond)?;
        let then = self.block_inner(&expr.then_branch)?;
        let else_ = expr
            .else_branch
            .as_ref()
            .map(|x| self.expr(&x.1))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or(quote! {None});
        Ok(quote! {
            bob.if_expr(#cond, #then, #else_)
        })
    }

    // Interesting detail here.  Struct has a slight asymmetry in the handling
    // of enums and structs that we take advantage of.  For structs, we can
    // have so-called Functional Record Update (FRU) syntax, in which the
    // struct definition includes interpolated (spread) values from another
    // struct.  This allows us to infer that an expression of the type:
    //
    //  Foo { a: 1, ..bar }
    //
    // means that `Foo` is a struct in this case, and we can get the Kind of
    // Foo by using the usual
    //
    //  <Foo as Digital>::static_kind()
    //
    // method.  However, in general, if we have something like
    //
    //  Foo {a: 1, b: 2},
    //
    // the type could be either an enum or a struct.  If it's an enum, we cannot
    // legally call <Foo as Digital>, since `Foo` is not a type.  However, we
    // can take advantage of the requirement that `Digital: Default`, and that
    // all fields of a `Digital` struct or `Digital` enum must also implement `Default`
    // to generate an instance of the thing at run time using
    //
    //  (Foo {a: Default::default(), b: Default::default()}).kind()
    //
    // In both cases, we want to include the Kind information into the AST at the
    // point the AST is generated.
    fn struct_ex(&mut self, structure: &syn::ExprStruct) -> Result<TS> {
        let path_inner = self.path_inner(&structure.path)?;
        let fields = structure
            .fields
            .iter()
            .map(|x| self.field_value(x))
            .collect::<Result<Vec<_>>>()?;
        if structure.qself.is_some() {
            return Err(syn::Error::new(
                structure.span(),
                "Unsupported qualified self in rhdl kernel function",
            ));
        }
        let rest = structure
            .rest
            .as_ref()
            .map(|x| self.expr(x))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or(quote! {None});
        if structure.rest.is_some() {
            // The presence of a rest means we know that path -> struct
            let path = &structure.path;
            let template = quote!(< #path as rhdl::core::Digital>::static_kind().place_holder());
            Ok(quote! {bob.struct_expr(#path_inner, vec![#(#fields),*], #rest, #template)})
        } else if path_is_enum_tuple_struct_by_convention(&structure.path) {
            // This is an enum struct construction (we assume) since it has
            // the form of Foo::Bar{}, and by convention in RHDL, this is an
            // enum variant, and not a struct.
            let (base, variant) = split_path_into_base_and_variant(&structure.path)?;
            // Not sure what to do about the unwrap here.
            let template = quote!(< #base as rhdl::core::Digital>::static_kind().enum_template(stringify!(#variant)).unwrap());
            Ok(quote! {
                bob.struct_expr(#path_inner, vec![#(#fields),*], #rest, #template)
            })
        } else {
            let path = &structure.path;
            let obj = quote!(<#path as rhdl::core::Digital>::static_kind().place_holder());
            Ok(quote! {
                bob.struct_expr(#path_inner, vec![#(#fields),*], #rest, #obj)
            })
        }
    }

    fn path(&mut self, path: &syn::Path) -> Result<TS> {
        if path.is_ident("None") {
            return Ok(quote! {
                bob.expr_none()
            });
        }
        // Check for a locally defined path
        let inner = self.path_inner(path)?;
        if !self.is_scoped_binding(path) {
            return Ok(quote! {
                bob.expr_typed_bits(#inner, rhdl::core::Digital::typed_bits(#path), stringify!(#path))
            });
        }
        Ok(quote! {
            bob.path_expr(#inner)
        })
    }

    fn path_inner(&mut self, path: &syn::Path) -> Result<TS> {
        let segments = path
            .segments
            .iter()
            .map(|x| self.path_segment(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            bob.path(vec![#(#segments),*],)
        })
    }

    fn path_segment(&mut self, segment: &syn::PathSegment) -> Result<TS> {
        let ident = &segment.ident;
        let args = self.path_arguments(&segment.arguments)?;
        Ok(quote! {
            bob.path_segment(stringify!(#ident), #args)
        })
    }

    fn path_arguments(&mut self, arguments: &syn::PathArguments) -> Result<TS> {
        // We only allow Const arguments.
        let args = match arguments {
            syn::PathArguments::None => quote! {bob.path_arguments_none()},
            syn::PathArguments::AngleBracketed(args) => {
                let args = args
                    .args
                    .iter()
                    .map(|x| self.generic_argument(x))
                    .collect::<Result<Vec<_>>>()?;
                quote! {vec![#(#args),*]}
            }
            _ => {
                return Err(syn::Error::new(
                    arguments.span(),
                    "Unsupported path arguments in rhdl kernel function",
                ))
            }
        };
        Ok(quote! {
            #args
        })
    }

    fn generic_argument(&mut self, argument: &syn::GenericArgument) -> Result<TS> {
        match argument {
            syn::GenericArgument::Const(expr) => Ok(quote! {
                stringify!(#expr)
            }),
            syn::GenericArgument::Type(Type::Path(path)) => Ok(quote! {
                stringify!(#path)
            }),
            _ => Err(syn::Error::new(
                argument.span(),
                "Unsupported generic argument in rhdl kernel function",
            )),
        }
    }

    fn assign(&mut self, assign: &syn::ExprAssign) -> Result<TS> {
        let left = self.expr(&assign.left)?;
        let right = self.expr(&assign.right)?;
        Ok(quote! {
            bob.assign_expr(#left, #right)
        })
    }

    fn field_expression(&mut self, field: &syn::ExprField) -> Result<TS> {
        let expr = self.expr(&field.base)?;
        let member = self.member(&field.member)?;
        Ok(quote! {
            bob.field_expr(#expr, #member)
        })
    }

    fn field_value(&mut self, field: &syn::FieldValue) -> Result<TS> {
        let member = self.member(&field.member)?;
        let value = self.expr(&field.expr)?;
        Ok(quote! {
            bob.field_value(#member, #value)
        })
    }

    fn member(&mut self, member: &syn::Member) -> Result<TS> {
        Ok(match member {
            syn::Member::Named(ident) => quote! {
                bob.member_named(stringify!(#ident))
            },
            syn::Member::Unnamed(index) => {
                let index = index.index;
                quote! {
                    bob.member_unnamed(#index)
                }
            }
        })
    }

    fn unary(&mut self, unary: &syn::ExprUnary) -> Result<TS> {
        let op = match unary.op {
            syn::UnOp::Neg(_) => quote!(rhdl::core::ast_builder::UnOp::Neg),
            syn::UnOp::Not(_) => quote!(rhdl::core::ast_builder::UnOp::Not),
            _ => {
                return Err(syn::Error::new(
                    unary.span(),
                    "Unsupported unary operator in rhdl kernel function",
                ))
            }
        };
        let expr = self.expr(&unary.expr)?;
        Ok(quote! {
            bob.unary_expr(#op, #expr)
        })
    }

    fn binary(&mut self, binary: &syn::ExprBinary) -> Result<TS> {
        let op = match binary.op {
            syn::BinOp::Add(_) => quote!(rhdl::core::ast_builder::BinOp::Add),
            syn::BinOp::Sub(_) => quote!(rhdl::core::ast_builder::BinOp::Sub),
            syn::BinOp::Mul(_) => quote!(rhdl::core::ast_builder::BinOp::Mul),
            syn::BinOp::And(_) => quote!(rhdl::core::ast_builder::BinOp::And),
            syn::BinOp::Or(_) => quote!(rhdl::core::ast_builder::BinOp::Or),
            syn::BinOp::BitXor(_) => quote!(rhdl::core::ast_builder::BinOp::BitXor),
            syn::BinOp::BitAnd(_) => quote!(rhdl::core::ast_builder::BinOp::BitAnd),
            syn::BinOp::BitOr(_) => quote!(rhdl::core::ast_builder::BinOp::BitOr),
            syn::BinOp::Shl(_) => quote!(rhdl::core::ast_builder::BinOp::Shl),
            syn::BinOp::Shr(_) => quote!(rhdl::core::ast_builder::BinOp::Shr),
            syn::BinOp::Eq(_) => quote!(rhdl::core::ast_builder::BinOp::Eq),
            syn::BinOp::Lt(_) => quote!(rhdl::core::ast_builder::BinOp::Lt),
            syn::BinOp::Le(_) => quote!(rhdl::core::ast_builder::BinOp::Le),
            syn::BinOp::Ne(_) => quote!(rhdl::core::ast_builder::BinOp::Ne),
            syn::BinOp::Ge(_) => quote!(rhdl::core::ast_builder::BinOp::Ge),
            syn::BinOp::Gt(_) => quote!(rhdl::core::ast_builder::BinOp::Gt),
            syn::BinOp::AddAssign(_) => quote!(rhdl::core::ast_builder::BinOp::AddAssign),
            syn::BinOp::SubAssign(_) => quote!(rhdl::core::ast_builder::BinOp::SubAssign),
            syn::BinOp::MulAssign(_) => quote!(rhdl::core::ast_builder::BinOp::MulAssign),
            syn::BinOp::BitXorAssign(_) => quote!(rhdl::core::ast_builder::BinOp::BitXorAssign),
            syn::BinOp::BitAndAssign(_) => quote!(rhdl::core::ast_builder::BinOp::BitAndAssign),
            syn::BinOp::BitOrAssign(_) => quote!(rhdl::core::ast_builder::BinOp::BitOrAssign),
            syn::BinOp::ShlAssign(_) => quote!(rhdl::core::ast_builder::BinOp::ShlAssign),
            syn::BinOp::ShrAssign(_) => quote!(rhdl::core::ast_builder::BinOp::ShrAssign),
            _ => {
                return Err(syn::Error::new(
                    binary.span(),
                    "Unsupported binary operator in rhdl kernel function",
                ))
            }
        };
        let left = self.expr(&binary.left)?;
        let right = self.expr(&binary.right)?;
        Ok(quote! {
                bob.binary_expr(#op, #left, #right)
        })
    }

    fn lit(&mut self, lit: &syn::ExprLit) -> Result<TS> {
        let inner = self.lit_inner(lit)?;
        Ok(quote! {
            bob.lit_expr(#inner)
        })
    }

    fn lit_inner(&mut self, lit: &syn::ExprLit) -> Result<TS> {
        let lit = &lit.lit;
        match lit {
            syn::Lit::Int(int) => {
                let value = int.token();
                Ok(quote! {
                        bob.expr_lit_int(stringify!(#value))
                })
            }
            syn::Lit::Bool(boolean) => {
                let value = boolean.value;
                Ok(quote! {
                        bob.expr_lit_bool(#value)
                })
            }
            _ => Err(syn::Error::new(
                lit.span(),
                "Unsupported literal type in rhdl kernel function",
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generic_kernel() {
        let test_code = quote! {
            fn update<T: Digital>(a: T, b: T) -> [T; 2] {
                [a, b]
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_basic_block() {
        let test_code = quote! {
            {
                let a = 1;
                let b = 2;
                let q = 0x1234_u32;
                let c = a + b;
                let mut d = 3;
                let g = Foo {r: 1, g: 120, b: 33};
                let h = g.r;
                c
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = Context::default().block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl::core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
    #[test]
    fn test_precedence_parser() {
        let test_code = quote! {
            {
                1 + 3 * 9
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = Context::default().block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl::core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_struct_expression_let() {
        let test_code = quote! {
            let d = Foo::<T> {a: 1, b: 2};
        };
        let local = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = Context::default().stmt(&local).unwrap();
        eprintln!("{}", result);
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_struct_expression_let_with_spread() {
        let test_code = quote! {
            {
                let b = 3;
                let d = Foo::<T> {a: 1, ..b};
            }
        };
        let local = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = Context::default().block(&local).unwrap();
        eprintln!("{}", result);
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_if_expression() {
        let test_code = quote! {
            {
                let d = 4;
                if d > 0 {
                    d = d - 1;
                }
            }
        };
        let if_expr = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = Context::default().block(&if_expr).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rhdl::core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_syn_match() {
        let test_code = quote! {
            match l {
                State::Init => {}
                State::Run(a) => {}
                State::Boom => {}
                State::NotOk(3) => {}
            }
        };
        let match_expr = syn::parse2::<syn::Stmt>(test_code).unwrap();
        println!("{:#?}", match_expr);
    }

    #[test]
    fn test_match_expression() {
        let test_code = quote! {
            {
                let l = 3;
                match l {
                    State::Init => {}
                    State::Run(a) => {
                        l = a;
                    }
                    State::Boom => {}
                }
            }
        };
        let match_expr = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = Context::default().block(&match_expr).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        //        let result = result.replace("rhdl::core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_self_update() {
        let test_code = quote! {
            let (a, b, c) = (3, 4, 5);
        };
        let assign = syn::parse2::<syn::Stmt>(test_code).unwrap();
        let result = Context::default().stmt(&assign).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        //        let result = result.replace("rhdl::core :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_suffix() {
        let test_code = quote! {
            fn update() {
                let b = 0x4313_u8;
                let j = 342;
                let i = 0x432_u8;
                let a = 54_234_i14;
                let p = 0o644_u12;
                let z = 2_u4;
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
                } else {
                    z = 5;
                }
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_match_arm_pattern() {
        let test_code = quote! {
            fn update(z: u8) {
                match z {
                    1_u4 => {},
                    2_u4 => {},
                    CONST_VAL => {},
                }
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        println!("{:?}", function);
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_match_arm_pattern_rewrite() {
        let test_code = quote! {
            fn update(z: Bar) {
                match z {
                    Bar::A => {},
                    Bar::B(x) => {},
                    Bar::C{x, y} => {},
                }
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        println!("{:?}", function);
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_generics() {
        let test_code = quote! {
            fn do_stuff<T: Digital, S: Digital>(x: Foo<T>, y: Foo<S>) -> bool {
                let c = x.a;
                let d = (x.a, y.b);
                let e = Foo::<T> { a: c, b: c };
                e == x
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        println!("{}", item);
        let new_code = quote! {#item};
        let result = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", result);
    }

    #[test]
    fn test_pattern_binding_test() {
        let test_code = quote! {
            match t {
                Foo::<T>{a: 1, b: 2} => {}
                Foo::<T>{a, b} => {}
                Foo(CACHE) => {}
                Foo{a: x} => {}
                Bar(CACHE, x) => {}
            }
        };
        let expr = syn::parse2::<syn::ExprMatch>(test_code).unwrap();
        expr.arms.iter().for_each(|arm| {
            eprintln!("{:?}", arm);
            eprintln!("pattern has bindings {:?}", pattern_has_bindings(&arm.pat));
        });
    }

    #[test]
    fn test_wrap_handles_mut_arg() {
        let test_code = quote! {
            fn update(mut a: u8) -> u8 {
                let mut b = 3;
                b = a;
                b
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let new_code = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", new_code);
    }

    #[test]
    fn test_argument_tuple() {
        let test_code = quote! {
            fn update((a , b): (u8, u8)) -> (u8, u8) {
                (a, a + b)
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let new_code = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", new_code);
    }

    #[test]
    fn test_arguments_with_single_element_tuples() {
        let test_code = quote! {
            fn counter<const N: usize>(i: CounterIn<N>, (count_q,): (Bits<N>,)) -> (Bits<N>, (DFFIn<Bits<N>>,)) {
                let count_q = count_q.0;
                let next = if i.enable { count_q + 1 } else { count_q };
                (
                    count_q,
                    (DFFIn {
                        clock: i.clock,
                        data: next,
                    },),
                )
            }
        };
        let function = syn::parse2::<syn::ItemFn>(test_code).unwrap();
        let item = Context::default().function(function).unwrap();
        let new_code = quote! {#item};
        let new_code = prettyplease::unparse(&syn::parse2::<syn::File>(new_code).unwrap());
        println!("{}", new_code);
    }

    #[test]
    #[allow(unused)]
    fn test_pattern_binding_rewrite_function() {
        enum Baz {
            A,
            B(u8),
            C { x: u8, y: u8 },
        }

        let a: Baz = Baz::A;
        match a {
            Baz::A => {}
            Baz::B(3) => {}
            Baz::C { x: 3, y: 4 } => {}
            Baz::C { x, y } => {}
            _ => {}
        }
        let test_code = quote! {
            match a {
                Baz::A => {}
                Baz::B(3) => {}
                Baz::C { x: 3, y: 4 } => {}
                Baz::C { x, y } => {}
                _ => {}
            }
        };
        let expr = syn::parse2::<syn::ExprMatch>(test_code).unwrap();
        expr.arms.iter().for_each(|arm| {
            eprintln!("{:?}", arm);
            eprintln!("pattern has bindings {:?}", pattern_has_bindings(&arm.pat));
            let rewrite = rewrite_pattern_to_use_defaults_for_bindings(&arm.pat);
            eprintln!("{}", rewrite);
        });
    }
}
