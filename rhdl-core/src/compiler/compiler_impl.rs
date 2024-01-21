use crate::{
    ast::ast_impl::{
        self, ArmKind, BinOp, Expr, ExprBinary, ExprIf, ExprKind, ExprLit, ExprTuple,
        ExprTypedBits, FieldValue, FunctionId, Local, NodeId, Pat, PatKind, Path, UnOp,
        INVALID_NODE_ID,
    },
    ast::display_ast::pretty_print_statement,
    ast::visit::Visitor,
    compiler::ty::{ty_empty, ty_indexed_item, ty_named_field, ty_unnamed_field, Bits, Ty, TypeId},
    compiler::UnifyContext,
    rhif::rhif_builder::{
        op_array, op_as_bits, op_as_signed, op_assign, op_binary, op_case, op_comment,
        op_discriminant, op_enum, op_exec, op_index, op_repeat, op_select, op_splice, op_struct,
        op_tuple, op_unary,
    },
    rhif::spec::{
        self, AluBinary, AluUnary, CaseArgument, ExternalFunction, FuncId, Member, OpCode, Slot,
    },
    rhif::Object,
    types::typed_bits::TypedBits,
    Digital, KernelFnKind, Kind,
};
use anyhow::{anyhow, bail, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::description::*;
use super::{description::Description, infer_types::id_to_var, ty::ty_bool};

const EARLY_RETURN_FLAG_NODE: NodeId = NodeId::new(!0);

impl From<ast_impl::Member> for spec::Member {
    fn from(member: ast_impl::Member) -> Self {
        match member {
            ast_impl::Member::Named(name) => spec::Member::Named(name),
            ast_impl::Member::Unnamed(index) => spec::Member::Unnamed(index),
        }
    }
}

type LocalsMap = HashMap<TypeId, Slot>;

#[derive(Debug, Clone)]
pub struct Rebind {
    from: Slot,
    to: Slot,
}

pub struct CompilerContext {
    pub ops: Vec<OpCode>,
    pub literals: Vec<ast_impl::ExprLit>,
    pub reg_count: usize,
    type_context: UnifyContext,
    ty: BTreeMap<Slot, Ty>,
    descriptions: HashMap<Slot, Description>,
    locals: LocalsMap,
    stash: Vec<ExternalFunction>,
    return_node: NodeId,
    arguments: Vec<Slot>,
    fn_id: FunctionId,
    name: String,
}

impl std::fmt::Display for CompilerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Kernel {} ({})", self.name, self.fn_id)?;
        for regs in self.ty.keys() {
            writeln!(f, "Reg r{} : {}", regs.reg().unwrap(), self.ty[regs],)?;
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(
                f,
                "Literal l{} : {} = {:?}",
                ndx,
                self.ty[&Slot::Literal(ndx)],
                literal
            )?;
        }
        for (ndx, func) in self.stash.iter().enumerate() {
            writeln!(
                f,
                "Function f{} name: {} code: {} signature: {}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        for op in &self.ops {
            writeln!(f, "  {}", op)?;
        }
        Ok(())
    }
}

fn collapse_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

