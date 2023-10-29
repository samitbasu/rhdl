use std::collections::HashMap;

use crate::ast::{ExprAssign, ExprIf, ExprLit};
use crate::ty::{ty_as_ref, ty_bits, ty_bool, ty_empty, ty_tuple};
use crate::unify::UnifyContext;
use crate::{
    ast::{self, BinOp, ExprBinary, ExprKind},
    ty::{ty_var, Ty},
    visit::{self, Visitor},
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

struct Scope {
    names: HashMap<String, Ty>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

#[derive(Default)]
pub struct TypeInference {
    scopes: Vec<Scope>,
    active_scope: ScopeId,
    context: UnifyContext,
}

pub fn infer(root: &ast::Block) -> Result<UnifyContext> {
    let mut generator = TypeInference::default();
    generator.visit_block(root)?;
    println!("Type inference: {}", generator.context);
    Ok(generator.context)
}

// Shortcut to allow us to reuse the node IDs as
// type variables in the resolver.
pub fn id_to_var(id: Option<ast::NodeId>) -> Result<Ty> {
    id.map(|x| x.as_u32() as usize)
        .map(ty_var)
        .ok_or_else(|| anyhow::anyhow!("No type ID found"))
}

impl Visitor for TypeInference {
    fn visit_stmt(&mut self, node: &ast::Stmt) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        if let ast::StmtKind::Expr(expr) = &node.kind {
            self.context.unify(my_ty, id_to_var(expr.id)?)?;
        } else {
            self.context.unify(my_ty, ty_empty())?;
        }
        visit::visit_stmt(self, node)
    }
    fn visit_local(&mut self, node: &ast::Local) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.context.unify(my_ty, ty_empty())?;
        if let Some(init) = node.init.as_ref() {
            self.context
                .unify(id_to_var(node.pat.id)?, id_to_var(init.id)?)?;
        }
        visit::visit_local(self, node)
    }
    fn visit_block(&mut self, node: &ast::Block) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        // Block is unified with the last statement (or is empty)
        if let Some(stmt) = node.stmts.last() {
            self.context.unify(my_ty, id_to_var(stmt.id)?)?;
        } else {
            self.context.unify(my_ty, ty_empty())?;
        }
        visit::visit_block(self, node)
    }
    fn visit_expr(&mut self, node: &ast::Expr) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        match &node.kind {
            // x <- l + r --> tx = tl = tr
            ExprKind::Binary(ExprBinary {
                op:
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::BitXor | BinOp::BitAnd | BinOp::BitOr,
                lhs,
                rhs,
            }) => {
                self.context.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
                self.context.unify(my_ty, id_to_var(rhs.id)?)?;
            }
            // x <- l && r --> tx = tl = tr = bool
            ExprKind::Binary(ExprBinary {
                op: BinOp::And | BinOp::Or,
                lhs,
                rhs,
            }) => {
                self.context.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
                self.context.unify(my_ty.clone(), id_to_var(rhs.id)?)?;
                self.context.unify(my_ty, ty_bool())?;
            }
            // x <- l << r --> tx = tl
            ExprKind::Binary(ExprBinary {
                op: BinOp::Shl | BinOp::Shr,
                lhs,
                rhs: _,
            }) => {
                self.context.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
            }
            // x <- l == r --> tx = bool, tl = tr
            ExprKind::Binary(ExprBinary {
                op: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge,
                lhs,
                rhs,
            }) => {
                self.context.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
                self.context.unify(my_ty, ty_bool())?;
            }
            // x <- l += r --> tx = {}, tl = tr
            ExprKind::Binary(ExprBinary {
                op:
                    BinOp::AddAssign
                    | BinOp::SubAssign
                    | BinOp::MulAssign
                    | BinOp::BitXorAssign
                    | BinOp::BitAndAssign
                    | BinOp::BitOrAssign,
                lhs,
                rhs,
            }) => {
                self.context.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
                self.context.unify(my_ty, ty_empty())?;
            }
            // x <- y = z --> tx = {}, ty = &tz
            ExprKind::Assign(ExprAssign { lhs, rhs }) => {
                self.context
                    .unify(id_to_var(lhs.id)?, ty_as_ref(id_to_var(rhs.id)?))?;
                self.context.unify(my_ty, ty_empty())?;
            }
            // x <- if c { t } else { e } --> tx = tt = te, tc = bool
            ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }) => {
                self.context.unify(id_to_var(cond.id)?, ty_bool())?;
                self.context
                    .unify(my_ty.clone(), id_to_var(then_branch.id)?)?;
                if let Some(else_branch) = else_branch {
                    self.context.unify(my_ty, id_to_var(else_branch.id)?)?;
                }
            }
            // x <- bits::<len>(y) --> tx = bits<len>
            ExprKind::Call(call) => {
                if call.path.segments.len() == 1
                    && call.path.segments[0].ident == "bits"
                    && call.args.len() == 1
                    && call.path.segments[0].arguments.len() == 1
                {
                    if let ExprKind::Lit(ExprLit::Int(len)) =
                        &call.path.segments[0].arguments[0].kind
                    {
                        if let Ok(bits) = len.parse::<usize>() {
                            self.context.unify(my_ty, ty_bits(bits))?;
                        }
                    }
                }
            }
            ExprKind::Tuple(tuple) => {
                self.context.unify(
                    my_ty,
                    ty_tuple(
                        tuple
                            .elements
                            .iter()
                            .map(|elem| id_to_var(elem.id))
                            .collect::<Result<Vec<_>>>()?,
                    ),
                )?;
            }
            ExprKind::Block(block) => {
                self.context.unify(my_ty, id_to_var(block.block.id)?)?;
            }
            _ => {}
        }
        visit::visit_expr(self, node)
    }
}
