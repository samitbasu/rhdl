use std::{collections::HashMap, ops::Range};

use log::debug;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;

use crate::rhdl_core::ast::ast_impl::{KernelFn, NodeId};

use crate::rhdl_core::ast::ast_impl as ast;

use super::spanned_source::SpannedSource;

struct SpannedSourceBuilder<'a> {
    source: &'a str,
    span_map: HashMap<NodeId, Range<usize>>,
}

impl<'a> SpannedSourceBuilder<'a> {
    fn new(source: &'a str) -> syn::Result<Self> {
        Ok(SpannedSourceBuilder {
            source,
            span_map: HashMap::new(),
        })
    }
    fn build(mut self, kernel: &KernelFn) -> syn::Result<SpannedSource> {
        let fn_item = syn::parse_str::<syn::ItemFn>(self.source)?;
        let span = fn_item.span().byte_range();
        self.span_map.insert(kernel.id, span);
        if fn_item.sig.inputs.len() != kernel.inputs.len() {
            return Err(syn::Error::new(
                fn_item.sig.inputs.span(),
                "Mismatched number of inputs",
            ));
        }
        for (syn_input, ast_input) in fn_item.sig.inputs.iter().zip(kernel.inputs.iter()) {
            let syn::FnArg::Typed(syn_input) = syn_input else {
                return Err(syn::Error::new(syn_input.span(), "Mismatched input kinds"));
            };
            self.span_map
                .insert(ast_input.id, syn_input.span().byte_range());
            let ast::PatKind::Type(ast_input) = &ast_input.kind else {
                return Err(syn::Error::new(syn_input.span(), "Mismatched input kinds"));
            };
            self.pattern(&syn_input.pat, &ast_input.pat)?;
        }
        self.block(&fn_item.block, &kernel.body)?;
        Ok(SpannedSource {
            source: self.source.into(),
            name: kernel.name.into(),
            span_map: self.span_map,
            fallback: kernel.id,
            filename: kernel.file.into(),
            function_id: kernel.fn_id,
        })
    }
    fn block(&mut self, syn_block: &syn::Block, ast_block: &ast::Block) -> syn::Result<()> {
        let span = syn_block.span().byte_range();
        self.span_map.insert(ast_block.id, span);
        if syn_block.stmts.len() != ast_block.stmts.len() {
            return Err(syn::Error::new(
                syn_block.span(),
                "Mismatched number of statements",
            ));
        }
        for (syn_stmt, ast_stmt) in syn_block.stmts.iter().zip(ast_block.stmts.iter()) {
            self.stmt(syn_stmt, ast_stmt)?;
        }
        Ok(())
    }
    fn stmt(&mut self, syn_stmt: &syn::Stmt, ast_stmt: &ast::Stmt) -> syn::Result<()> {
        log::trace!("AST statement: (id: {:?}) {:?}", ast_stmt.id, ast_stmt);
        log::trace!("Syn statement: {:?}", syn_stmt);
        self.span_map
            .insert(ast_stmt.id, syn_stmt.span().byte_range());
        match (&syn_stmt, &ast_stmt.kind) {
            (syn::Stmt::Local(syn_local), ast::StmtKind::Local(ast_local)) => {
                self.local(syn_local, ast_local)
            }
            (
                syn::Stmt::Expr(syn_expr, _),
                ast::StmtKind::Expr(ast_expr) | ast::StmtKind::Semi(ast_expr),
            ) => self.expr(syn_expr, ast_expr),
            _ => Err(syn::Error::new(
                syn_stmt.span(),
                "Mismatched statement kinds",
            )),
        }
    }
    fn local(&mut self, syn_local: &syn::Local, ast_local: &ast::Local) -> syn::Result<()> {
        self.span_map
            .insert(ast_local.id, syn_local.span().byte_range());
        self.pattern(&syn_local.pat, &ast_local.pat)?;
        match (&syn_local.init, &ast_local.init) {
            (Some(syn_local_init), Some(ast_expr)) => self.expr(&syn_local_init.expr, ast_expr),
            (None, None) => Ok(()),
            _ => Err(syn::Error::new(
                syn_local.span(),
                "Mismatched local initialization",
            )),
        }
    }
    fn pat_list<T>(
        &mut self,
        syn_pats: &syn::punctuated::Punctuated<syn::Pat, T>,
        ast_pats: &[Box<ast::Pat>],
        span: Span,
    ) -> syn::Result<()> {
        if syn_pats.len() != ast_pats.len() {
            return Err(syn::Error::new(span, "Mismatched number of patterns"));
        }
        for (syn_pat, ast_pat) in syn_pats.iter().zip(ast_pats.iter()) {
            self.pattern(syn_pat, ast_pat)?;
        }
        Ok(())
    }
    fn pattern(&mut self, syn_pat: &syn::Pat, ast_pat: &ast::Pat) -> syn::Result<()> {
        debug!("pattern: {:?} -> {:?}", ast_pat.id, syn_pat.span());
        self.span_map
            .insert(ast_pat.id, syn_pat.span().byte_range());
        match (&syn_pat, &ast_pat.kind) {
            (syn::Pat::Ident(_), ast::PatKind::Ident(_)) => Ok(()),
            (syn::Pat::Wild(_), ast::PatKind::Wild) => Ok(()),
            (syn::Pat::Lit(_), ast::PatKind::Lit(_)) => Ok(()),
            (syn::Pat::Or(syn), ast::PatKind::Or(ast)) => {
                self.pat_list(&syn.cases, &ast.segments, syn.span())
            }
            (syn::Pat::Paren(syn), ast::PatKind::Paren(ast)) => self.pattern(&syn.pat, &ast.pat),
            (syn::Pat::Path(_), ast::PatKind::Path(_)) => Ok(()),
            (syn::Pat::Slice(syn), ast::PatKind::Slice(ast)) => {
                self.pat_list(&syn.elems, &ast.elems, syn.span())
            }
            (syn::Pat::Struct(syn), ast::PatKind::Struct(ast)) => {
                if syn.fields.len() != ast.fields.len() {
                    return Err(syn::Error::new(
                        syn.span(),
                        "Mismatched number of struct-pattern fields",
                    ));
                }
                for (syn_field, ast_field) in syn.fields.iter().zip(ast.fields.iter()) {
                    self.pattern(&syn_field.pat, &ast_field.pat)?;
                }
                Ok(())
            }
            (syn::Pat::Tuple(syn), ast::PatKind::Tuple(ast)) => {
                self.pat_list(&syn.elems, &ast.elements, syn.span())
            }
            (syn::Pat::TupleStruct(syn), ast::PatKind::TupleStruct(ast)) => {
                self.pat_list(&syn.elems, &ast.elems, syn.span())
            }
            (syn::Pat::Type(syn), ast::PatKind::Type(ast)) => self.pattern(&syn.pat, &ast.pat),
            (_, ast::PatKind::Wild) => Ok(()),
            _ => Err(syn::Error::new(
                syn_pat.span(),
                format!("Mismatched pattern kinds {:#?} {:#?}", syn_pat, ast_pat,),
            )),
        }
    }
    fn expr_list<T>(
        &mut self,
        syn: &syn::punctuated::Punctuated<syn::Expr, T>,
        ast: &[Box<ast::Expr>],
        span: Span,
    ) -> syn::Result<()> {
        if syn.len() != ast.len() {
            return Err(syn::Error::new(span, "Mismatched number of expressions"));
        }
        for (syn_expr, ast_expr) in syn.iter().zip(ast.iter()) {
            self.expr(syn_expr, ast_expr)?;
        }
        Ok(())
    }
    fn expr_opts(
        &mut self,
        syn: &Option<Box<syn::Expr>>,
        ast: &Option<Box<ast::Expr>>,
        span: Span,
    ) -> syn::Result<()> {
        match (syn, ast) {
            (Some(syn_expr), Some(ast_expr)) => self.expr(syn_expr, ast_expr),
            (None, None) => Ok(()),
            _ => Err(syn::Error::new(span, "Mismatched number of expressions")),
        }
    }
    fn arm(&mut self, syn: &syn::Arm, ast: &ast::Arm) -> syn::Result<()> {
        self.span_map.insert(ast.id, syn.span().byte_range());
        if let (syn, ast::ArmKind::Enum(ast)) = (&syn.pat, &ast.kind) {
            self.pattern(syn, &ast.pat)?
        }
        self.expr(&syn.body, &ast.body)
    }
    fn expr_if_let(&mut self, syn: &syn::ExprIf, ast: &ast::ExprIfLet) -> syn::Result<()> {
        let syn::Expr::Let(syn_let) = syn.cond.as_ref() else {
            return Err(syn::Error::new(syn.span(), "Mismatched if-let condition"));
        };
        if let ast::ArmKind::Enum(arm_pat) = &ast.kind {
            self.pattern(&syn_let.pat, &arm_pat.pat)?;
        }
        self.expr(&syn_let.expr, &ast.test)?;
        self.block(&syn.then_branch, &ast.then_block)?;
        match (&syn.else_branch, &ast.else_branch) {
            (Some((_, syn_else)), Some(ast_else)) => self.expr(syn_else, ast_else),
            (None, None) => Ok(()),
            _ => Err(syn::Error::new(
                syn.span(),
                "Mismatched number of else branches",
            )),
        }
    }
    fn expr(&mut self, syn: &syn::Expr, ast: &ast::Expr) -> syn::Result<()> {
        self.span_map.insert(ast.id, syn.span().byte_range());
        match (syn, &ast.kind) {
            (syn::Expr::Array(syn), ast::ExprKind::Array(ast)) => {
                self.expr_list(&syn.elems, &ast.elems, syn.span())
            }
            (syn::Expr::Binary(syn), ast::ExprKind::Binary(ast)) => {
                self.expr(&syn.left, &ast.lhs)?;
                self.expr(&syn.right, &ast.rhs)
            }
            (syn::Expr::Assign(syn), ast::ExprKind::Assign(ast)) => {
                self.expr(&syn.left, &ast.lhs)?;
                self.expr(&syn.right, &ast.rhs)
            }
            (syn::Expr::Block(syn), ast::ExprKind::Block(ast)) => {
                self.block(&syn.block, &ast.block)
            }
            (syn::Expr::Call(syn), ast::ExprKind::Call(ast)) => {
                self.expr_list(&syn.args, &ast.args, syn.span())
            }
            (syn::Expr::Call(syn), ast::ExprKind::Bits(ast)) => {
                self.expr_list(&syn.args, &[ast.arg.clone()], syn.span())
            }
            (syn::Expr::Call(syn), ast::ExprKind::Block(ast)) => {
                self.span_map.insert(ast.block.id, syn.span().byte_range());
                Ok(())
            }
            (syn::Expr::Field(syn), ast::ExprKind::Field(ast)) => self.expr(&syn.base, &ast.expr),
            (syn::Expr::ForLoop(syn), ast::ExprKind::ForLoop(ast)) => {
                self.pattern(&syn.pat, &ast.pat)?;
                self.expr(&syn.expr, &ast.expr)?;
                self.block(&syn.body, &ast.body)
            }
            (syn::Expr::Group(syn), ast::ExprKind::Group(ast)) => self.expr(&syn.expr, &ast.expr),
            (syn::Expr::If(syn), ast::ExprKind::If(ast)) => {
                self.expr(&syn.cond, &ast.cond)?;
                self.block(&syn.then_branch, &ast.then_branch)?;
                match (&syn.else_branch, &ast.else_branch) {
                    (Some((_, syn_else)), Some(ast_else)) => self.expr(syn_else, ast_else),
                    (None, None) => Ok(()),
                    _ => Err(syn::Error::new(
                        syn.span(),
                        "Mismatched number of else branches",
                    )),
                }
            }
            (syn::Expr::If(syn), ast::ExprKind::IfLet(ast))
                if matches!(syn.cond.as_ref(), syn::Expr::Let(_)) =>
            {
                self.expr_if_let(syn, ast)
            }
            (syn::Expr::Index(syn), ast::ExprKind::Index(ast)) => {
                self.expr(&syn.expr, &ast.expr)?;
                self.expr(&syn.index, &ast.index)
            }
            (syn::Expr::Let(syn), ast::ExprKind::Let(ast)) => {
                self.pattern(&syn.pat, &ast.pattern)?;
                self.expr(&syn.expr, &ast.value)
            }
            (_, ast::ExprKind::Lit(_)) => Ok(()),
            (syn::Expr::Match(syn), ast::ExprKind::Match(ast)) => {
                self.expr(&syn.expr, &ast.expr)?;
                if syn.arms.len() != ast.arms.len() {
                    return Err(syn::Error::new(
                        syn.span(),
                        "Mismatched number of match arms",
                    ));
                }
                for (syn_arm, ast_arm) in syn.arms.iter().zip(ast.arms.iter()) {
                    self.arm(syn_arm, ast_arm)?;
                }
                Ok(())
            }
            (syn::Expr::MethodCall(syn), ast::ExprKind::MethodCall(ast)) => {
                self.expr(&syn.receiver, &ast.receiver)?;
                self.expr_list(&syn.args, &ast.args, syn.span())
            }
            (syn::Expr::Paren(syn), ast::ExprKind::Paren(ast)) => self.expr(&syn.expr, &ast.expr),
            (syn::Expr::Path(_), ast::ExprKind::Path(_)) => Ok(()),
            (syn::Expr::Range(syn), ast::ExprKind::Range(ast)) => {
                self.expr_opts(&syn.start, &ast.start, syn.span())?;
                self.expr_opts(&syn.end, &ast.end, syn.span())
            }
            (syn::Expr::Repeat(syn), ast::ExprKind::Repeat(ast)) => {
                self.expr(&syn.expr, &ast.value)
            }
            (syn::Expr::Return(syn), ast::ExprKind::Ret(ast)) => {
                self.expr_opts(&syn.expr, &ast.expr, syn.span())
            }
            (syn::Expr::Struct(syn), ast::ExprKind::Struct(ast)) => {
                if syn.fields.len() != ast.fields.len() {
                    return Err(syn::Error::new(
                        syn.span(),
                        "Mismatched number of struct fields",
                    ));
                }
                for (syn_field, ast_field) in syn.fields.iter().zip(ast.fields.iter()) {
                    self.expr(&syn_field.expr, &ast_field.value)?;
                }
                self.expr_opts(&syn.rest, &ast.rest, syn.span())
            }
            (syn::Expr::Try(syn), ast::ExprKind::Try(ast)) => self.expr(&syn.expr, &ast.expr),
            (syn::Expr::Tuple(syn), ast::ExprKind::Tuple(ast)) => {
                self.expr_list(&syn.elems, &ast.elements, syn.span())
            }
            (syn::Expr::Unary(syn), ast::ExprKind::Unary(ast)) => self.expr(&syn.expr, &ast.expr),
            (syn::Expr::Cast(syn), ast::ExprKind::Cast(ast)) => self.expr(&syn.expr, &ast.expr),
            (_, ast::ExprKind::Type(_)) => Ok(()),
            (syn::Expr::Path(_), ast::ExprKind::Call(_)) => Ok(()),
            (syn::Expr::Block(syn), _kind) => {
                // It's possible that quote inserted a block, where we expected a statement.
                if syn.block.stmts.len() != 1 {
                    return Err(syn::Error::new(
                        syn.span(),
                        "Expected a block statement, found multiple statements",
                    ));
                }
                let syn::Stmt::Expr(syn_expr, _) = &syn.block.stmts[0] else {
                    return Err(syn::Error::new(syn.span(), "Expected an expression statement"));
                };
                self.expr(syn_expr, ast)
            }
            (syn, ast::ExprKind::Block(inner)) => {
                // It's also possible that quote elided a block
                if inner.block.stmts.len() != 1 {
                    return Err(syn::Error::new(
                        syn.span(),
                        "Expected a block statement, found multiple statements",
                    ));
                }
                let span = syn.span().byte_range();
                self.span_map.insert(ast.id, span.clone());
                self.span_map.insert(inner.block.id, span.clone());
                self.span_map.insert(inner.block.stmts[0].id, span);
                let ast_expr = match &inner.block.stmts[0].kind {
                    ast::StmtKind::Expr(expr) => expr,
                    ast::StmtKind::Semi(expr) => expr,
                    _ => {
                        return Err(syn::Error::new(
                            syn.span(),
                            "Expected an expression statement",
                        ));
                    }
                };
                self.expr(syn, ast_expr)
            }
            _ => Err(syn::Error::new(
                syn.span(),
                format!(
                    "Mismatched expression kinds \n\n ----- \n {:?} \n ---- \n {:?} \n ----- \n {:?}\n",
                    syn, quote!(#syn).to_string(), ast,
                ),
            )),
        }
    }
}

pub fn build_spanned_source_for_kernel(kernel: &KernelFn) -> syn::Result<SpannedSource> {
    let builder = SpannedSourceBuilder::new(kernel.text)?;
    builder.build(kernel)
}
