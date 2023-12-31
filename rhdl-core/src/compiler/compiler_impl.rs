use crate::{
    ast::ast_impl::{
        self, ArmKind, BinOp, Expr, ExprBinary, ExprIf, ExprKind, ExprLit, ExprTuple,
        ExprTypedBits, FieldValue, FunctionId, Local, NodeId, Pat, PatKind, Path,
    },
    ast::display_ast::pretty_print_statement,
    ast::visit::Visitor,
    compiler::ty::{ty_empty, ty_indexed_item, ty_named_field, ty_unnamed_field, Bits, Ty, TypeId},
    compiler::UnifyContext,
    rhif::rhif_builder::{
        op_array, op_as_bits, op_as_signed, op_assign, op_binary, op_case, op_comment,
        op_discriminant, op_enum, op_exec, op_if, op_index, op_repeat, op_return, op_struct,
        op_tuple, op_unary,
    },
    rhif::rhif_spec::{
        self, AluBinary, AluUnary, BlockId, CaseArgument, ExternalFunction, FuncId, Member, OpCode,
        Slot,
    },
    rhif::Object,
    types::typed_bits::TypedBits,
    Digital, KernelFnKind, Kind,
};
use anyhow::{anyhow, bail, Result};
use std::collections::{BTreeMap, HashMap};

use super::infer_types::id_to_var;

const ROOT_BLOCK: BlockId = BlockId(0);

impl From<ast_impl::Member> for rhif_spec::Member {
    fn from(member: ast_impl::Member) -> Self {
        match member {
            ast_impl::Member::Named(name) => rhif_spec::Member::Named(name),
            ast_impl::Member::Unnamed(index) => rhif_spec::Member::Unnamed(index),
        }
    }
}

pub struct Block {
    pub id: BlockId,
    pub names: HashMap<String, Slot>,
    pub ops: Vec<OpCode>,
    pub result: Slot,
    pub children: Vec<BlockId>,
    pub parent: BlockId,
}

#[derive(Clone, Debug)]
pub struct NamedSlot {
    name: String,
    slot: Slot,
}

