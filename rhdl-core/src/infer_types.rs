use crate::ast::{ExprAssign, ExprIf};
use crate::ty::{ty_bool, ty_empty};
use crate::unify::UnifyContext;
use crate::{
    ast::{self, BinOp, ExprBinary, ExprKind},
    ty::{ty_var, Ty},
    visit::{walk_block, Visitor},
};
use anyhow::Result;

#[derive(Default)]
pub struct TypeInference {
    context: UnifyContext,
}

impl TypeInference {
    pub fn infer(&mut self, root: &ast::Block) -> Result<()> {
        let mut generator = TypeInference::default();
        walk_block(&mut generator, root)?;
        println!("Type inference: {}", generator.context);
        Ok(())
    }
}

// Shortcut to allow us to reuse the node IDs as
// type variables in the resolver.
fn id_to_var(id: Option<ast::NodeId>) -> Result<Ty> {
    id.map(|x| x.as_u32() as usize)
        .map(ty_var)
        .ok_or_else(|| anyhow::anyhow!("No type ID found"))
}

impl Visitor for TypeInference {
    fn visit_expr(&mut self, expr: &ast::Expr) -> Result<()> {
        let my_ty = id_to_var(expr.id)?;
        match &expr.kind {
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
            // x <- y = z --> tx = {}, ty = tz
            ExprKind::Assign(ExprAssign { lhs, rhs }) => {
                self.context.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
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
            _ => {}
        }
        Ok(())
    }
}
