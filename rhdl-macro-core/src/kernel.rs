use std::collections::HashSet;

use quote::{format_ident, quote};
use syn::{spanned::Spanned, ExprPath, Ident, Pat, Path, Type};
type TS = proc_macro2::TokenStream;
type Result<T> = syn::Result<T>;

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

impl Context {
    fn function(&mut self, function: syn::ItemFn) -> Result<TS> {
        let orig_name = &function.sig.ident;
        let (impl_generics, ty_generics, where_clause) = function.sig.generics.split_for_impl();
        let phantom_fields = function
            .sig
            .generics
            .params
            .iter()
            .enumerate()
            .map(|(ndx, param)| {
                let ident = format_ident!("__phantom_{}", ndx);
                let ty = match param {
                    syn::GenericParam::Type(ty) => &ty.ident,
                    syn::GenericParam::Lifetime(lt) => &lt.lifetime.ident,
                    syn::GenericParam::Const(cst) => &cst.ident,
                };
                quote! {#ident: std::marker::PhantomData<#ty>}
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
            syn::ReturnType::Default => quote! {rhdl_core::Kind::Empty},
            syn::ReturnType::Type(_, ty) => {
                quote! {<#ty as rhdl_core::Digital>::static_kind()}
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
                    let kind = quote! {<#ty as rhdl_core::Digital>::static_kind()};
                    Ok(quote! { rhdl_core::ast_builder::type_pat(#pat, #kind)})
                }
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            #function

            struct #name #ty_generics {#(#phantom_fields,)*}

            impl #impl_generics rhdl_core::digital_fn::DigitalFn for #name #ty_generics #where_clause {
                fn kernel_fn() -> rhdl_core::digital_fn::KernelFnKind {
                    rhdl_core::ast_builder::kernel_fn(
                        stringify!(#orig_name),
                        vec!{#(#args),*},
                        #ret,
                        #block,
                    )
                }
            }
        })
    }

    fn block(&mut self, block: &syn::Block) -> Result<TS> {
        self.new_scope();
        let block = self.block_inner(block)?;
        self.end_scope();
        Ok(quote! {
            rhdl_core::ast_builder::block_expr(#block)
        })
    }

    fn block_inner(&mut self, block: &syn::Block) -> Result<TS> {
        let stmts = block
            .stmts
            .iter()
            .map(|x| self.stmt(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            rhdl_core::ast_builder::block(vec![#(#stmts),*],)
        })
    }

    fn stmt(&mut self, statement: &syn::Stmt) -> Result<TS> {
        match statement {
            syn::Stmt::Local(local) => self.stmt_local(local),
            syn::Stmt::Expr(expr, semi) => {
                let expr = self.expr(expr)?;
                if semi.is_some() {
                    Ok(quote! {
                        rhdl_core::ast_builder::semi_stmt(#expr)
                    })
                } else {
                    Ok(quote! {
                        rhdl_core::ast_builder::expr_stmt(#expr)
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
                rhdl_core::ast_builder::local_stmt(#pattern, #local_init)
        })
    }

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
                    rhdl_core::ast_builder::ident_pat(stringify!(#name).to_string(),#mutability)
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
                    rhdl_core::ast_builder::tuple_struct_pat(#path, vec![#(#elems),*])
                })
            }
            syn::Pat::Tuple(tuple) => {
                let elems = tuple
                    .elems
                    .iter()
                    .map(|x| self.pat(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok(quote! {
                    rhdl_core::ast_builder::tuple_pat(vec![#(#elems),*])
                })
            }
            syn::Pat::Slice(slice) => {
                let elems = slice
                    .elems
                    .iter()
                    .map(|x| self.pat(x))
                    .collect::<Result<Vec<_>>>()?;
                Ok(quote! {
                    rhdl_core::ast_builder::slice_pat(vec![#(#elems),*])
                })
            }
            syn::Pat::Path(path) => {
                let path = self.path_inner(&path.path)?;
                Ok(quote! {
                    rhdl_core::ast_builder::path_pat(#path)
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
                    rhdl_core::ast_builder::struct_pat(#path, vec![#(#fields),*], #rest)
                })
            }
            syn::Pat::Type(pat) => {
                let ty = &pat.ty;
                let pat = self.pat(&pat.pat)?;
                let kind = quote! {<#ty as rhdl_core::Digital>::static_kind()};
                Ok(quote! {
                    rhdl_core::ast_builder::type_pat(#pat, #kind)
                })
            }
            syn::Pat::Lit(pat) => {
                let lit = self.lit_inner(pat)?;
                Ok(quote! {
                    rhdl_core::ast_builder::lit_pat(#lit)
                })
            }
            syn::Pat::Wild(_) => Ok(quote! {
                rhdl_core::ast_builder::wild_pat()
            }),
            _ => Err(syn::Error::new(pat.span(), "Unsupported pattern type")),
        }
    }

    fn field_pat(&mut self, expr: &syn::FieldPat) -> Result<TS> {
        let member = self.member(&expr.member)?;
        let pat = self.pat(&expr.pat)?;
        Ok(quote! {
            rhdl_core::ast_builder::field_pat(#member, #pat)
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
            //syn::Expr::MethodCall(expr) => self.method_call(expr),
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
        Ok(quote! {
            rhdl_core::ast_builder::method_expr(#receiver, vec![#(#args),*], stringify!(#method).to_string())
        })
    }

    fn index(&mut self, expr: &syn::ExprIndex) -> Result<TS> {
        let index = self.expr(&expr.index)?;
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::index_expr(#expr, #index)
        })
    }

    fn array(&mut self, expr: &syn::ExprArray) -> Result<TS> {
        let elems = expr
            .elems
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            rhdl_core::ast_builder::array_expr(vec![#(#elems),*])
        })
    }

    // A call expression like `a = Foo(...)` can be either a variant
    // or an actual function call.  Use the `inspect_digital` function
    // to extract a call signature from the function.
    fn call(&mut self, expr: &syn::ExprCall) -> Result<TS> {
        let syn::Expr::Path(func_path) = expr.func.as_ref() else {
            return Err(syn::Error::new(
                expr.func.span(),
                "Unsupported function call in rhdl kernel function (only paths allowed here)",
            ));
        };
        // Kludge - I don't know how else to do this.  We want to differentiate
        // between the builtin variant constructor function and a RHDL kernel.
        // Unfortunately, the only way to do this is by adding some convention or
        // by coupling global state across the macro invocations (e.g., by writing to
        // a file or something equally gross).  So, I instead choose to use a simple
        // convention.  Enum Variants must be namespaced (e.g., `Foo::Bar`), and both
        // the enum and the variant must be capitalized.  This is a simple convention
        // that is easy to follow and easy to understand.  It also allows us to
        // differentiate between the two cases without any global state.
        let code = self.get_code(func_path)?;
        let call_to_get_type = quote!(rhdl_core::digital_fn::inspect_digital(#func_path));
        let path = self.path_inner(&func_path.path)?;
        let args = expr
            .args
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            rhdl_core::ast_builder::call_expr(#path, vec![#(#args),*], #call_to_get_type, #code)
        })
    }

    fn get_code(&mut self, path: &ExprPath) -> Result<TS> {
        if path.path.segments.len() == 0 {
            return Err(syn::Error::new(
                path.span(),
                "Empty path in rhdl kernel function",
            ));
        }
        let last = &path.path.segments.last().unwrap().ident;
        if last == "bits" || last == "signed" {
            return Ok(quote!(None));
        }
        if path.path.segments.len() >= 2 {
            let len = path.path.segments.len();
            let last = &path.path.segments[len - 1];
            let second_to_last = &path.path.segments[len - 2];
            if ident_starts_with_capital_letter(&last.ident)
                && ident_starts_with_capital_letter(&second_to_last.ident)
            {
                return Ok(quote!(None));
            }
        }
        Ok(quote!(Some(<#path as rhdl_core::digital_fn::DigitalFn>::kernel_fn())))
    }

    fn for_loop(&mut self, expr: &syn::ExprForLoop) -> Result<TS> {
        self.new_scope();
        let pat = self.pat(&expr.pat)?;
        self.add_scoped_binding(&expr.pat)?;
        let body = self.block_inner(&expr.body)?;
        let expr = self.expr(&expr.expr)?;
        self.end_scope();
        Ok(quote! {
            rhdl_core::ast_builder::for_expr(#pat, #expr, #body)
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
        let len = self.expr(&expr.len)?;
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::repeat_expr(#expr, #len)
        })
    }

    fn tuple(&mut self, expr: &syn::ExprTuple) -> Result<TS> {
        let elems = expr
            .elems
            .iter()
            .map(|x| self.expr(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            rhdl_core::ast_builder::tuple_expr(vec![#(#elems),*])
        })
    }

    fn group(&mut self, expr: &syn::ExprGroup) -> Result<TS> {
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::group_expr(#expr)
        })
    }

    fn paren(&mut self, expr: &syn::ExprParen) -> Result<TS> {
        let expr = self.expr(&expr.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::paren_expr(#expr)
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
            rhdl_core::ast_builder::return_expr(#expr)
        })
    }

    fn try_ex(&mut self, expr: &syn::ExprTry) -> Result<TS> {
        Err(syn::Error::new(
            expr.span(),
            "Unsupported try expression in rhdl kernel function",
        ))
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
                quote!(rhdl_core::ast_builder::range_limits_half_open())
            }
            syn::RangeLimits::Closed(_) => quote!(rhdl_core::ast_builder::range_limits_closed()),
        };
        Ok(quote! {
            rhdl_core::ast_builder::range_expr(#start, #limits, #end)
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
            rhdl_core::ast_builder::match_expr(#expr, vec![#(#arms),*])
        })
    }

    fn arm(&mut self, arm: &syn::Arm) -> Result<TS> {
        self.new_scope();
        let pat = self.pat(&arm.pat)?;
        self.add_scoped_binding(&arm.pat)?;
        let guard = arm
            .guard
            .as_ref()
            .map(|(_if, x)| self.expr(x))
            .transpose()?
            .map(|x| quote! {Some(#x)})
            .unwrap_or(quote! {None});
        let body = self.expr(&arm.body)?;
        self.end_scope();
        Ok(quote! {
            rhdl_core::ast_builder::arm(#pat, #guard, #body)
        })
    }

    fn let_ex(&mut self, expr: &syn::ExprLet) -> Result<TS> {
        let pattern = self.pat(&expr.pat)?;
        let value = self.expr(&expr.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::let_expr(#pattern, #value)
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
            rhdl_core::ast_builder::if_expr(#cond, #then, #else_)
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
        let path = self.path_inner(&structure.path)?;
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
            .unwrap_or(quote! {None});
        let kind = if structure.rest.is_some() {
            // The presence of a rest means we know that path -> struct
            let path = &structure.path;
            quote!(< #path as rhdl_core::Digital>::static_kind())
        } else {
            let path = &structure.path;
            // Could be either a struct or an enum.  So we have to construct one
            // to find out.  And the only way to do that is to use the fields and
            // fill defaults for them all.
            let fields_with_default = structure
                .fields
                .iter()
                .map(|x| self.default_field_value(x))
                .collect::<Result<Vec<_>>>()?;
            quote! {
                Digital::kind(& #path { #(#fields_with_default),* })
            }
        };
        Ok(quote! {
            rhdl_core::ast_builder::struct_expr(#path,vec![#(#fields),*],#rest,#kind),
        })
    }

    fn default_field_value(&mut self, field: &syn::FieldValue) -> Result<TS> {
        match &field.member {
            syn::Member::Unnamed(_) => Err(syn::Error::new(
                field.span(),
                "Unsupported unnamed field in rhdl kernel function",
            )),
            syn::Member::Named(x) => Ok(quote!(#x: Default::default())),
        }
    }

    fn path(&mut self, path: &syn::Path) -> Result<TS> {
        // Check for a locally defined path
        let inner = self.path_inner(path)?;
        if !self.is_scoped_binding(path) {
            return Ok(quote! {
                rhdl_core::ast_builder::expr_typed_bits(#inner, rhdl_core::Digital::typed_bits(#path))
            });
        }
        Ok(quote! {
            rhdl_core::ast_builder::path_expr(#inner)
        })
    }

    fn path_inner(&mut self, path: &syn::Path) -> Result<TS> {
        let segments = path
            .segments
            .iter()
            .map(|x| self.path_segment(x))
            .collect::<Result<Vec<_>>>()?;
        Ok(quote! {
            rhdl_core::ast_builder::path(vec![#(#segments),*],)
        })
    }

    fn path_segment(&mut self, segment: &syn::PathSegment) -> Result<TS> {
        let ident = &segment.ident;
        let args = self.path_arguments(&segment.arguments)?;
        Ok(quote! {
            rhdl_core::ast_builder::path_segment(stringify!(#ident).to_string(), #args)
        })
    }

    fn path_arguments(&mut self, arguments: &syn::PathArguments) -> Result<TS> {
        // We only allow Const arguments.
        let args = match arguments {
            syn::PathArguments::None => quote! {rhdl_core::ast_builder::path_arguments_none()},
            syn::PathArguments::AngleBracketed(args) => {
                let args = args
                    .args
                    .iter()
                    .map(|x| self.generic_argument(x))
                    .collect::<Result<Vec<_>>>()?;
                quote! {rhdl_core::ast_builder::path_arguments_angle_bracketed(vec![#(#args),*])}
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
            syn::GenericArgument::Const(expr) => {
                let expr = self.expr(expr)?;
                Ok(quote! {
                    rhdl_core::ast_builder::generic_argument_const(#expr)
                })
            }
            syn::GenericArgument::Type(Type::Path(path)) => Ok(quote! {
                rhdl_core::ast_builder::generic_argument_type(<#path as Digital>::static_kind())
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
            rhdl_core::ast_builder::assign_expr(#left, #right)
        })
    }

    fn field_expression(&mut self, field: &syn::ExprField) -> Result<TS> {
        let expr = self.expr(&field.base)?;
        let member = self.member(&field.member)?;
        Ok(quote! {
            rhdl_core::ast_builder::field_expr(#expr, #member)
        })
    }

    fn field_value(&mut self, field: &syn::FieldValue) -> Result<TS> {
        let member = self.member(&field.member)?;
        let value = self.expr(&field.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::field_value(#member, #value)
        })
    }

    fn member(&mut self, member: &syn::Member) -> Result<TS> {
        Ok(match member {
            syn::Member::Named(ident) => quote! {
                rhdl_core::ast_builder::member_named(stringify!(#ident).to_string())
            },
            syn::Member::Unnamed(index) => {
                let index = index.index;
                quote! {
                    rhdl_core::ast_builder::member_unnamed(#index)
                }
            }
        })
    }

    fn unary(&mut self, unary: &syn::ExprUnary) -> Result<TS> {
        let op = match unary.op {
            syn::UnOp::Neg(_) => quote!(rhdl_core::ast_builder::UnOp::Neg),
            syn::UnOp::Not(_) => quote!(rhdl_core::ast_builder::UnOp::Not),
            _ => {
                return Err(syn::Error::new(
                    unary.span(),
                    "Unsupported unary operator in rhdl kernel function",
                ))
            }
        };
        let expr = self.expr(&unary.expr)?;
        Ok(quote! {
            rhdl_core::ast_builder::unary_expr(#op, #expr)
        })
    }

    fn binary(&mut self, binary: &syn::ExprBinary) -> Result<TS> {
        let op = match binary.op {
            syn::BinOp::Add(_) => quote!(rhdl_core::ast_builder::BinOp::Add),
            syn::BinOp::Sub(_) => quote!(rhdl_core::ast_builder::BinOp::Sub),
            syn::BinOp::Mul(_) => quote!(rhdl_core::ast_builder::BinOp::Mul),
            syn::BinOp::And(_) => quote!(rhdl_core::ast_builder::BinOp::And),
            syn::BinOp::Or(_) => quote!(rhdl_core::ast_builder::BinOp::Or),
            syn::BinOp::BitXor(_) => quote!(rhdl_core::ast_builder::BinOp::BitXor),
            syn::BinOp::BitAnd(_) => quote!(rhdl_core::ast_builder::BinOp::BitAnd),
            syn::BinOp::BitOr(_) => quote!(rhdl_core::ast_builder::BinOp::BitOr),
            syn::BinOp::Shl(_) => quote!(rhdl_core::ast_builder::BinOp::Shl),
            syn::BinOp::Shr(_) => quote!(rhdl_core::ast_builder::BinOp::Shr),
            syn::BinOp::Eq(_) => quote!(rhdl_core::ast_builder::BinOp::Eq),
            syn::BinOp::Lt(_) => quote!(rhdl_core::ast_builder::BinOp::Lt),
            syn::BinOp::Le(_) => quote!(rhdl_core::ast_builder::BinOp::Le),
            syn::BinOp::Ne(_) => quote!(rhdl_core::ast_builder::BinOp::Ne),
            syn::BinOp::Ge(_) => quote!(rhdl_core::ast_builder::BinOp::Ge),
            syn::BinOp::Gt(_) => quote!(rhdl_core::ast_builder::BinOp::Gt),
            syn::BinOp::AddAssign(_) => quote!(rhdl_core::ast_builder::BinOp::AddAssign),
            syn::BinOp::SubAssign(_) => quote!(rhdl_core::ast_builder::BinOp::SubAssign),
            syn::BinOp::MulAssign(_) => quote!(rhdl_core::ast_builder::BinOp::MulAssign),
            syn::BinOp::BitXorAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitXorAssign),
            syn::BinOp::BitAndAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitAndAssign),
            syn::BinOp::BitOrAssign(_) => quote!(rhdl_core::ast_builder::BinOp::BitOrAssign),
            syn::BinOp::ShlAssign(_) => quote!(rhdl_core::ast_builder::BinOp::ShlAssign),
            syn::BinOp::ShrAssign(_) => quote!(rhdl_core::ast_builder::BinOp::ShrAssign),
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
            rhdl_core::ast_builder::binary_expr(#op, #left, #right)
        })
    }

    fn lit(&mut self, lit: &syn::ExprLit) -> Result<TS> {
        let inner = self.lit_inner(lit)?;
        Ok(quote! {
            rhdl_core::ast_builder::lit_expr(#inner)
        })
    }

    fn lit_inner(&mut self, lit: &syn::ExprLit) -> Result<TS> {
        let lit = &lit.lit;
        match lit {
            syn::Lit::Int(int) => {
                let value = int.token();
                Ok(quote! {
                        rhdl_core::ast_builder::expr_lit_int(stringify!(#value).to_string())
                })
            }
            syn::Lit::Bool(boolean) => {
                let value = boolean.value;
                Ok(quote! {
                        rhdl_core::ast_builder::expr_lit_bool(#value)
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
        let result = result.replace("rhdl_core :: ast :: ", "");
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
        let result = result.replace("rhdl_core :: ast :: ", "");
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
        let result = result.replace("rhdl_core :: ast :: ", "");
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
        //        let result = result.replace("rhdl_core :: ast :: ", "");
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
        //        let result = result.replace("rhdl_core :: ast :: ", "");
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
                    2_u4 => {}
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
}