impl CompilerContext {
    fn new(type_context: UnifyContext) -> Self {
        Self {
            literals: vec![],
            reg_count: 0,
            type_context,
            ty: [(Slot::Empty, ty_empty())].into_iter().collect(),
            locals: Default::default(),
            descriptions: Default::default(),
            stash: Default::default(),
            return_node: INVALID_NODE_ID,
            ops: Default::default(),
            arguments: Default::default(),
            fn_id: Default::default(),
            name: Default::default(),
        }
    }
    fn node_ty(&self, id: NodeId) -> Result<Ty> {
        let var = id_to_var(id)?;
        Ok(self.type_context.apply(var))
    }
    fn reg(&mut self, ty: Ty, description: Description) -> Result<Slot> {
        if ty.is_empty() {
            return Ok(Slot::Empty);
        }
        let reg = Slot::Register(self.reg_count);
        self.ty.insert(reg, ty);
        self.descriptions.insert(reg, description);
        self.reg_count += 1;
        Ok(reg)
    }
    fn stash(&mut self, func: ExternalFunction) -> Result<FuncId> {
        let ndx = self.stash.len();
        self.stash.push(func);
        Ok(FuncId(ndx))
    }
    fn literal_from_type_and_int(&mut self, ty: &Ty, value: i32) -> Result<Slot> {
        let typed_bits = match ty {
            Ty::Const(Bits::U128) => {
                let x: u128 = value.try_into()?;
                x.typed_bits()
            }
            Ty::Const(Bits::I128) => {
                let x: i128 = value.into();
                x.typed_bits()
            }
            Ty::Const(Bits::Unsigned(n)) => {
                let x: u128 = value.try_into()?;
                x.typed_bits().unsigned_cast(*n)?
            }
            Ty::Const(Bits::Signed(n)) => {
                let x: i128 = value.into();
                x.typed_bits().signed_cast(*n)?
            }
            Ty::Const(Bits::Usize) => {
                let x: usize = value.try_into()?;
                x.typed_bits()
            }
            _ => {
                bail!("Unsupported literal type {:?} - most likely this means your for loop index variable is of an unexpected type", ty)
            }
        };
        self.literal_from_typed_bits(&typed_bits)
    }
    fn literal_from_typed_bits(&mut self, value: &TypedBits) -> Result<Slot> {
        let ndx = self.literals.len();
        let ty = value.kind.clone().into();
        self.literals
            .push(ast_impl::ExprLit::TypedBits(ExprTypedBits {
                path: Box::new(Path { segments: vec![] }),
                value: value.clone(),
            }));
        self.ty.insert(Slot::Literal(ndx), ty);
        Ok(Slot::Literal(ndx))
    }
    fn op(&mut self, op: OpCode) {
        self.ops.push(op);
    }
    fn slot_to_index(&self, slot: Slot) -> Result<usize> {
        let Slot::Literal(ndx) = slot else {
            bail!("ICE - index is not a literal")
        };
        let ndx = match &self.literals[ndx] {
            ExprLit::TypedBits(tb) => tb.value.as_i64()? as usize,
            ExprLit::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            ExprLit::Int(i) => i.parse::<usize>()?,
        };
        Ok(ndx)
    }
    fn ty(&self, id: NodeId) -> Result<Ty> {
        let var = id_to_var(id)?;
        Ok(self.type_context.apply(var))
    }
    // Create a local variable binding based on the
    // type information in the given node.  It will
    // be referred to by cross references in the type
    // context.
    fn bind(&mut self, id: NodeId, name: &str) -> Result<()> {
        let ty = self.ty(id)?;
        let reg = self.reg(ty, describe_local_binding(name))?;
        eprintln!("Binding {name}#{id} -> {reg}");
        if self.locals.insert(id.into(), reg).is_some() {
            bail!("Duplicate local variable binding for {:?}", id)
        }
        Ok(())
    }
    fn desc(&self, slot: Slot) -> Result<&Description> {
        self.descriptions
            .get(&slot)
            .ok_or(anyhow::anyhow!("ICE - No description for {:?}", slot))
    }
    fn rebind(&mut self, id: NodeId, description: &Description) -> Result<Rebind> {
        let ty = self.ty(id)?;
        let Some(prev) = self.locals.get(&id.into()).copied() else {
            bail!("No local variable binding for {:?}", id)
        };
        let reg = self.reg(ty, describe_rebinding(description))?;
        eprintln!("Rebinding {prev} -> {reg}");

        self.locals.insert(id.into(), reg);
        Ok(Rebind {
            from: prev,
            to: reg,
        })
    }
    fn resolve_parent(&mut self, child: NodeId) -> Result<Slot> {
        let child_id = id_to_var(child)?;
        if let Ty::Var(child_id) = child_id {
            if let Some(parent) = self.type_context.get_parent(child_id) {
                return self.locals.get(&parent).cloned().ok_or(anyhow::anyhow!(
                    "No local variable for {:?} defined when needed",
                    child_id,
                ));
            }
        }
        bail!("No parent for {:?}", child_id)
    }
    fn resolve_local(&mut self, id: NodeId) -> Result<Slot> {
        self.locals
            .get(&id.into())
            .cloned()
            .ok_or(anyhow::anyhow!("No local variable for {:?}", id))
    }
    fn unop(
        &mut self,
        id: NodeId,
        unary: &ast_impl::ExprUnary,
        desc: &Description,
    ) -> Result<Slot> {
        let arg = self.expr(
            &unary.expr,
            &Description::UnopArgument(unary.op, desc.clone().into()),
        )?;
        let result = self.reg(
            self.node_ty(id)?,
            Description::UnopTarget(unary.op, desc.clone().into()),
        )?;
        let op = match unary.op {
            ast_impl::UnOp::Neg => AluUnary::Neg,
            ast_impl::UnOp::Not => AluUnary::Not,
        };
        self.op(op_unary(op, result, arg));
        Ok(result)
    }
    fn stmt(&mut self, statement: &ast_impl::Stmt, description: &Description) -> Result<Slot> {
        let statement_text = pretty_print_statement(statement, &self.type_context)?;
        self.op(op_comment(statement_text));
        let description = Description::Statement(statement_text, description.clone().into());
        match &statement.kind {
            ast_impl::StmtKind::Local(local) => {
                self.local(local, &description)?;
                Ok(Slot::Empty)
            }
            ast_impl::StmtKind::Expr(expr) => self.expr(expr, &description),
            ast_impl::StmtKind::Semi(expr) => {
                self.expr(expr, &description)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn initialize_local(&mut self, pat: &Pat, rhs: Slot, description: &Description) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(_ident) => {
                if let Ok(lhs) = self.resolve_local(pat.id) {
                    self.op(op_assign(lhs, rhs));
                }
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                let rhs_ty = self
                    .ty
                    .get(&rhs)
                    .ok_or(anyhow::anyhow!(
                        "No type for {:?} when initializing tuple",
                        rhs
                    ))?
                    .clone();
                for (ndx, pat) in tuple.elements.iter().enumerate() {
                    let element_ty = ty_unnamed_field(&rhs_ty, ndx)?;
                    let element_rhs = self.reg(
                        element_ty,
                        Description::TupleInitLeftHandSide(ndx, description.clone().into()),
                    )?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(
                        pat,
                        element_rhs,
                        &Description::TupleInitRightHandSide(ndx, description.clone().into()),
                    )?;
                }
                Ok(())
            }
            PatKind::Struct(_struct) => {
                let rhs_ty = self
                    .ty
                    .get(&rhs)
                    .ok_or(anyhow::anyhow!(
                        "No type for {:?} when initializing struct",
                        rhs
                    ))?
                    .clone();
                for field in &_struct.fields {
                    let element_ty = match &field.member {
                        ast_impl::Member::Named(name) => ty_named_field(&rhs_ty, name)?,
                        ast_impl::Member::Unnamed(ndx) => ty_unnamed_field(&rhs_ty, *ndx as usize)?,
                    };
                    let element_rhs = self.reg(
                        element_ty,
                        Description::StructInitLeftHandSide(
                            field.member.clone(),
                            description.clone().into(),
                        ),
                    )?;
                    let path = field.member.clone().into();
                    self.op(op_index(element_rhs, rhs, path));
                    self.initialize_local(
                        &field.pat,
                        element_rhs,
                        &Description::StructInitRightHandSide(
                            field.member.clone(),
                            description.clone().into(),
                        ),
                    )?;
                }
                Ok(())
            }
            PatKind::TupleStruct(_tuple_struct) => {
                let rhs_ty = self
                    .ty
                    .get(&rhs)
                    .ok_or(anyhow::anyhow!(
                        "No type for {:?} when initializing tuple struct",
                        rhs
                    ))?
                    .clone();
                for (ndx, pat) in _tuple_struct.elems.iter().enumerate() {
                    let element_ty = ty_unnamed_field(&rhs_ty, ndx)?;
                    let element_rhs = self.reg(
                        element_ty,
                        Description::TupleStructInitLeftHandSide(ndx, description.clone().into()),
                    )?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(
                        pat,
                        element_rhs,
                        &Description::TupleStructInitRightHandSide(ndx, description.clone().into()),
                    )?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                let rhs_ty = self
                    .ty
                    .get(&rhs)
                    .ok_or(anyhow::anyhow!(
                        "No type for {:?} when initializing slice",
                        rhs
                    ))?
                    .clone();
                for (ndx, pat) in slice.elems.iter().enumerate() {
                    let element_ty = ty_indexed_item(&rhs_ty, ndx)?;
                    let element_rhs = self.reg(
                        element_ty,
                        Description::SliceInitLeftHandSide(ndx, description.clone().into()),
                    )?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(
                        pat,
                        element_rhs,
                        &Description::SliceInitRightHandSide(ndx, description.clone().into()),
                    )?;
                }
                Ok(())
            }
            PatKind::Type(ty) => self.initialize_local(&ty.pat, rhs, description),
            PatKind::Wild | PatKind::Lit(_) | PatKind::Path(_) => Ok(()),
            _ => todo!("Unsupported let init pattern: {:?}", pat),
        }
    }
    fn local(&mut self, local: &Local, description: &Description) -> Result<()> {
        self.bind_pattern(&local.pat)?;
        if let Some(init) = &local.init {
            let rhs = self.expr(init, description)?;
            self.initialize_local(&local.pat, rhs, description)?;
        }
        Ok(())
    }
    fn block(
        &mut self,
        block_result: Slot,
        block: &ast_impl::Block,
        description: &Description,
    ) -> Result<()> {
        let statement_count = block.stmts.len();
        for (ndx, statement) in block.stmts.iter().enumerate() {
            let is_last = ndx == statement_count - 1;
            let result = self.stmt(statement, description)?;
            if is_last && (block_result != result) {
                self.op(op_assign(block_result, result));
            }
        }
        Ok(())
    }
    fn expr_list(&mut self, exprs: &[Box<Expr>], description: &Description) -> Result<Vec<Slot>> {
        exprs
            .iter()
            .enumerate()
            .map(|(ndx, x)| {
                self.expr(
                    x,
                    &Description::ExpressionListItem(ndx, description.clone().into()),
                )
            })
            .collect::<Result<_>>()
    }
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple, description: &Description) -> Result<Slot> {
        let result = self.reg(
            self.node_ty(id)?,
            Description::TupleLeftHandSide(description.clone().into()),
        )?;
        let fields = self.expr_list(
            &tuple.elements,
            &Description::TupleRightHandSide(description.clone().into()),
        )?;
        self.op(op_tuple(result, fields));
        Ok(result)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf, description: &Description) -> Result<Slot> {
        let op_result = self.reg(
            self.node_ty(id)?,
            Description::IfStatementResult(description.clone().into()),
        )?;
        let then_result = self.reg(
            self.node_ty(id)?,
            Description::IfStatementResultOfThenBranch(description.clone().into()),
        )?;
        let else_result = self.reg(
            self.node_ty(id)?,
            Description::IfStatementResultOfElseBranch(description.clone().into()),
        )?;
        let cond = self.expr(
            &if_expr.cond,
            &Description::IfStatementCondition(description.clone().into()),
        )?;
        let locals_prior_to_branch = self.locals.clone();
        eprintln!("Locals prior to branch {:?}", locals_prior_to_branch);
        self.block(
            then_result,
            &if_expr.then_branch,
            &Description::IfStatementThenBranch(description.clone().into()),
        )?;
        let locals_after_then_branch = self.locals.clone();
        eprintln!("Locals after then branch {:?}", locals_after_then_branch);
        self.locals = locals_prior_to_branch.clone();
        if let Some(expr) = if_expr.else_branch.as_ref() {
            self.wrap_expr_in_block(
                else_result,
                expr,
                &Description::IfStatementElseBranch(description.clone().into()),
            )?;
        }
        let locals_after_else_branch = self.locals.clone();
        self.locals = locals_prior_to_branch.clone();
        // Linearize the if statement.
        // TODO - For now, inline this logic, but ultimately, we want
        // to be able to generalize to remove the `case` op.
        let mut rebound_locals =
            get_locals_changed(&locals_prior_to_branch, &locals_after_then_branch)?;
        rebound_locals.extend(get_locals_changed(
            &locals_prior_to_branch,
            &locals_after_else_branch,
        )?);
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<TypeId, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind((*x).into()).map(|r| (*x, r)))
            .collect::<Result<_>>()?;
        eprintln!("post_branch bindings set {:?}", post_branch_bindings);
        for (var, rebind) in &post_branch_bindings {
            let then_binding = *locals_after_then_branch.get(var).ok_or(anyhow!(
                "ICE - no local var found for binding {var:?} in then branch"
            ))?;
            let else_binding = *locals_after_else_branch.get(var).ok_or(anyhow!(
                "ICE - no local var found for binding {var:?} in else branch"
            ))?;
            let new_binding = rebind.to;
            self.op(op_select(new_binding, cond, then_binding, else_binding));
        }
        self.op(op_select(op_result, cond, then_result, else_result));
        Ok(op_result)
    }
    fn index(
        &mut self,
        id: NodeId,
        index: &ast_impl::ExprIndex,
        description: &Description,
    ) -> Result<Slot> {
        let lhs = self.reg(
            self.node_ty(id)?,
            Description::IndexExpressionLeftHandSide(description.clone().into()),
        )?;
        let arg = self.expr(
            &index.expr,
            &Description::IndexExpressionTarget(description.clone().into()),
        )?;
        let index = self.expr(
            &index.index,
            &Description::IndexExpressionIndex(description.clone().into()),
        )?;
        if index.is_literal() {
            let ndx = self.slot_to_index(index)?;
            self.op(op_index(lhs, arg, crate::path::Path::default().index(ndx)));
        } else {
            self.op(op_index(
                lhs,
                arg,
                crate::path::Path::default().dynamic(index),
            ));
        }
        Ok(lhs)
    }
    fn array(
        &mut self,
        id: NodeId,
        array: &ast_impl::ExprArray,
        description: &Description,
    ) -> Result<Slot> {
        let lhs = self.reg(
            self.node_ty(id)?,
            Description::ArrayExpressionLeftHandSide(description.clone().into()),
        )?;
        let elements = self.expr_list(
            &array.elems,
            &Description::ArrayExpressionRightHandSide(description.clone().into()),
        )?;
        self.op(op_array(lhs, elements));
        Ok(lhs)
    }
    fn field(
        &mut self,
        id: NodeId,
        field: &ast_impl::ExprField,
        description: &Description,
    ) -> Result<Slot> {
        let lhs = self.reg(
            self.node_ty(id)?,
            Description::StructFieldExpressionLeftHandSide(
                field.member.clone(),
                description.clone().into(),
            ),
        )?;
        let arg = self.expr(&field.expr)?;
        let path = field.member.clone().into();
        self.op(op_index(lhs, arg, path));
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<spec::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(spec::FieldValue {
            value,
            member: element.member.clone().into(),
        })
    }
    fn struct_expr(&mut self, id: NodeId, _struct: &ast_impl::ExprStruct) -> Result<Slot> {
        eprintln!("Struct expr {:?} template: {}", _struct, _struct.template);
        let lhs = self.reg(self.node_ty(id)?)?;
        let fields = _struct
            .fields
            .iter()
            .map(|x| self.field_value(x))
            .collect::<Result<_>>()?;
        let rest = _struct.rest.as_ref().map(|x| self.expr(x)).transpose()?;
        if let Kind::Enum(_enum) = &_struct.template.kind {
            eprintln!("Emitting enum opcode");
            self.op(op_enum(lhs, fields, _struct.template.clone()));
        } else {
            eprintln!("Emitting struct opcode");
            self.op(op_struct(lhs, fields, rest, _struct.template.clone()));
        }
        Ok(lhs)
    }
    fn match_expr(&mut self, id: NodeId, _match: &ast_impl::ExprMatch) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let target_ty = self.ty(_match.expr.id)?;
        let target = self.expr(&_match.expr)?;
        let discriminant = if let Ty::Enum(enum_ty) = target_ty {
            let disc_reg = self.reg(*enum_ty.discriminant.clone())?;
            self.op(op_discriminant(disc_reg, target));
            disc_reg
        } else {
            target
        };
        // Need to handle local rebindings in the bodies of the arms.
        let locals_prior_to_match = self.locals.clone();
        let mut arguments = vec![];
        let mut arm_locals = vec![];
        let mut arm_lhs = vec![];
        for arm in &_match.arms {
            self.locals = locals_prior_to_match.clone();
            let lhs = self.reg(self.node_ty(id)?)?;
            let disc = self.expr_arm(target, lhs, arm)?;
            arm_lhs.push(lhs);
            arguments.push(disc);
            arm_locals.push(self.locals.clone());
        }
        self.locals = locals_prior_to_match.clone();
        let mut rebound_locals = BTreeSet::new();
        for branch_locals in &arm_locals {
            let branch_rebindings = get_locals_changed(&self.locals, branch_locals)?;
            rebound_locals.extend(branch_rebindings);
        }
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<TypeId, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind((*x).into()).map(|r| (*x, r)))
            .collect::<Result<_>>()?;
        for (var, rebind) in &post_branch_bindings {
            let arm_bindings = arm_locals
                .iter()
                .map(|x| {
                    x.get(var).ok_or(anyhow!(
                        "ICE - no local var found for binding {var:?} in arm branch"
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            let cases = arguments
                .iter()
                .cloned()
                .zip(arm_bindings.into_iter().cloned())
                .collect::<Vec<_>>();
            let new_binding = rebind.to;
            self.op(op_case(new_binding, discriminant, cases));
        }
        let match_expr_table = arguments.iter().cloned().zip(arm_lhs).collect::<Vec<_>>();
        self.op(op_case(lhs, discriminant, match_expr_table));
        Ok(lhs)
    }
    fn bind_arm_pattern(&mut self, pattern: &Pat) -> Result<()> {
        match &pattern.kind {
            PatKind::Ident(ident) => self.bind(pattern.id, &ident.name),
            PatKind::Tuple(tuple) => {
                for pat in &tuple.elements {
                    self.bind_arm_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Type(ty) => self.bind_arm_pattern(&ty.pat),
            PatKind::Struct(_struct) => {
                for field in &_struct.fields {
                    self.bind_arm_pattern(&field.pat)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(_tuple_struct) => {
                for pat in &_tuple_struct.elems {
                    self.bind_arm_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for pat in &slice.elems {
                    self.bind_arm_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.bind_arm_pattern(&paren.pat),
            PatKind::Lit(_) | PatKind::Wild | PatKind::Path(_) => Ok(()),
            _ => bail!("Unsupported match binding pattern {:?}", pattern),
        }
    }

    fn expr_arm(&mut self, target: Slot, lhs: Slot, arm: &ast_impl::Arm) -> Result<CaseArgument> {
        match &arm.kind {
            ArmKind::Wild => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                Ok(CaseArgument::Wild)
            }
            ArmKind::Constant(constant) => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                let value =
                    cast_literal_to_inferred_type(constant.value.clone(), self.node_ty(arm.id)?)?
                        .discriminant()?;
                Ok(CaseArgument::Constant(value))
            }
            ArmKind::Enum(arm_enum) => {
                // Allocate the local bindings for the match pattern
                self.bind_arm_pattern(&arm_enum.pat)?;
                let discriminant = arm_enum.template.discriminant()?;
                let disc_as_i64 = arm_enum.template.discriminant()?.as_i64()?;
                let path = crate::path::Path::default().payload_by_value(disc_as_i64);
                let payload = self.reg(arm_enum.payload_kind.clone().into())?;
                self.op(op_index(payload, target, path));
                self.initialize_local(&arm_enum.pat, payload)?;
                let result = self.expr(&arm.body)?;
                self.op(op_assign(lhs, result));
                Ok(CaseArgument::Constant(discriminant))
            }
        }
    }
    fn return_expr(&mut self, _return: &ast_impl::ExprRet) -> Result<Slot> {
        // An early return of the type "return <expr>" is transformed
        // into the following equivalent expression
        // if !__early_return {
        //    __early_return = true;
        //    return_slot = <expr>
        // }
        // Because the function is pure, and has no side effects, the rest
        // of the function can continue as normal.  We do need to make sure
        // that a phi node is inserted at the end of this synthetic `if` statement
        // to build a distributed priority encoder for the `__early_return` flag and
        // for the return slot itself.

        let literal_true = self.literal_from_typed_bits(&true.typed_bits())?;
        let early_return_flag = self.rebind(EARLY_RETURN_FLAG_NODE)?;
        let return_slot = self.rebind(self.return_node)?;
        let early_return_expr = if let Some(return_expr) = &_return.expr {
            self.expr(return_expr)?
        } else {
            Slot::Empty
        };
        // Next, we need to code the following:
        //  if early_return_flag.from {
        //     return_slot.to = return_slot.from
        //     early_return_flag.to = early_return_flag.from
        //  } else {
        //     return_slot.to = <expr>
        //     early_return_flag.to = true
        //  }
        // These need to be encoded into 2 select instructions as:
        // return_slot.to = select(early_return_flag.from, return_slot.from, <expr>)
        // early_return_flag.to = select(early_return_flag.from, early_return_flag.from, true)
        self.op(op_select(
            return_slot.to,
            early_return_flag.from,
            return_slot.from,
            early_return_expr,
        ));
        self.op(op_select(
            early_return_flag.to,
            early_return_flag.from,
            early_return_flag.from,
            literal_true,
        ));
        Ok(Slot::Empty)
    }
    fn repeat(&mut self, id: NodeId, repeat: &ast_impl::ExprRepeat) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let len = self.expr(&repeat.len)?;
        let len = self.slot_to_index(len)?;
        let value = self.expr(&repeat.value)?;
        self.op(op_repeat(lhs, value, len));
        Ok(lhs)
    }
    fn assign(&mut self, assign: &ast_impl::ExprAssign) -> Result<Slot> {
        let rhs = self.expr(&assign.rhs)?;
        let (rebind, path) = self.expr_lhs(&assign.lhs)?;
        if path.is_empty() {
            self.op(op_assign(rebind.to, rhs));
        } else {
            self.op(op_splice(rebind.to, rebind.from, path, rhs));
        }
        Ok(Slot::Empty)
    }
    fn method_call(&mut self, id: NodeId, method_call: &ast_impl::ExprMethodCall) -> Result<Slot> {
        // First handle unary ops only
        let op = match method_call.method.as_str() {
            "any" => AluUnary::Any,
            "all" => AluUnary::All,
            "xor" => AluUnary::Xor,
            "as_unsigned" => AluUnary::Unsigned,
            "as_signed" => AluUnary::Signed,
            _ => bail!("Unsupported method call {:?}", method_call),
        };
        let lhs = self.reg(self.node_ty(id)?)?;
        let arg = self.expr(&method_call.receiver)?;
        self.op(op_unary(op, lhs, arg));
        Ok(lhs)
    }
    fn call(&mut self, id: NodeId, call: &ast_impl::ExprCall) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let path = collapse_path(&call.path);
        let args = self.expr_list(&call.args)?;
        // inline calls to bits and signed
        match &call.code {
            KernelFnKind::BitConstructor(len) => self.op(op_as_bits(lhs, args[0], *len)),
            KernelFnKind::SignedBitsConstructor(len) => self.op(op_as_signed(lhs, args[0], *len)),
            KernelFnKind::TupleStructConstructor(tb) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| spec::FieldValue {
                        value: *x,
                        member: Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_struct(lhs, fields, None, tb.clone()));
            }
            KernelFnKind::EnumTupleStructConstructor(template) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| spec::FieldValue {
                        value: *x,
                        member: Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_enum(lhs, fields, template.clone()));
            }
            _ => {
                let id = self.stash(ExternalFunction {
                    code: call.code.clone(),
                    path: path.clone(),
                    signature: call.signature.clone(),
                })?;
                self.op(op_exec(lhs, id, args));
            }
        }
        Ok(lhs)
    }
    fn self_assign_binop(&mut self, bin: &ExprBinary) -> Result<Slot> {
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let (dest, path) = self.expr_lhs(&bin.lhs)?;
        let temp = self.reg(self.node_ty(bin.lhs.id)?)?;
        let result = Slot::Empty;
        let op = &bin.op;
        let alu = match op {
            BinOp::AddAssign => AluBinary::Add,
            BinOp::SubAssign => AluBinary::Sub,
            BinOp::MulAssign => AluBinary::Mul,
            BinOp::BitXorAssign => AluBinary::BitXor,
            BinOp::BitAndAssign => AluBinary::BitAnd,
            BinOp::BitOrAssign => AluBinary::BitOr,
            BinOp::ShlAssign => AluBinary::Shl,
            BinOp::ShrAssign => AluBinary::Shr,
            _ => bail!("ICE - self_assign_binop {:?}", op),
        };
        self.op(op_binary(alu, temp, lhs, rhs));
        if path.is_empty() {
            self.op(op_assign(dest.to, temp));
        } else {
            self.op(op_splice(dest.to, dest.from, path, temp));
        }
        Ok(result)
    }
    fn binop(&mut self, id: NodeId, bin: &ExprBinary) -> Result<Slot> {
        let op = &bin.op;
        let self_assign = matches!(
            op,
            BinOp::AddAssign
                | BinOp::SubAssign
                | BinOp::MulAssign
                | BinOp::BitXorAssign
                | BinOp::BitAndAssign
                | BinOp::ShlAssign
                | BinOp::BitOrAssign
                | BinOp::ShrAssign
        );
        if self_assign {
            return self.self_assign_binop(bin);
        }
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let result = self.reg(self.node_ty(id)?)?;
        let alu = match op {
            BinOp::Add | BinOp::AddAssign => AluBinary::Add,
            BinOp::Sub | BinOp::SubAssign => AluBinary::Sub,
            BinOp::Mul | BinOp::MulAssign => AluBinary::Mul,
            BinOp::BitXor | BinOp::BitXorAssign => AluBinary::BitXor,
            BinOp::And | BinOp::BitAnd | BinOp::BitAndAssign => AluBinary::BitAnd,
            BinOp::Or | BinOp::BitOr | BinOp::BitOrAssign => AluBinary::BitOr,
            BinOp::Shl | BinOp::ShlAssign => AluBinary::Shl,
            BinOp::Shr | BinOp::ShrAssign => AluBinary::Shr,
            BinOp::Eq => AluBinary::Eq,
            BinOp::Lt => AluBinary::Lt,
            BinOp::Le => AluBinary::Le,
            BinOp::Ne => AluBinary::Ne,
            BinOp::Ge => AluBinary::Ge,
            BinOp::Gt => AluBinary::Gt,
        };
        assert!(!self_assign);
        self.op(op_binary(alu, result, lhs, rhs));
        Ok(result)
    }
    fn for_loop(&mut self, for_loop: &ast_impl::ExprForLoop) -> Result<Slot> {
        self.bind_pattern(&for_loop.pat)?;
        // Determine the loop type
        let index_reg = self.resolve_local(for_loop.pat.id)?;
        let index_ty = self.ty.get(&index_reg).cloned().ok_or(anyhow::anyhow!(
            "No type for index register {:?}",
            index_reg
        ))?;
        let ExprKind::Range(range) = &for_loop.expr.kind else {
            bail!("For loop must be over a range")
        };
        let Some(start) = &range.start else {
            bail!("For loop range must have a start")
        };
        let Some(end) = &range.end else {
            bail!("For loop range must have an end")
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(ExprLit::Int(start_lit)),
        } = start.as_ref()
        else {
            bail!("For loop range must have a literal start")
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(ExprLit::Int(end_lit)),
        } = end.as_ref()
        else {
            bail!("For loop range must have a literal end")
        };
        let start_lit = start_lit.parse::<i32>()?;
        let end_lit = end_lit.parse::<i32>()?;
        for ndx in start_lit..end_lit {
            let value = self.literal_from_type_and_int(&index_ty, ndx)?;
            self.rebind(for_loop.pat.id)?;
            self.initialize_local(&for_loop.pat, value)?;
            self.block(Slot::Empty, &for_loop.body)?;
        }
        Ok(Slot::Empty)
    }
    // We need three components
    //    - the original variable that holds the LHS
    //    - the path to change (if any)
    //    - the new place to write the value.
    // So, for example, if we have
    //    a[n] = b
    // Then we need to know:
    //    The original binding of `a`
    //    The path corresponding to `[n]`
    //    The place to store the result of splicing `a[n]<-b` in a new
    //    binding of the name 'a'.
    fn expr_lhs(&mut self, expr: &Expr) -> Result<(Rebind, crate::path::Path)> {
        match &expr.kind {
            ExprKind::Path(_path) => {
                let child_id = id_to_var(expr.id)?;
                let Ty::Var(child_id) = child_id else {
                    bail!("ICE - child id is not a var")
                };
                let Some(parent_tyid) = self.type_context.get_parent(child_id) else {
                    bail!("No parent for {:?}", child_id)
                };
                let rebind = self.rebind(parent_tyid.into())?;
                Ok((rebind, crate::path::Path::default()))
            }
            ExprKind::Field(field) => {
                let (rebind, path) = self.expr_lhs(&field.expr)?;
                let field = field.member.clone().into();
                Ok((rebind, path.join(&field)))
            }
            ExprKind::Index(index) => {
                let (rebind, path) = self.expr_lhs(&index.expr)?;
                let index = self.expr(&index.index)?;
                if index.is_literal() {
                    let ndx = self.slot_to_index(index)?;
                    Ok((rebind, path.index(ndx)))
                } else {
                    Ok((rebind, path.dynamic(index)))
                }
            }
            _ => todo!("expr_lhs {:?}", expr),
        }
    }
    fn expr(&mut self, expr: &Expr, description: &Description) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Array(array) => self.array(expr.id, array),
            ExprKind::Binary(bin) => self.binop(expr.id, bin),
            ExprKind::Block(block) => {
                let block_result = self.reg(self.node_ty(expr.id)?)?;
                self.block(block_result, &block.block)?;
                Ok(block_result)
            }
            ExprKind::If(if_expr) => self.if_expr(expr.id, if_expr),
            ExprKind::Lit(lit) => {
                let ndx = self.literals.len();
                let ty = self.ty(expr.id)?;
                self.literals.push(lit.clone());
                self.ty.insert(Slot::Literal(ndx), ty);
                Ok(Slot::Literal(ndx))
            }
            ExprKind::Field(field) => self.field(expr.id, field),
            ExprKind::Group(group) => self.expr(&group.expr),
            ExprKind::Index(index) => self.index(expr.id, index),
            ExprKind::Paren(paren) => self.expr(&paren.expr),
            ExprKind::Path(_path) => self.resolve_parent(expr.id),
            ExprKind::Struct(_struct) => self.struct_expr(expr.id, _struct),
            ExprKind::Tuple(tuple) => self.tuple(expr.id, tuple),
            ExprKind::Unary(unary) => self.unop(expr.id, unary),
            ExprKind::Match(_match) => self.match_expr(expr.id, _match),
            ExprKind::Ret(_return) => self.return_expr(_return),
            ExprKind::ForLoop(for_loop) => self.for_loop( for_loop),
            ExprKind::Assign(assign) => self.assign(assign),
            ExprKind::Range(_) => bail!("Ranges are only supported in for loops"),
            ExprKind::Let(_) => bail!("Fallible let expressions are not currently supported in rhdl.  Use a match instead"),
            ExprKind::Repeat(repeat) => self.repeat(expr.id, repeat),
            ExprKind::Call(call) => self.call(expr.id, call),
            ExprKind::MethodCall(method) => self.method_call(expr.id, method),
            ExprKind::Type(_) => Ok(Slot::Empty),
        }
    }
    fn wrap_expr_in_block(
        &mut self,
        block_result: Slot,
        expr: &Expr,
        description: &Description,
    ) -> Result<()> {
        let result = self.expr(expr, description)?;
        // Protects against empty assignments
        if block_result != result {
            self.op(op_assign(block_result, result));
        }
        Ok(())
    }
    fn bind_pattern(&mut self, pattern: &Pat) -> Result<()> {
        match &pattern.kind {
            PatKind::Ident(ident) => self.bind(pattern.id, &ident.name),
            PatKind::Tuple(tuple) => {
                for pat in &tuple.elements {
                    self.bind_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Type(ty) => self.bind_pattern(&ty.pat),
            PatKind::Struct(_struct) => {
                for field in &_struct.fields {
                    self.bind_pattern(&field.pat)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(_tuple_struct) => {
                for pat in &_tuple_struct.elems {
                    self.bind_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for pat in &slice.elems {
                    self.bind_pattern(pat)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.bind_pattern(&paren.pat),
            PatKind::Wild => Ok(()),
            _ => bail!("Unsupported binding pattern {:?}", pattern),
        }
    }

    // Add an implicit return statement at the end of the main block
    fn insert_implicit_return(&mut self, slot: Slot) -> Result<()> {
        // at the end of the main block, we need to insert a return of the return slot
        let early_return_flag = self.resolve_local(EARLY_RETURN_FLAG_NODE)?;
        let early_return_slot = self.rebind(self.return_node)?;
        self.op(op_select(
            early_return_slot.to,
            early_return_flag,
            early_return_slot.from,
            slot,
        ));
        Ok(())
    }
}

fn argument_id(arg_pat: &Pat) -> Result<(String, NodeId)> {
    match &arg_pat.kind {
        PatKind::Ident(name) => Ok((name.name.to_string(), arg_pat.id)),
        PatKind::Type(ty) => argument_id(&ty.pat),
        _ => {
            bail!("Arguments to kernel functions must be identifiers, instead got {arg_pat:?}")
        }
    }
}

impl Visitor for CompilerContext {
    fn visit_kernel_fn(&mut self, node: &ast_impl::KernelFn) -> Result<()> {
        // Allocate local binding for each argument
        let mut arguments = vec![];
        for arg in &node.inputs {
            let (arg_name, arg_id) = argument_id(arg)?;
            self.bind(arg_id, &arg_name)?;
            arguments.push(self.resolve_local(arg_id)?);
        }
        self.arguments = arguments;
        let block_result = self.reg(self.ty(node.id)?)?;
        self.return_node = node.id;
        // Allocate an unnamed local binding for the early return flag
        self.type_context
            .unify(self.ty(EARLY_RETURN_FLAG_NODE)?, ty_bool())?;
        // We create 2 bindings (local vars) inside the function
        //   - the early return flag - a flag of type bool that we initialize to false
        //   - the return slot - a register of type ret<fn>, that we initialize to the default
        // This is equivalent to injecting
        //   let mut __early$exit = false;
        //   let mut fn_name = Default::default();
        // at the beginning of the function body.
        // Each "return" statement must then be replaced with
        //    return x --> if !__early$exit { __early$exit = true; fn_name = x; }
        // The return slot is then used to return the value of the function.
        self.bind(EARLY_RETURN_FLAG_NODE, "__early$exit")?;
        self.bind(node.id, &node.name)?;
        // Initialize the early exit flag in the main block
        let init_early_exit_op = op_assign(
            self.resolve_local(NodeId::new(!0))?,
            self.literal_from_typed_bits(&false.typed_bits())?,
        );
        // Initialize the return slot in the main block
        let init_return_slot = op_assign(
            self.resolve_local(node.id)?,
            self.literal_from_typed_bits(&node.ret)?,
        );
        self.block(block_result, &node.body)?;
        self.ops.insert(0, init_early_exit_op);
        self.ops.insert(1, init_return_slot);
        self.insert_implicit_return(block_result)?;
        self.name = node.name.clone();
        self.fn_id = node.fn_id;
        Ok(())
    }
}

pub fn compile(func: &ast_impl::KernelFn, ctx: UnifyContext) -> Result<Object> {
    let mut compiler = CompilerContext::new(ctx);
    compiler.visit_kernel_fn(func)?;
    // Get the final name for the return value
    let return_slot = compiler.resolve_local(compiler.return_node)?;
    let literals = compiler
        .literals
        .into_iter()
        .enumerate()
        .map(|(ndx, lit)| {
            let ty = compiler
                .ty
                .get(&Slot::Literal(ndx))
                .cloned()
                .ok_or(anyhow!(
                    "ICE no literal type found for a literal in the table"
                ))?;
            cast_literal_to_inferred_type(lit, ty)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Object {
        literals,
        ty: compiler.ty,
        ops: compiler.ops,
        return_slot,
        externals: compiler.stash,
        arguments: compiler.arguments,
        fn_id: compiler.fn_id,
        name: compiler.name,
    })
}

fn cast_literal_to_inferred_type(t: ExprLit, ty: Ty) -> Result<TypedBits> {
    match t {
        ExprLit::TypedBits(t) => {
            if ty != t.value.kind.clone().into() {
                bail!(
                    "Literal with explicit type {:?} does not match inferred type {:?}",
                    t.value.kind,
                    ty
                )
            } else {
                Ok(t.value)
            }
        }
        ExprLit::Bool(b) => {
            if !ty.is_bool() {
                bail!(
                    "Literal with explicit type of bool does not match inferred type {:?}",
                    ty
                )
            } else {
                Ok(b.typed_bits())
            }
        }
        ExprLit::Int(x) => {
            if ty.is_unsigned() {
                let x_as_u128 = if let Some(x) = x.strip_prefix("0b") {
                    u128::from_str_radix(x, 2)?
                } else if let Some(x) = x.strip_prefix("0o") {
                    u128::from_str_radix(x, 8)?
                } else if let Some(x) = x.strip_prefix("0x") {
                    u128::from_str_radix(x, 16)?
                } else {
                    x.parse::<u128>()?
                };
                x_as_u128.typed_bits().unsigned_cast(ty.unsigned_bits()?)
            } else {
                let x_as_i128 = if let Some(x) = x.strip_prefix("0b") {
                    i128::from_str_radix(x, 2)?
                } else if let Some(x) = x.strip_prefix("0o") {
                    i128::from_str_radix(x, 8)?
                } else if let Some(x) = x.strip_prefix("0x") {
                    i128::from_str_radix(x, 16)?
                } else {
                    x.parse::<i128>()?
                };
                x_as_i128.typed_bits().signed_cast(ty.signed_bits()?)
            }
        }
    }
}

fn get_locals_changed(from: &LocalsMap, to: &LocalsMap) -> Result<BTreeSet<TypeId>> {
    from.iter()
        .filter_map(|(id, slot)| {
            {
                if let Some(to_slot) = to.get(id) {
                    Ok(if to_slot != slot { Some(*id) } else { None })
                } else {
                    Err(anyhow!(
                        "ICE - local variable {:?} not found in branch map",
                        id
                    ))
                }
            }
            .transpose()
        })
        .collect()
}
