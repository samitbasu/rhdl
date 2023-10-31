use crate::ast::{ExprAssign, ExprIf, ExprLit, NodeId};
use crate::kernel::Kernel;
use crate::ty::{ty_array, ty_as_ref, ty_bits, ty_bool, ty_empty, ty_signed, ty_tuple};
use crate::unify::UnifyContext;
use crate::{
    ast::{self, BinOp, ExprBinary, ExprKind},
    ty::{ty_var, Ty},
    visit::{self, Visitor},
};
use anyhow::{bail, Result};
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

struct Scope {
    block: NodeId,
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

impl TypeInference {
    fn new_scope(&mut self, block: NodeId) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            block,
            names: HashMap::new(),
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
    fn current_scope(&self) -> ScopeId {
        self.active_scope
    }
    fn unify(&mut self, lhs: Ty, rhs: Ty) -> Result<()> {
        self.context.unify(lhs, rhs)
    }
    fn bind(&mut self, name: &str, id: Option<NodeId>) -> Result<()> {
        println!("Binding {} to {:?}", name, id);
        self.scopes[self.active_scope.0]
            .names
            .insert(name.to_string(), id_to_var(id)?);
        Ok(())
    }
    fn lookup(&mut self, path: &str) -> Option<Ty> {
        let mut scope = self.active_scope;
        loop {
            if let Some(ty) = self.scopes[scope.0].names.get(path) {
                return Some(ty.clone());
            }
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        None
    }
    fn bind_pattern(&mut self, pat: &ast::Pat) -> Result<()> {
        match &pat.kind {
            ast::PatKind::Ident(ref ident) => {
                self.bind(&ident.name, pat.id)?;
            }
            ast::PatKind::Tuple(ref tuple) => {
                for elem in tuple.elements.iter() {
                    self.bind_pattern(elem)?;
                }
                self.unify(
                    id_to_var(pat.id)?,
                    ty_tuple(
                        tuple
                            .elements
                            .iter()
                            .map(|elem| id_to_var(elem.id))
                            .collect::<Result<Vec<_>>>()?,
                    ),
                )?;
            }
            ast::PatKind::Slice(ref slice) => {
                let array_type = slice
                    .elems
                    .first()
                    .map(|x| id_to_var(x.id))
                    .transpose()?
                    .unwrap_or_else(ty_empty);
                for elem in slice.elems.iter() {
                    self.bind_pattern(elem)?;
                    self.context
                        .unify(id_to_var(elem.id)?, array_type.clone())?;
                }
                self.context
                    .unify(id_to_var(pat.id)?, ty_array(array_type, slice.elems.len()))?;
            }
            ast::PatKind::Type(ref ty) => {
                self.bind_pattern(&ty.pat)?;
                self.context
                    .unify(id_to_var(ty.pat.id)?, ty.kind.clone().into())?;
                self.context
                    .unify(id_to_var(pat.id)?, id_to_var(ty.pat.id)?)?;
            }
            _ => bail!("Unsupported pattern kind: {:?}", pat.kind),
        }
        Ok(())
    }
}

pub fn infer(root: &Kernel) -> Result<UnifyContext> {
    let mut generator = TypeInference::default();
    generator.new_scope(NodeId::new(100_000));
    for (ndx, (name, kind)) in root.args.iter().enumerate() {
        let id = NodeId::new((ndx + 100_000) as u32);
        generator.bind(name, Some(id))?;
        generator.unify(id_to_var(Some(id))?, kind.clone().into())?;
    }
    generator.visit_block(&root.code)?;
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
            self.unify(my_ty, id_to_var(expr.id)?)?;
        } else {
            self.unify(my_ty, ty_empty())?;
        }
        visit::visit_stmt(self, node)
    }
    fn visit_local(&mut self, node: &ast::Local) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.unify(my_ty, ty_empty())?;
        if let Some(init) = node.init.as_ref() {
            self.context
                .unify(id_to_var(node.pat.id)?, id_to_var(init.id)?)?;
        }
        self.bind_pattern(&node.pat)?;
        visit::visit_local(self, node)
    }
    fn visit_block(&mut self, node: &ast::Block) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.new_scope(node.id.ok_or(anyhow::anyhow!("No ID found"))?);
        // Block is unified with the last statement (or is empty)
        if let Some(stmt) = node.stmts.last() {
            self.unify(my_ty, id_to_var(stmt.id)?)?;
        } else {
            self.unify(my_ty, ty_empty())?;
        }
        visit::visit_block(self, node)?;
        self.end_scope();
        Ok(())
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
                self.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
                self.unify(my_ty, id_to_var(rhs.id)?)?;
            }
            // x <- l && r --> tx = tl = tr = bool
            ExprKind::Binary(ExprBinary {
                op: BinOp::And | BinOp::Or,
                lhs,
                rhs,
            }) => {
                self.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
                self.unify(my_ty.clone(), id_to_var(rhs.id)?)?;
                self.unify(my_ty, ty_bool())?;
            }
            // x <- l << r --> tx = tl
            ExprKind::Binary(ExprBinary {
                op: BinOp::Shl | BinOp::Shr,
                lhs,
                rhs: _,
            }) => {
                self.unify(my_ty.clone(), id_to_var(lhs.id)?)?;
            }
            // x <- l == r --> tx = bool, tl = tr
            ExprKind::Binary(ExprBinary {
                op: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge,
                lhs,
                rhs,
            }) => {
                self.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
                self.unify(my_ty, ty_bool())?;
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
                self.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
                self.unify(my_ty, ty_empty())?;
            }
            // x <- y = z --> tx = {}, ty = &tz
            ExprKind::Assign(ExprAssign { lhs, rhs }) => {
                self.context
                    .unify(id_to_var(lhs.id)?, ty_as_ref(id_to_var(rhs.id)?))?;
                self.unify(my_ty, ty_empty())?;
            }
            // x <- if c { t } else { e } --> tx = tt = te, tc = bool
            ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }) => {
                self.unify(id_to_var(cond.id)?, ty_bool())?;
                self.context
                    .unify(my_ty.clone(), id_to_var(then_branch.id)?)?;
                if let Some(else_branch) = else_branch {
                    self.unify(my_ty, id_to_var(else_branch.id)?)?;
                }
            }
            // x <- bits::<len>(y) --> tx = bits<len>
            // TODO - make this extensible and not gross.
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
                            self.unify(my_ty, ty_bits(bits))?;
                        }
                    }
                } else if call.path.segments.len() == 1
                    && call.path.segments[0].ident == "signed"
                    && call.args.len() == 1
                    && call.path.segments[0].arguments.len() == 1
                {
                    if let ExprKind::Lit(ExprLit::Int(len)) =
                        &call.path.segments[0].arguments[0].kind
                    {
                        if let Ok(bits) = len.parse::<usize>() {
                            self.unify(my_ty, ty_signed(bits))?;
                        }
                    }
                }
            }
            ExprKind::Tuple(tuple) => {
                self.unify(
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
            ExprKind::Array(array) => {
                let array_type = array
                    .elems
                    .first()
                    .map(|x| id_to_var(x.id))
                    .transpose()?
                    .unwrap_or_else(ty_empty);
                let array_len = array.elems.len();
                self.context
                    .unify(my_ty, ty_array(array_type.clone(), array_len))?;
                for elem in &array.elems {
                    self.context
                        .unify(id_to_var(elem.id)?, array_type.clone())?;
                }
            }
            ExprKind::Block(block) => {
                self.unify(my_ty, id_to_var(block.block.id)?)?;
            }
            ExprKind::Path(path) => {
                if path.path.segments.len() == 1 && path.path.segments[0].arguments.is_empty() {
                    let name = &path.path.segments[0].ident;
                    if let Some(ty) = self.lookup(name) {
                        self.unify(my_ty, ty.clone())?;
                    }
                }
            }
            ExprKind::Field(field) => {
                visit::visit_expr(self, node)?;
                let arg = id_to_var(field.expr.id)?;
                let sub = match field.member {
                    ast::Member::Named(ref name) => self.context.get_named_field(arg, name),
                    ast::Member::Unnamed(ref index) => {
                        self.context.get_unnamed_field(arg, *index as usize)
                    }
                }?;
                self.unify(my_ty, sub)?;
            }
            _ => {}
        }
        visit::visit_expr(self, node)
    }
}
