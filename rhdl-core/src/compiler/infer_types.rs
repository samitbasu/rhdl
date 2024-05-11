use crate::ast::ast_impl::{ArmKind, ExprAssign, ExprIf, ExprLit, ExprUnary, Member, NodeId};
use crate::ast::{
    ast_impl::{self, BinOp, ExprBinary, ExprKind},
    visit::{self, Visitor},
};
use crate::compiler::ty;
use crate::compiler::ty::{
    ty_array, ty_bits, ty_bool, ty_empty, ty_integer, ty_signed, ty_tuple, ty_usize, ty_var, Ty,
};
use crate::compiler::UnifyContext;
use crate::kernel::Kernel;
use crate::Kind;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

use super::ty::ty_signal;

#[derive(Copy, Clone, Debug, PartialEq)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

#[derive(Debug)]
struct Scope {
    names: HashMap<String, Ty>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (name, ty) in &self.names {
            write!(f, "{}: {}, ", name, ty)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOpKind {
    AddLike,
    ShiftLike,
    CompareLike,
    AddAssignLike,
    ShiftAssignLike,
}

// This struct represents a deferred unify operation
// associated with a binomial operator.
// So we have x = a + b, and we want to unify all
// three. At the time that the binomial operator
// is visited, we may not know enough about a or b to
// infer the type of x.  So we revisit in a queue afterwards.

#[derive(Debug, Clone)]
pub struct BinomialInference {
    op: BinaryOpKind,
    lhs: Ty,
    rhs: Ty,
    target: Ty,
}

#[derive(Default)]
pub struct TypeInference {
    scopes: Vec<Scope>,
    active_scope: ScopeId,
    context: UnifyContext,
    ret: Option<Ty>,
    deferred: Vec<BinomialInference>,
}

impl TypeInference {
    fn new_scope(&mut self) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
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
    fn unify(&mut self, lhs: Ty, rhs: Ty) -> Result<()> {
        eprintln!("Unifying {:?} = {:?}", lhs, rhs);
        if let Err(err) = self.context.unify(lhs, rhs) {
            bail!(
                "Type error: {}, active_scope: {:?}, scopes: {}",
                err,
                self.scopes[self.active_scope.0],
                self.scopes
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );
        } else {
            Ok(())
        }
    }
    fn process_deferred_unification(
        &mut self,
        deferred: BinomialInference,
    ) -> Result<Option<BinomialInference>> {
        let lhs = self.context.apply(deferred.lhs.clone());
        let rhs = self.context.apply(deferred.rhs.clone());
        let target = self.context.apply(deferred.target.clone());
        if lhs.is_variable() && rhs.is_variable() {
            return Ok(Some(deferred));
        }
        match deferred.op {
            BinaryOpKind::AddLike => {
                eprintln!(
                    "Processing deferred unification: {:?} + {:?} -> {:?}",
                    lhs, rhs, target
                );
                if lhs.is_signal() {
                    self.unify(target.clone(), lhs.clone())?;
                }
                if rhs.is_signal() {
                    self.unify(target.clone(), rhs.clone())?;
                }
                if !lhs.is_signal() && !rhs.is_signal() {
                    self.unify(target.clone(), lhs.clone())?;
                    self.unify(target, rhs.clone())?;
                }
            }
            BinaryOpKind::CompareLike => {
                eprintln!(
                    "Processing deferred unification: {:?} > {:?} -> {:?}",
                    lhs, rhs, target
                );
                if let Ty::Var(target_id) = target {
                    if lhs.is_signal() {
                        self.unify(target.clone(), ty_signal(ty_bool(), target_id))?;
                    }
                    if rhs.is_signal() {
                        self.unify(target.clone(), ty_signal(ty_bool(), target_id))?;
                    }
                    if !lhs.is_signal() && !rhs.is_signal() {
                        self.unify(lhs.clone(), rhs.clone())?;
                        self.unify(target, ty_bool())?;
                    }
                }
            }
            _ => {
                unimplemented!()
            } /*             BinaryOpKind::CompareLike => {
                             if lhs.is_signal() {
                                 self.unify(target, ty_signal(ty_bool()))?;
                             }
                             self.unify(deferred.lhs, deferred.rhs)?;
                             self.unify(deferred.target, ty_bool())?;
                         }
                         BinaryOpKind::AddAssignLike => {
                             self.unify(deferred.lhs, deferred.rhs)?;
                             self.unify(deferred.target, ty_empty())?;
                         }
                         BinaryOpKind::AndAssignLike => {
                             self.unify(deferred.lhs, ty_bool())?;
                             self.unify(deferred.rhs, ty_bool())?;
                             self.unify(deferred.target, ty_empty())?;
                         }
              */
        }
        Ok(None)
    }
    fn process_deferred_unifications(&mut self) -> Result<()> {
        if self.deferred.is_empty() {
            return Ok(());
        }
        eprintln!("Processing deferred unifications: {}", self.deferred.len());
        let mut new_deferred = vec![];
        for inference in self.deferred.clone() {
            if let Some(inference) = self.process_deferred_unification(inference)? {
                new_deferred.push(inference.clone());
            }
        }
        self.deferred = new_deferred;
        Ok(())
    }
    fn bind(&mut self, name: &str, id: NodeId) -> Result<()> {
        eprintln!("Binding {} to {:?}", name, id);
        self.scopes[self.active_scope.0]
            .names
            .insert(name.to_string(), id_to_var(id)?);
        Ok(())
    }
    fn cross_reference(&mut self, parent: Ty, child: Ty) -> Result<()> {
        eprintln!("Cross reference p{:?} = c{:?}", parent, child);
        if let (Ty::Var(parent), Ty::Var(child)) = (parent, child) {
            self.context.bind(parent, child);
            Ok(())
        } else {
            bail!("Cannot cross reference non-variables");
        }
    }
    fn lookup(&self, path: &str) -> Option<Ty> {
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

    fn bind_arm_pattern(&mut self, pat: &ast_impl::Pat) -> Result<()> {
        eprintln!("bind match pattern {:?}", pat);
        match &pat.kind {
            ast_impl::PatKind::TupleStruct(tuple_struct) => {
                for (elem, ty) in tuple_struct
                    .elems
                    .iter()
                    .zip(&tuple_struct.signature.arguments)
                {
                    self.bind_arm_pattern(elem)?;
                    self.unify(id_to_var(elem.id)?, ty.clone().into())?;
                }
            }
            ast_impl::PatKind::Struct(ty) => {
                let term = self.context.apply(id_to_var(pat.id)?);
                if let Ty::Struct(struct_ty) = term {
                    eprintln!("struct type is just a struct");
                    for field in &ty.fields {
                        if let Member::Named(name) = &field.member {
                            if let Some(ty) = struct_ty.fields.get(name) {
                                self.bind_arm_pattern(&field.pat)?;
                                self.unify(id_to_var(field.pat.id)?, ty.clone())?;
                            }
                        }
                    }
                }
            }
            ast_impl::PatKind::Ident(ident) => {
                self.bind(&ident.name, pat.id)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn bind_pattern(&mut self, pat: &ast_impl::Pat) -> Result<()> {
        eprintln!("bind pattern {:?}", pat);
        match &pat.kind {
            ast_impl::PatKind::Ident(ref ident) => {
                self.bind(&ident.name, pat.id)?;
            }
            ast_impl::PatKind::Tuple(ref tuple) => {
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
            ast_impl::PatKind::Slice(ref slice) => {
                let array_type = slice
                    .elems
                    .first()
                    .map(|x| id_to_var(x.id))
                    .transpose()?
                    .unwrap_or_else(ty_empty);
                for elem in slice.elems.iter() {
                    self.bind_pattern(elem)?;
                    self.unify(id_to_var(elem.id)?, array_type.clone())?;
                }
                self.unify(id_to_var(pat.id)?, ty_array(array_type, slice.elems.len()))?;
            }
            ast_impl::PatKind::Type(ref ty) => {
                self.bind_pattern(&ty.pat)?;
                self.unify(id_to_var(ty.pat.id)?, ty.kind.clone().into())?;
                self.unify(id_to_var(pat.id)?, id_to_var(ty.pat.id)?)?;
            }
            ast_impl::PatKind::Lit(ref _lit) => {}
            ast_impl::PatKind::Struct(ref ty) => {
                let term = self.context.apply(id_to_var(pat.id)?);
                eprintln!("Struct type: {term}");
                if let Ty::Struct(struct_ty) = term {
                    eprintln!("struct type is just a struct");
                    for field in &ty.fields {
                        if let Member::Named(name) = &field.member {
                            if let Some(ty) = struct_ty.fields.get(name) {
                                self.bind_pattern(&field.pat)?;
                                self.unify(id_to_var(field.pat.id)?, ty.clone())?;
                            }
                        }
                    }
                } else if let Ty::Enum(enum_ty) = term {
                    eprintln!("struct type is an enum");
                    if let Some(variant_name) = &ty.path.segments.last() {
                        eprintln!("variant name is {}", variant_name.ident);
                        if let Some(Ty::Struct(variant_ty)) =
                            enum_ty.payload.fields.get(&variant_name.ident.to_string())
                        {
                            eprintln!("variant has fields {:?}", variant_ty);
                            for field in &ty.fields {
                                if let Member::Named(name) = &field.member {
                                    if let Some(ty) = variant_ty.fields.get(name) {
                                        self.bind_pattern(&field.pat)?;
                                        self.unify(id_to_var(field.pat.id)?, ty.clone())?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ast_impl::PatKind::TupleStruct(ref ty) => {
                let term = self.context.apply(id_to_var(pat.id)?);
                eprintln!("Tuple Struct type: {term}");
                if let Ty::Enum(enum_ty) = term {
                    if let Some(variant_name) = &ty.path.segments.last() {
                        if let Some(Ty::Tuple(variant_ty)) =
                            enum_ty.payload.fields.get(&variant_name.ident.to_string())
                        {
                            if ty.elems.len() != variant_ty.len() {
                                bail!(
                                    "Wrong number of arguments to enum variant: {}",
                                    ty.elems.len()
                                );
                            }
                            for (elem, ty) in ty.elems.iter().zip(variant_ty) {
                                self.bind_pattern(elem)?;
                                self.unify(id_to_var(elem.id)?, ty.clone())?;
                            }
                        }
                    }
                } else {
                    self.unify(id_to_var(pat.id)?, ty.signature.ret.clone().into())?;
                    if ty.signature.arguments.len() != ty.elems.len() {
                        bail!(
                            "Wrong number of arguments to tuple struct: {}",
                            ty.elems.len()
                        );
                    }
                    for (elem, ty) in ty.elems.iter().zip(&ty.signature.arguments) {
                        self.bind_pattern(elem)?;
                        self.unify(id_to_var(elem.id)?, ty.clone().into())?;
                    }
                }
            }
            ast_impl::PatKind::Wild => {}
            ast_impl::PatKind::Path(_path) => {}
            _ => bail!("Unsupported pattern kind: {:?}", pat.kind),
        }
        Ok(())
    }
    fn handle_method_call(&mut self, my_ty: Ty, call: &ast_impl::ExprMethodCall) -> Result<()> {
        let target = self.context.apply(id_to_var(call.receiver.id)?);
        // We only support method calls on Bits and Signed for now.
        let method_name = &call.method;
        match method_name.as_str() {
            "set_bit" => {
                if let Ty::Const(ty::Bits::Unsigned(_len)) = target {
                    // Signature is set_bit(self, index: usize, value: bool) -> bits
                    if call.args.len() != 2 {
                        bail!("Wrong number of arguments to set_bit: {}", call.args.len());
                    }
                    self.unify(id_to_var(call.args[0].id)?, ty_usize())?;
                    self.unify(id_to_var(call.args[1].id)?, ty_bool())?;
                    self.unify(my_ty, ty_empty())?;
                }
            }
            "get_bit" => {
                if let Ty::Const(ty::Bits::Unsigned(_len)) = target {
                    // Signature is get_bit(self, index: usize) -> bool
                    if call.args.len() != 1 {
                        bail!("Wrong number of arguments to get_bit: {}", call.args.len());
                    }
                    self.unify(id_to_var(call.args[0].id)?, ty_usize())?;
                    self.unify(my_ty, ty_bool())?;
                }
            }
            "any" | "all" | "xor" => {
                if let Ty::Const(ty::Bits::Unsigned(_len)) = target {
                    self.unify(my_ty, ty_bool())?;
                }
            }
            "sign_bit" => {
                if let Ty::Const(ty::Bits::Signed(_len)) = target {
                    self.unify(my_ty, ty_bool())?;
                }
            }
            "slice" | "into" => {}
            "as_signed" => {
                if let Ty::Const(ty::Bits::Unsigned(len)) = target {
                    self.unify(my_ty, ty_signed(len))?;
                }
            }
            "as_unsigned" => {
                if let Ty::Const(ty::Bits::Signed(len)) = target {
                    self.unify(my_ty, ty_bits(len))?;
                }
            }
            "val" => {
                self.unify(my_ty, target)?;
            }
            _ => {
                bail!("Unsupported method call: {}", method_name);
            }
        }
        Ok(())
    }
    fn handle_call(&mut self, my_ty: Ty, call: &ast_impl::ExprCall) -> Result<()> {
        self.unify(my_ty, call.signature.ret.clone().into())?;
        if call.args.len() == call.signature.arguments.len() {
            for (arg, ty) in call.args.iter().zip(&call.signature.arguments) {
                self.unify(id_to_var(arg.id)?, ty.clone().into())?;
            }
        } else {
            bail!("Wrong number of arguments to function: {}", call.args.len());
        }
        Ok(())
    }
}

// Shortcut to allow us to reuse the node IDs as
// type variables in the resolver.
pub(super) fn id_to_var(id: ast_impl::NodeId) -> Result<Ty> {
    if id.is_invalid() {
        bail!("Invalid node ID");
    }
    Ok(ty_var(id.as_u32() as usize))
}

impl Visitor for TypeInference {
    fn visit_stmt(&mut self, node: &ast_impl::Stmt) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        if let ast_impl::StmtKind::Expr(expr) = &node.kind {
            self.unify(my_ty, id_to_var(expr.id)?)?;
        } else {
            self.unify(my_ty, ty_empty())?;
        }
        visit::visit_stmt(self, node)
    }
    fn visit_local(&mut self, node: &ast_impl::Local) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.unify(my_ty, ty_empty())?;
        if let Some(init) = node.init.as_ref() {
            self.unify(id_to_var(node.pat.id)?, id_to_var(init.id)?)?;
        }
        visit::visit_local(self, node)?;
        self.bind_pattern(&node.pat)
    }
    fn visit_block(&mut self, node: &ast_impl::Block) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.new_scope();
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
    fn visit_expr(&mut self, node: &ast_impl::Expr) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        match &node.kind {
            // x <- l + r --> tx = tl = tr
            ExprKind::Binary(ExprBinary {
                op:
                    BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::BitXor
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::And
                    | BinOp::Or,
                lhs,
                rhs,
            }) => {
                self.deferred.push(BinomialInference {
                    op: BinaryOpKind::AddLike,
                    lhs: id_to_var(lhs.id)?,
                    rhs: id_to_var(rhs.id)?,
                    target: my_ty.clone(),
                });
            }
            // x <- l << r --> tx = tl
            ExprKind::Binary(ExprBinary {
                op: BinOp::Shl | BinOp::Shr,
                lhs,
                rhs,
            }) => {
                self.deferred.push(BinomialInference {
                    op: BinaryOpKind::ShiftLike,
                    lhs: id_to_var(lhs.id)?,
                    rhs: id_to_var(rhs.id)?,
                    target: my_ty.clone(),
                });
            }
            // x <- (l <<= r) --> tx = {}
            ExprKind::Binary(ExprBinary {
                op: BinOp::ShlAssign | BinOp::ShrAssign,
                lhs,
                rhs,
            }) => {
                self.deferred.push(BinomialInference {
                    op: BinaryOpKind::ShiftAssignLike,
                    lhs: id_to_var(lhs.id)?,
                    rhs: id_to_var(rhs.id)?,
                    target: my_ty.clone(),
                });
            }
            // x <- l == r --> tx = bool, tl = tr
            ExprKind::Binary(ExprBinary {
                op: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge,
                lhs,
                rhs,
            }) => {
                self.deferred.push(BinomialInference {
                    op: BinaryOpKind::CompareLike,
                    lhs: id_to_var(lhs.id)?,
                    rhs: id_to_var(rhs.id)?,
                    target: my_ty.clone(),
                });
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
                self.deferred.push(BinomialInference {
                    op: BinaryOpKind::AddAssignLike,
                    lhs: id_to_var(lhs.id)?,
                    rhs: id_to_var(rhs.id)?,
                    target: my_ty.clone(),
                });
            }
            // x <- y = z --> tx = {}, ty = tz
            ExprKind::Assign(ExprAssign { lhs, rhs }) => {
                self.unify(id_to_var(lhs.id)?, id_to_var(rhs.id)?)?;
                self.unify(my_ty, ty_empty())?;
            }
            // x <- if c { t } else { e } --> tx = tt = te, tc = bool
            ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }) => {
                self.unify(id_to_var(cond.id)?, ty_bool())?;
                self.unify(my_ty.clone(), id_to_var(then_branch.id)?)?;
                if let Some(else_branch) = else_branch {
                    self.unify(my_ty, id_to_var(else_branch.id)?)?;
                }
            }
            ExprKind::Match(match_) => {
                // Unify the match target conditional with the type of the match arms
                let match_expr = id_to_var(match_.expr.id)?;
                for arm in &match_.arms {
                    self.unify(match_expr.clone(), id_to_var(arm.id)?)?;
                }
                // Unify the type of the match expression with the types of the
                // arm bodies
                for arm in &match_.arms {
                    self.unify(my_ty.clone(), id_to_var(arm.body.id)?)?;
                }
            }
            // x <- bits::<len>(y) --> tx = bits<len>
            ExprKind::Call(call) => {
                self.handle_call(my_ty, call)?;
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
                self.unify(my_ty, ty_array(array_type.clone(), array_len))?;
                for elem in &array.elems {
                    self.unify(id_to_var(elem.id)?, array_type.clone())?;
                }
            }
            ExprKind::Block(block) => {
                self.unify(my_ty, id_to_var(block.block.id)?)?;
            }
            ExprKind::Path(path) => {
                if path.path.segments.len() == 1 && path.path.segments[0].arguments.is_empty() {
                    let name = &path.path.segments[0].ident;
                    if let Some(ty) = self.lookup(name) {
                        self.unify(my_ty.clone(), ty.clone())?;
                        // Record a cross reference between the two type variables
                        // for later use in the compiler
                        self.cross_reference(ty, my_ty)?;
                    }
                }
            }
            ExprKind::Field(field) => {
                visit::visit_expr(self, node)?;
                let arg = id_to_var(field.expr.id)?;
                let sub = match field.member {
                    ast_impl::Member::Named(ref name) => self.context.get_named_field(arg, name),
                    ast_impl::Member::Unnamed(ref index) => {
                        self.context.get_unnamed_field(arg, *index as usize)
                    }
                }?;
                self.unify(my_ty, sub)?;
            }
            ExprKind::Struct(struct_) => {
                self.unify(my_ty, struct_.template.kind.clone().into())?;
                if let Some(s_kind) = match &struct_.variant {
                    Kind::Struct(s) => Some(s),
                    _ => None,
                } {
                    for field in struct_.fields.iter() {
                        if let Member::Named(name) = &field.member {
                            if let Some(k_field) = s_kind.fields.iter().find(|x| &x.name == name) {
                                self.unify(
                                    id_to_var(field.value.id)?,
                                    k_field.kind.clone().into(),
                                )?;
                            }
                        }
                    }
                }
            }
            ExprKind::Index(index) => {
                visit::visit_expr(self, node)?;
                let arg = id_to_var(index.expr.id)?;
                self.unify(my_ty, self.context.get_array_base(arg)?)?;
            }
            ExprKind::Ret(ret) => {
                if let Some(expr) = ret.expr.as_ref() {
                    self.unify(my_ty.clone(), id_to_var(expr.id)?)?;
                } else {
                    self.unify(my_ty.clone(), ty_empty())?;
                }
                if let Some(ret) = &self.ret {
                    self.unify(my_ty, ret.clone())?;
                }
            }
            ExprKind::Paren(paren) => {
                self.unify(my_ty, id_to_var(paren.expr.id)?)?;
            }
            ExprKind::Group(group) => {
                self.unify(my_ty, id_to_var(group.expr.id)?)?;
            }
            // x <- +/- y --> tx = ty
            ExprKind::Unary(ExprUnary { op: _, expr }) => {
                self.unify(my_ty, id_to_var(expr.id)?)?;
            }
            ExprKind::Lit(lit) => match lit {
                // We apply the generic integer type to all integer
                // literals only _after_ all other steps of type inference.
                // This is to allow other constraints to be applied first.
                //
                // For example, if we have y = bits(3), then 3 is an
                // integer literal, but the `bits` function constrains
                // it to be a u128.
                //
                // On the other hand, expressions like `if 3 > 4` do not
                // apply any strong type constraints.  These are under-constrained
                // literals, and Rust will automatically assume they are i32.
                // We do likewise, but in a separate inference pass.
                //
                ExprLit::Int(_) => {
                    // self.unify(my_ty, ty_integer())?;
                }
                ExprLit::Bool(_) => {
                    self.unify(my_ty, ty_bool())?;
                }
                ExprLit::TypedBits(b) => {
                    self.unify(my_ty, b.value.kind.clone().into())?;
                }
            },
            ExprKind::Repeat(repeat) => {
                //self.unify(id_to_var(repeat.len.id)?, ty_usize())?;
                /*
                if let ExprKind::Lit(ExprLit::Int(len)) = &repeat.len.kind {
                    if let Ok(len) = len.parse::<usize>() {
                        self.unify(my_ty, ty_array(id_to_var(repeat.value.id)?, len))?;
                    }
                }
                */
            }
            ExprKind::MethodCall(call) => {
                self.handle_method_call(my_ty, call)?;
            }
            ExprKind::ForLoop(for_loop) => {
                self.new_scope();
                self.bind_pattern(&for_loop.pat)?;
                self.unify(my_ty, ty_empty())?;
                visit::visit_expr(self, node)?;
                self.end_scope();
            }
            ExprKind::Range(range) => {
                if let Some(start) = range.start.as_ref() {
                    self.unify(my_ty.clone(), id_to_var(start.id)?)?;
                }
                if let Some(end) = range.end.as_ref() {
                    self.unify(my_ty, id_to_var(end.id)?)?;
                }
            }
            _ => todo!("{:?}", node.kind),
        }
        visit::visit_expr(self, node)
    }
    fn visit_match_arm(&mut self, node: &ast_impl::Arm) -> Result<()> {
        eprintln!("match arm visit - create new scope");
        self.new_scope();
        if let ArmKind::Enum(arm_enum) = &node.kind {
            self.unify(
                id_to_var(arm_enum.pat.id)?,
                arm_enum.payload_kind.clone().into(),
            )?;
            eprintln!("arm pattern binding");
            self.bind_arm_pattern(&arm_enum.pat)?;
        }
        eprintln!("handle body");
        visit::visit_match_arm(self, node)?;
        eprintln!("end scope");
        self.end_scope();
        Ok(())
    }
    fn visit_kernel_fn(&mut self, node: &ast_impl::KernelFn) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        self.unify(my_ty, node.ret.clone().into())?;
        self.ret = Some(node.ret.clone().into());
        self.new_scope();
        for pat in &node.inputs {
            self.bind_pattern(pat)?;
        }
        self.unify(id_to_var(node.body.id)?, node.ret.clone().into())?;
        visit::visit_kernel_fn(self, node)?;
        self.end_scope();
        Ok(())
    }
}

struct InferenceForGenericIntegers<'a> {
    context: &'a mut UnifyContext,
}

impl<'a> Visitor for InferenceForGenericIntegers<'a> {
    fn visit_expr(&mut self, node: &ast_impl::Expr) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        let resolved_type = self.context.apply(my_ty.clone());
        if let ExprKind::Lit(lit) = &node.kind {
            if let (Ty::Var(_), ExprLit::Int(_)) = (resolved_type, lit) {
                self.context.unify(my_ty, ty_integer())?;
            }
        }
        visit::visit_expr(self, node)
    }
    fn visit_pat(&mut self, node: &ast_impl::Pat) -> Result<()> {
        let my_ty = id_to_var(node.id)?;
        let resolved_type = self.context.apply(my_ty.clone());
        if let ast_impl::PatKind::Lit(lit) = &node.kind {
            if let (Ty::Var(_), ExprLit::Int(_)) = (resolved_type, lit.lit.as_ref()) {
                self.context.unify(my_ty, ty_integer())?;
            }
        }
        visit::visit_pat(self, node)
    }
}

pub fn infer(root: &Kernel) -> Result<UnifyContext> {
    let mut inference_engine = TypeInference::default();
    inference_engine.visit_kernel_fn(root.inner())?;
    eprintln!("defer count {:?}", inference_engine.deferred.len());
    let mut integer_fixup = InferenceForGenericIntegers {
        context: &mut inference_engine.context,
    };
    integer_fixup.visit_kernel_fn(root.inner())?;
    let mut previous_number = 0;
    while previous_number != inference_engine.deferred.len() {
        previous_number = inference_engine.deferred.len();
        inference_engine.process_deferred_unifications()?;
    }
    if !inference_engine.deferred.is_empty() {
        bail!(
            "Failed to resolve all type constraints {:?}",
            inference_engine.deferred
        );
    }
    Ok(inference_engine.context)
}
