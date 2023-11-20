use crate::ast::{ExprAssign, ExprIf, ExprLit, ExprUnary, Member, NodeId, PatKind};
use crate::kernel::Kernel;
use crate::ty::{ty_array, ty_bits, ty_bool, ty_empty, ty_signed, ty_tuple, TyMap};
use crate::unify::UnifyContext;
use crate::Kind;
use crate::{
    ast::{self, BinOp, ExprBinary, ExprKind},
    ty::{ty_var, Ty},
    visit::{self, Visitor},
};
use anyhow::{bail, Result};
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
    names: HashMap<String, Ty>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

#[derive(Default)]
pub struct TypeInference {
    scopes: Vec<Scope>,
    active_scope: ScopeId,
    context: UnifyContext,
    structs: HashMap<String, Ty>,
    enums: HashMap<String, Ty>,
    ret: Option<Ty>,
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
    fn flat_path(path: &ast::Path) -> String {
        path.segments
            .iter()
            .map(|x| x.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
    pub fn define_kind(&mut self, kind: Kind) -> Result<()> {
        match &kind {
            Kind::Struct(struct_) => {
                self.structs.insert(struct_.name.clone(), kind.into());
            }
            Kind::Enum(enum_) => {
                self.enums.insert(enum_.name.clone(), kind.into());
            }
            _ => {}
        }
        Ok(())
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
    // Given a path of the form a::b::c::d --> "a::b::c"
    fn flat_path_of_parent(path: &ast::Path) -> String {
        path.segments
            .iter()
            .take(path.segments.len() - 1)
            .map(|x| x.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
    fn lookup_enum_unit_variant(&self, path: &ast::Path) -> Option<Ty> {
        let last_segment = path.segments.iter().last()?;
        // Collect all but the last segment into a '::' separated string
        let path = Self::flat_path_of_parent(path);
        if let Some(Ty::Enum(ty)) = self.enums.get(&path) {
            // Check to see if the last segment is a unit variant
            if let Some(Ty::Const(crate::ty::Bits::Empty)) = ty.fields.get(&last_segment.ident) {
                return Some(Ty::Enum(ty.clone()));
            }
        }
        None
    }
    // Look for a tuple variant, i.e., A::Variant(x, y, z), and if so,
    // return the type of the variant, and the type for the tuple of arguments.
    fn lookup_enum_tuple_variant(&self, path: &ast::Path) -> Option<(Ty, Vec<Ty>)> {
        let last_segment = path.segments.iter().last()?;
        let path = Self::flat_path_of_parent(path);
        if let Some(Ty::Enum(ty)) = self.enums.get(&path) {
            // Check to see if the last segment is a tuple struct variant
            if let Some(Ty::Tuple(fields)) = ty.fields.get(&last_segment.ident).cloned() {
                return Some((Ty::Enum(ty.clone()), fields));
            }
        }
        None
    }
    // Look for a struct variant, i.e., A::Variant{x: x, y: y, z: z}, and if so,
    // return the type of the variant.  Also return the fields of the variant.
    fn lookup_enum_struct_variant(&self, path: &ast::Path) -> Option<(Ty, TyMap)> {
        let last_segment = path.segments.iter().last()?;
        let path = Self::flat_path_of_parent(path);
        if let Some(Ty::Enum(ty)) = self.enums.get(&path) {
            // Check to see if the last segment is a tuple struct variant
            if let Some(Ty::Struct(fields)) = ty.fields.get(&last_segment.ident).cloned() {
                return Some((Ty::Enum(ty.clone()), fields));
            }
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
                    self.unify(id_to_var(elem.id)?, array_type.clone())?;
                }
                self.unify(id_to_var(pat.id)?, ty_array(array_type, slice.elems.len()))?;
            }
            ast::PatKind::Type(ref ty) => {
                self.bind_pattern(&ty.pat)?;
                self.unify(id_to_var(ty.pat.id)?, ty.kind.clone().into())?;
                self.unify(id_to_var(pat.id)?, id_to_var(ty.pat.id)?)?;
            }
            ast::PatKind::Struct(ref ty) => {
                if let Some(Ty::Struct(struct_ty)) =
                    self.structs.get(&Self::flat_path(&ty.path)).cloned()
                {
                    for field in &ty.fields {
                        if let Member::Named(name) = &field.member {
                            if let Some(ty) = struct_ty.fields.get(name) {
                                self.bind_pattern(&field.pat)?;
                                self.unify(id_to_var(field.pat.id)?, ty.clone())?;
                            }
                        }
                    }
                    self.unify(id_to_var(pat.id)?, Ty::Struct(struct_ty.clone()))?;
                } else if let Some((enum_ty, variant_ty)) =
                    self.lookup_enum_struct_variant(&ty.path)
                {
                    for field in &ty.fields {
                        if let Member::Named(name) = &field.member {
                            if let Some(ty) = variant_ty.fields.get(name) {
                                self.bind_pattern(&field.pat)?;
                                self.unify(id_to_var(field.pat.id)?, ty.clone())?;
                            }
                        }
                    }
                    self.unify(id_to_var(pat.id)?, enum_ty)?;
                }
            }
            ast::PatKind::TupleStruct(ref ty) => {
                if let Some((enum_ty, variant_ty)) = self.lookup_enum_tuple_variant(&ty.path) {
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
                    self.unify(id_to_var(pat.id)?, enum_ty)?;
                }
            }
            ast::PatKind::Wild => {}
            ast::PatKind::Path(path) => {
                if let Some(ty) = self.lookup_enum_unit_variant(&path.path) {
                    self.unify(id_to_var(pat.id)?, ty)?;
                }
            }
            _ => bail!("Unsupported pattern kind: {:?}", pat.kind),
        }
        Ok(())
    }
    pub fn infer(mut self, root: &Kernel) -> Result<UnifyContext> {
        self.visit_kernel_fn(&root.ast)?;
        self.visit_kernel_fn(&root.ast)?;
        Ok(self.context)
    }
    fn handle_method_call(&mut self, my_ty: Ty, call: &ast::ExprMethodCall) -> Result<()> {
        let target = self.context.apply(id_to_var(call.receiver.id)?);
        // We only support method calls on Bits and Signed for now.
        let method_name = &call.method;
        match method_name.as_str() {
            "set_bit" => {
                // Signature is set_bit(self, index: usize, value: bool) -> bits
                if call.args.len() != 2 {
                    bail!("Wrong number of arguments to set_bit: {}", call.args.len());
                }
                self.unify(id_to_var(call.args[1].id)?, ty_bool())?;
                self.unify(my_ty, ty_empty())?;
            }
            "get_bit" => {
                // Signature is get_bit(self, index: usize) -> bool
                if call.args.len() != 1 {
                    bail!("Wrong number of arguments to get_bit: {}", call.args.len());
                }
                self.unify(my_ty, ty_bool())?;
            }
            "any" | "all" | "xor" | "sign_bit" | "is_negative" | "is_non_negative" => {
                self.unify(my_ty, ty_bool())?;
            }
            "slice" | "into" => {}
            "as_signed" => {
                if let Ty::Const(crate::ty::Bits::Unsigned(len)) = target {
                    self.unify(my_ty, ty_signed(len))?;
                }
            }
            "as_unsigned" => {
                if let Ty::Const(crate::ty::Bits::Signed(len)) = target {
                    self.unify(my_ty, ty_bits(len))?;
                }
            }
            _ => {
                bail!("Unsupported method call: {}", method_name);
            }
        }
        Ok(())
    }
    fn handle_call(&mut self, my_ty: Ty, call: &ast::ExprCall) -> Result<()> {
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
            self.unify(id_to_var(node.pat.id)?, id_to_var(init.id)?)?;
        }
        visit::visit_local(self, node)?;
        self.bind_pattern(&node.pat)
    }
    fn visit_block(&mut self, node: &ast::Block) -> Result<()> {
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
            // x <- (l <<= r) --> tx = {}
            ExprKind::Binary(ExprBinary {
                op: BinOp::ShlAssign | BinOp::ShrAssign,
                ..
            }) => {
                self.unify(my_ty, ty_empty())?;
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
                    self.unify(match_expr.clone(), id_to_var(arm.pattern.id)?)?;
                }
                // Unify the type of the match expression with the types of the
                // arm bodies
                for arm in &match_.arms {
                    self.unify(my_ty.clone(), id_to_var(arm.body.id)?)?;
                }
            }
            // x <- bits::<len>(y) --> tx = bits<len>
            // TODO - make this extensible and not gross.
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
                        self.unify(my_ty, ty.clone())?;
                    }
                } else {
                    // Check for the case of enum unit variant
                    if let Some(ty) = self.lookup_enum_unit_variant(&path.path) {
                        self.unify(my_ty, ty.clone())?;
                    }
                }
                // TODO - handle more complex paths
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
            ExprKind::Struct(struct_) => {
                self.unify(my_ty, struct_.kind.clone().into())?;
                if let Some(s_kind) = match &struct_.kind {
                    Kind::Struct(s) => Some(s),
                    Kind::Variant(_, v) => match &v.kind {
                        Kind::Struct(s) => Some(s),
                        _ => None,
                    },
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
                ExprLit::Int(_) => {
                    //self.unify(my_ty, ty_uint())?;
                }
                ExprLit::Bool(_) => {
                    self.unify(my_ty, ty_bool())?;
                }
                ExprLit::TypedBits(b) => {
                    self.unify(my_ty, b.value.kind.clone().into())?;
                }
            },
            ExprKind::Repeat(repeat) => {
                if let ExprKind::Lit(ExprLit::Int(len)) = &repeat.len.kind {
                    if let Ok(len) = len.parse::<usize>() {
                        self.unify(my_ty, ty_array(id_to_var(repeat.value.id)?, len))?;
                    }
                }
            }
            ExprKind::MethodCall(call) => {
                self.handle_method_call(my_ty, call)?;
            }
            _ => {}
        }
        visit::visit_expr(self, node)
    }
    fn visit_match_arm(&mut self, node: &ast::Arm) -> Result<()> {
        self.new_scope();
        self.bind_pattern(&node.pattern)?;
        visit::visit_match_arm(self, node)?;
        self.end_scope();
        Ok(())
    }
    fn visit_kernel_fn(&mut self, node: &ast::KernelFn) -> Result<()> {
        for arg in &node.inputs {
            if let PatKind::Type(ty) = &arg.kind {
                self.define_kind(ty.kind.clone())?;
            }
        }
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