pub struct CompilerContext {
    pub blocks: Vec<Block>,
    pub literals: Vec<Box<ast_impl::ExprLit>>,
    pub reg_count: usize,
    active_block: BlockId,
    type_context: UnifyContext,
    ty: BTreeMap<Slot, Ty>,
    locals: HashMap<TypeId, NamedSlot>,
    stash: Vec<ExternalFunction>,
    return_slot: Slot,
    main_block: BlockId,
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
        for block in &self.blocks {
            writeln!(f, "Block {}", block.id.0)?;
            for op in &block.ops {
                writeln!(f, "  {}", op)?;
            }
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
            blocks: vec![Block {
                id: ROOT_BLOCK,
                names: Default::default(),
                ops: vec![],
                result: Slot::Empty,
                children: vec![],
                parent: ROOT_BLOCK,
            }],
            literals: vec![],
            reg_count: 0,
            active_block: ROOT_BLOCK,
            type_context,
            ty: [(Slot::Empty, ty_empty())].into_iter().collect(),
            locals: Default::default(),
            stash: Default::default(),
            return_slot: Slot::Empty,
            main_block: ROOT_BLOCK,
            arguments: Default::default(),
            fn_id: Default::default(),
            name: Default::default(),
        }
    }
    fn node_ty(&self, id: NodeId) -> Result<Ty> {
        let var = id_to_var(id)?;
        Ok(self.type_context.apply(var))
    }
    fn reg(&mut self, ty: Ty) -> Result<Slot> {
        if ty.is_empty() {
            return Ok(Slot::Empty);
        }
        let reg = Slot::Register(self.reg_count);
        self.ty.insert(reg, ty);
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
                let x: i128 = value.try_into()?;
                x.typed_bits()
            }
            Ty::Const(Bits::Unsigned(n)) => {
                let x: u128 = value.try_into()?;
                x.typed_bits().unsigned_cast(*n)?
            }
            Ty::Const(Bits::Signed(n)) => {
                let x: i128 = value.try_into()?;
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
            .push(Box::new(ast_impl::ExprLit::TypedBits(ExprTypedBits {
                path: Box::new(Path { segments: vec![] }),
                value: value.clone(),
            })));
        self.ty.insert(Slot::Literal(ndx), ty);
        Ok(Slot::Literal(ndx))
    }
    fn op(&mut self, op: OpCode) {
        self.blocks[self.active_block.0].ops.push(op);
    }
    fn new_block(&mut self, result: Slot) -> BlockId {
        let id = BlockId(self.blocks.len());
        self.blocks.push(Block {
            id,
            names: Default::default(),
            ops: vec![],
            result,
            children: vec![],
            parent: self.active_block,
        });
        self.blocks[self.active_block.0].children.push(id);
        self.active_block = id;
        id
    }
    fn current_block(&self) -> BlockId {
        self.active_block
    }
    fn set_active_block(&mut self, id: BlockId) {
        self.active_block = id;
    }
    fn slot_to_index(&self, slot: Slot) -> Result<usize> {
        let Slot::Literal(ndx) = slot else {
            bail!("ICE - index is not a literal")
        };
        let ndx = match self.literals[ndx].as_ref() {
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
        let reg = self.reg(ty)?;
        eprintln!("Binding {name}#{id} -> {reg}");
        if self
            .locals
            .insert(
                id.into(),
                NamedSlot {
                    slot: reg,
                    name: name.to_string(),
                },
            )
            .is_some()
        {
            bail!("Duplicate local variable binding for {:?}", id)
        }
        Ok(())
    }
    fn resolve_parent(&mut self, child: NodeId) -> Result<Slot> {
        let child_id = id_to_var(child)?;
        if let Ty::Var(child_id) = child_id {
            if let Some(parent) = self.type_context.get_parent(child_id) {
                return self
                    .locals
                    .get(&parent)
                    .map(|x| x.slot)
                    .ok_or(anyhow::anyhow!(
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
            .map(|x| x.slot)
            .ok_or(anyhow::anyhow!("No local variable for {:?}", id))
    }
    fn unop(&mut self, id: NodeId, unary: &ast_impl::ExprUnary) -> Result<Slot> {
        let arg = self.expr(&unary.expr)?;
        let result = self.reg(self.node_ty(id)?)?;
        let op = match unary.op {
            ast_impl::UnOp::Neg => AluUnary::Neg,
            ast_impl::UnOp::Not => AluUnary::Not,
        };
        self.op(op_unary(op, result, arg));
        Ok(result)
    }
    fn stmt(&mut self, statement: &ast_impl::Stmt) -> Result<Slot> {
        let statement_text = pretty_print_statement(statement, &self.type_context)?;
        self.op(op_comment(statement_text));
        match &statement.kind {
            ast_impl::StmtKind::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            ast_impl::StmtKind::Expr(expr) => self.expr(expr),
            ast_impl::StmtKind::Semi(expr) => {
                self.expr(expr)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn initialize_local(&mut self, pat: &Pat, rhs: Slot) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(_ident) => {
                if let Ok(lhs) = self.resolve_local(pat.id) {
                    self.op(op_assign(lhs, rhs, crate::path::Path::default()));
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
                    let element_rhs = self.reg(element_ty)?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(pat, element_rhs)?;
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
                    let element_rhs = self.reg(element_ty)?;
                    let path = field.member.clone().into();
                    self.op(op_index(element_rhs, rhs, path));
                    self.initialize_local(&field.pat, element_rhs)?;
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
                    let element_rhs = self.reg(element_ty)?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(pat, element_rhs)?;
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
                    let element_rhs = self.reg(element_ty)?;
                    self.op(op_index(
                        element_rhs,
                        rhs,
                        crate::path::Path::default().index(ndx),
                    ));
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Type(ty) => self.initialize_local(&ty.pat, rhs),
            PatKind::Wild | PatKind::Lit(_) | PatKind::Path(_) => Ok(()),
            _ => todo!("Unsupported let init pattern: {:?}", pat),
        }
    }
    fn local(&mut self, local: &Local) -> Result<()> {
        self.bind_pattern(&local.pat)?;
        if let Some(init) = &local.init {
            let rhs = self.expr(init)?;
            self.initialize_local(&local.pat, rhs)?;
        }
        Ok(())
    }
    fn block(&mut self, block_result: Slot, block: &ast_impl::Block) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(block_result);
        let statement_count = block.stmts.len();
        for (ndx, statement) in block.stmts.iter().enumerate() {
            let is_last = ndx == statement_count - 1;
            let result = self.stmt(statement)?;
            if is_last && (block_result != result) {
                self.op(op_assign(
                    block_result,
                    result,
                    crate::path::Path::default(),
                ));
            }
        }
        self.set_active_block(current_block);
        Ok(id)
    }
    fn expr_list(&mut self, exprs: &[Box<Expr>]) -> Result<Vec<Slot>> {
        exprs.iter().map(|x| self.expr(x)).collect::<Result<_>>()
    }
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple) -> Result<Slot> {
        let result = self.reg(self.node_ty(id)?)?;
        let fields = self.expr_list(&tuple.elements)?;
        self.op(op_tuple(result, fields));
        Ok(result)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf) -> Result<Slot> {
        let result = self.reg(self.node_ty(id)?)?;
        let cond = self.expr(&if_expr.cond)?;
        let then_branch = self.block(result, &if_expr.then_branch)?;
        let else_branch = if let Some(expr) = if_expr.else_branch.as_ref() {
            self.wrap_expr_in_block(result, expr)
        } else {
            self.empty_block(result)
        }?;
        self.op(op_if(result, cond, then_branch, else_branch));
        Ok(result)
    }
    fn index(&mut self, id: NodeId, index: &ast_impl::ExprIndex) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let arg = self.expr(&index.expr)?;
        let index = self.expr(&index.index)?;
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
    fn array(&mut self, id: NodeId, array: &ast_impl::ExprArray) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let elements = self.expr_list(&array.elems)?;
        self.op(op_array(lhs, elements));
        Ok(lhs)
    }
    fn field(&mut self, id: NodeId, field: &ast_impl::ExprField) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let arg = self.expr(&field.expr)?;
        let path = field.member.clone().into();
        self.op(op_index(lhs, arg, path));
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<rhif_spec::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(rhif_spec::FieldValue {
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
        let table = _match
            .arms
            .iter()
            .map(|x| self.expr_arm(target, lhs, x))
            .collect::<Result<_>>()?;
        self.op(op_case(discriminant, table));
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

    fn expr_arm(
        &mut self,
        target: Slot,
        lhs: Slot,
        arm: &ast_impl::Arm,
    ) -> Result<(CaseArgument, BlockId)> {
        match &arm.kind {
            ArmKind::Wild => {
                let block = self.wrap_expr_in_block(lhs, &arm.body)?;
                Ok((CaseArgument::Wild, block))
            }
            ArmKind::Constant(constant) => {
                let block = self.wrap_expr_in_block(lhs, &arm.body)?;
                let value =
                    cast_literal_to_inferred_type(constant.value.clone(), self.node_ty(arm.id)?)?
                        .discriminant()?;
                Ok((CaseArgument::Constant(value), block))
            }
            ArmKind::Enum(arm_enum) => {
                // Allocate the local bindings for the match pattern
                self.bind_arm_pattern(&arm_enum.pat)?;
                let discriminant = arm_enum.template.discriminant()?;
                let disc_as_i64 = arm_enum.template.discriminant()?.as_i64()?;
                let path = crate::path::Path::default().payload_by_value(disc_as_i64);
                let current_block = self.current_block();
                let id = self.new_block(lhs);
                let payload = self.reg(arm_enum.payload_kind.clone().into())?;
                self.op(op_index(payload, target, path));
                self.initialize_local(&arm_enum.pat, payload)?;
                let result = self.expr(&arm.body)?;
                self.op(op_assign(lhs, result, crate::path::Path::default()));
                self.set_active_block(current_block);
                Ok((CaseArgument::Constant(discriminant), id))
            }
        }
    }
    fn return_expr(&mut self, _return: &ast_impl::ExprRet) -> Result<Slot> {
        if let Some(expr) = &_return.expr {
            let result = self.expr(expr)?;
            self.op(op_assign(
                self.return_slot,
                result,
                crate::path::Path::default(),
            ));
        }
        self.op(op_return());
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
        let (lhs, path) = self.expr_lhs(&assign.lhs)?;
        let rhs = self.expr(&assign.rhs)?;
        self.op(op_assign(lhs, rhs, path));
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
                    .map(|(ndx, x)| rhif_spec::FieldValue {
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
                    .map(|(ndx, x)| rhif_spec::FieldValue {
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
        let (dest, path) = self.expr_lhs(&bin.lhs)?;
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
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
        self.op(op_assign(dest, temp, path));
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
        let current_block = self.current_block();
        let loop_block = self.new_block(Slot::Empty);
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
            self.initialize_local(&for_loop.pat, value)?;
            let body = self.block(Slot::Empty, &for_loop.body)?;
            self.op(OpCode::Block(body));
        }
        self.set_active_block(current_block);
        self.op(OpCode::Block(loop_block));
        Ok(Slot::Empty)
    }
    fn expr_lhs(&mut self, expr: &Expr) -> Result<(Slot, crate::path::Path)> {
        match &expr.kind {
            ExprKind::Path(_path) => {
                let parent = self.resolve_parent(expr.id)?;
                Ok((parent, crate::path::Path::default()))
            }
            ExprKind::Field(field) => {
                let (slot, path) = self.expr_lhs(&field.expr)?;
                let field = field.member.clone().into();
                Ok((slot, path.join(&field)))
            }
            ExprKind::Index(index) => {
                let (slot, path) = self.expr_lhs(&index.expr)?;
                let index = self.expr(&index.index)?;
                if index.is_literal() {
                    let ndx = self.slot_to_index(index)?;
                    Ok((slot, path.index(ndx)))
                } else {
                    Ok((slot, path.dynamic(index)))
                }
            }
            _ => todo!("expr_lhs {:?}", expr),
        }
    }
    fn expr(&mut self, expr: &Expr) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Array(array) => self.array(expr.id, array),
            ExprKind::Binary(bin) => self.binop(expr.id, bin),
            ExprKind::Block(block) => {
                let block_result = self.reg(self.node_ty(expr.id)?)?;
                let block_id = self.block(block_result, &block.block)?;
                self.op(OpCode::Block(block_id));
                Ok(block_result)
            }
            ExprKind::If(if_expr) => self.if_expr(expr.id, if_expr),
            ExprKind::Lit(lit) => {
                let ndx = self.literals.len();
                let ty = self.ty(expr.id)?;
                self.literals.push(Box::new(lit.clone()));
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
    fn wrap_expr_in_block(&mut self, block_result: Slot, expr: &Expr) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(block_result);
        let result = self.expr(expr)?;
        if result != block_result {
            self.op(op_assign(
                block_result,
                result,
                crate::path::Path::default(),
            ));
        }
        self.set_active_block(current_block);
        Ok(id)
    }
    fn empty_block(&mut self, block_result: Slot) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(block_result);
        self.set_active_block(current_block);
        Ok(id)
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
        self.return_slot = block_result;
        self.main_block = self.block(block_result, &node.body)?;
        self.name = node.name.clone();
        self.fn_id = node.fn_id;
        Ok(())
    }
}

pub fn compile(func: &ast_impl::KernelFn, ctx: UnifyContext) -> Result<Object> {
    let mut compiler = CompilerContext::new(ctx);
    compiler.visit_kernel_fn(func)?;
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
            cast_literal_to_inferred_type(*lit, ty)
        })
        .collect::<Result<Vec<_>>>()?;
    let blocks = compiler
        .blocks
        .into_iter()
        .map(|x| rhif_spec::Block {
            id: x.id,
            ops: x.ops,
        })
        .collect();
    Ok(Object {
        literals,
        ty: compiler.ty,
        blocks,
        return_slot: compiler.return_slot,
        externals: compiler.stash,
        main_block: compiler.main_block,
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
