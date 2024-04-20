use crate::ast::{ast_impl, visit};
use crate::compiler::{ty::Ty, UnifyContext};
use crate::{ast::visit::Visitor, kernel::Kernel};
use anyhow::bail;
use anyhow::Result;

use crate::compiler::ascii::render_statement_to_string;
use crate::compiler::display_ast::pretty_print_statement;
use crate::compiler::infer_types::id_to_var;

pub fn check_inference(kernel: &Kernel, ty: &UnifyContext) -> Result<()> {
    let mut validator = Validator::new(ty);
    validator.validate_kernel(kernel)
}

struct Validator<'a> {
    ty: &'a UnifyContext,
    current_statement: String,
}

impl<'a> Validator<'a> {
    fn new(ty: &'a UnifyContext) -> Self {
        Self {
            ty,
            current_statement: Default::default(),
        }
    }
    fn validate_kernel(&mut self, kernel: &Kernel) -> Result<()> {
        crate::ast::visit::visit_kernel_fn(self, kernel.inner())
    }
    fn validate_bound_type(&mut self, node_id: ast_impl::NodeId) -> Result<()> {
        let var = id_to_var(node_id)?;
        let ty = self.ty.apply(var);
        if let Ty::Var(ndx) = ty {
            bail!(
                "unbound type variable V{} in statement: {}",
                ndx.0,
                self.current_statement
            );
        }
        Ok(())
    }
}

impl<'a> Visitor for Validator<'a> {
    fn visit_stmt(&mut self, node: &crate::ast::ast_impl::Stmt) -> Result<()> {
        self.current_statement = format!(
            "\n\nStatement:\n{}AST:\n{}",
            pretty_print_statement(node, self.ty)?,
            render_statement_to_string(node, self.ty)?
        );
        self.validate_bound_type(node.id)?;
        visit::visit_stmt(self, node)
    }
    fn visit_block(&mut self, node: &crate::ast::ast_impl::Block) -> Result<()> {
        self.validate_bound_type(node.id)?;
        visit::visit_block(self, node)
    }
    fn visit_local(&mut self, node: &crate::ast::ast_impl::Local) -> Result<()> {
        self.validate_bound_type(node.id)?;
        visit::visit_local(self, node)
    }
    fn visit_pat(&mut self, node: &crate::ast::ast_impl::Pat) -> Result<()> {
        self.validate_bound_type(node.id)?;
        visit::visit_pat(self, node)
    }
    fn visit_expr(&mut self, node: &ast_impl::Expr) -> Result<()> {
        self.validate_bound_type(node.id)?;
        visit::visit_expr(self, node)
    }
}
