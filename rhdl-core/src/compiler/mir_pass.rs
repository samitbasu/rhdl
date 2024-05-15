// Convert the AST into the mid-level representation, which is RHIF, but with out
// concrete types.

// AST elements that carry Type information:
// PatType
// ExprType - not covered
// ExprStruct
// ArmEnum
// KernelFn (ret)

use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use crate::ast::ast_impl;
use crate::ast::ast_impl::BitsKind;
use crate::ast::ast_impl::ExprBits;
use crate::ast::ast_impl::ExprTypedBits;
use crate::ast::ast_impl::Member;
use crate::ast::ast_impl::{
    Arm, ArmKind, Block, ExprArray, ExprAssign, ExprBinary, ExprCall, ExprField, ExprForLoop,
    ExprIf, ExprIndex, ExprMatch, ExprMethodCall, ExprPath, ExprRepeat, ExprRet, ExprStruct,
    ExprTuple, ExprUnary, FieldValue, Local, Pat, PatKind, Stmt, StmtKind,
};
use crate::ast::visit::Visitor;
use crate::ast::visit_mut::VisitorMut;
use crate::ast_builder::BinOp;
use crate::ast_builder::UnOp;
use crate::kernel::Kernel;
use crate::rhif;
use crate::rhif::object::SymbolMap;
use crate::rhif::rhif_builder::op_as_bits_inferred;
use crate::rhif::rhif_builder::op_as_signed_inferred;
use crate::rhif::rhif_builder::op_retime;
use crate::rhif::rhif_builder::{
    op_array, op_as_bits, op_as_signed, op_assign, op_binary, op_case, op_comment, op_enum,
    op_exec, op_index, op_repeat, op_select, op_splice, op_struct, op_tuple, op_unary,
};
use crate::rhif::spanned_source::build_spanned_source_for_kernel;
use crate::rhif::spec;
use crate::rhif::spec::AluBinary;
use crate::rhif::spec::AluUnary;
use crate::rhif::spec::CaseArgument;
use crate::rhif::spec::ExternalFunctionCode;
use crate::rhif::spec::FuncId;
use crate::KernelFnKind;
use crate::Kind;
use crate::TypedBits;
use crate::{
    ast::ast_impl::{Expr, ExprKind, ExprLit, FunctionId, NodeId},
    rhif::spec::{ExternalFunction, OpCode, Slot},
};

use super::assign_node::NodeIdGenerator;
use super::display_ast::pretty_print_statement;
use super::mir::Mir;
use super::mir::OpCodeWithSource;

#[derive(Debug, Clone)]
pub struct Rebind {
    from: Slot,
    to: Slot,
}

impl From<ast_impl::Member> for spec::Member {
    fn from(member: ast_impl::Member) -> Self {
        match member {
            ast_impl::Member::Named(name) => spec::Member::Named(name),
            ast_impl::Member::Unnamed(index) => spec::Member::Unnamed(index),
        }
    }
}

fn binop_to_alu(op: BinOp) -> AluBinary {
    match op {
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
    }
}

fn unary_op_to_alu(op: UnOp) -> AluUnary {
    match op {
        UnOp::Neg => AluUnary::Neg,
        UnOp::Not => AluUnary::Not,
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScopeId(usize);

const ROOT_SCOPE: ScopeId = ScopeId(0);

impl Default for ScopeId {
    fn default() -> Self {
        ROOT_SCOPE
    }
}

fn collapse_path(path: &ast_impl::Path) -> String {
    path.segments
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

// TODO = worry about the string clones later.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScopeIndex {
    scope: ScopeId,
    name: String,
}

type LocalsMap = HashMap<ScopeIndex, Slot>;

#[derive(Debug)]
struct Scope {
    names: HashMap<String, Slot>,
    children: Vec<ScopeId>,
    parent: ScopeId,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope {{")?;
        for (name, id) in &self.names {
            writeln!(f, "  {} -> {}", name, id)?;
        }
        writeln!(f, "}}")
    }
}

const EARLY_RETURN_FLAG_NAME: &str = "__$early_return_flag";

pub struct MirContext {
    scopes: Vec<Scope>,
    ops: Vec<OpCodeWithSource>,
    reg_count: usize,
    literals: BTreeMap<Slot, ExprLit>,
    reg_source_map: BTreeMap<Slot, NodeId>,
    ty: BTreeMap<Slot, Kind>,
    ty_equate: BTreeSet<(Slot, Slot)>,
    stash: Vec<ExternalFunction>,
    return_slot: Slot,
    arguments: Vec<Slot>,
    fn_id: FunctionId,
    name: String,
    active_scope: ScopeId,
}

impl Default for MirContext {
    fn default() -> Self {
        MirContext {
            scopes: vec![Scope {
                names: HashMap::new(),
                children: vec![],
                parent: ROOT_SCOPE,
            }],
            ops: vec![],
            reg_count: 0,
            literals: BTreeMap::new(),
            reg_source_map: BTreeMap::new(),
            ty: BTreeMap::new(),
            ty_equate: BTreeSet::new(),
            stash: vec![],
            return_slot: Slot::Empty,
            arguments: vec![],
            fn_id: FunctionId::default(),
            name: "".to_string(),
            active_scope: ROOT_SCOPE,
        }
    }
}

impl std::fmt::Display for MirContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arguments = self
            .arguments
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            f,
            "Kernel {}({})->{} ({})",
            self.name, arguments, self.return_slot, self.fn_id
        )?;
        for (slot, kind) in &self.ty {
            writeln!(f, "{} : {}", slot, kind)?;
        }
        for (lit, expr) in &self.literals {
            writeln!(f, "{} -> {}", lit, expr)?;
        }
        for (ndx, func) in self.stash.iter().enumerate() {
            writeln!(
                f,
                "Function f{} name: {} code: {} signature: {}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        for op in &self.ops {
            writeln!(f, "{}", op.op)?;
        }
        Ok(())
    }
}

impl MirContext {
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
    // Walk the scope hierarchy, and return a list of all local variables
    // visible from the current scope.  Some of these may not be accessible
    // due to shadowing.
    fn locals(&self) -> LocalsMap {
        let mut locals = LocalsMap::new();
        let mut scope = self.active_scope;
        loop {
            locals.extend(self.scopes[scope.0].names.iter().map(|(k, v)| {
                (
                    ScopeIndex {
                        scope,
                        name: k.clone(),
                    },
                    *v,
                )
            }));
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        locals
    }
    fn set_locals(&mut self, map: &LocalsMap) -> Result<()> {
        for (ndx, slot) in map {
            let ScopeIndex { scope, name } = ndx;
            let Some(binding) = self.scopes[scope.0].names.get_mut(name) else {
                bail!("ICE - attempt to set local variable that does not exist")
            };
            *binding = *slot;
        }
        Ok(())
    }
    fn unpack_arguments(&mut self, args: &[Box<Pat>]) -> Result<()> {
        for arg in args {
            let slot = self.reg(arg.id);
            let PatKind::Type(ty) = &arg.kind else {
                bail!("ICE - argument pattern not supported {:?}", arg)
            };
            self.ty.insert(slot, ty.kind.clone());
            self.arguments.push(slot);
        }
        Ok(())
    }
    // Create a local variable binding to the given name, and return the
    // resulting register.
    fn bind(&mut self, name: &str, id: NodeId) {
        let reg = self.reg(id);
        eprintln!("Binding {}#{} to {:?}", name, id, reg);
        self.scopes[self.active_scope.0]
            .names
            .insert(name.to_string(), reg);
    }
    // Rebind a local variable to a new slot.  We need
    // to know the previous name for the slot in this case.
    // For example, if we have:
    // ```
    //  let a = 5; //<-- original binding to name of "a"
    //  let a = 6; //<-- rebind of "a" to new slot
    //```
    fn rebind(&mut self, name: &str, id: NodeId) -> Result<Rebind> {
        let reg = self.reg(id);
        let Some((prev, scope)) = self.lookup_name(name) else {
            bail!("ICE - rebind of unbound variable {}", name);
        };
        self.scopes[scope.0].names.insert(name.to_string(), reg);
        eprintln!("Rebound {} from {} to {}", name, prev, reg);
        Ok(Rebind {
            from: prev,
            to: reg,
        })
    }
    fn reg(&mut self, id: NodeId) -> Slot {
        let reg = Slot::Register(self.reg_count);
        self.reg_source_map.insert(reg, id);
        self.reg_count += 1;
        reg
    }
    fn lit(&mut self, id: NodeId, lit: ExprLit) -> Slot {
        eprintln!("Allocate literal {:?} for {}", lit, id);
        let ndx = self.literals.len();
        let slot = Slot::Literal(ndx);
        self.literals.insert(slot, lit);
        self.reg_source_map.insert(slot, id);
        slot
    }
    fn literal_int(&mut self, id: NodeId, val: i32) -> Slot {
        self.lit(id, ExprLit::Int(val.to_string()))
    }
    fn literal_bool(&mut self, id: NodeId, val: bool) -> Slot {
        self.lit(id, ExprLit::Bool(val))
    }
    fn literal_tb(&mut self, id: NodeId, place_holder: &TypedBits) -> Slot {
        self.lit(
            id,
            ExprLit::TypedBits(ExprTypedBits {
                path: Box::new(ast_impl::Path { segments: vec![] }),
                value: place_holder.clone(),
            }),
        )
    }
    fn lookup_name(&self, path: &str) -> Option<(Slot, ScopeId)> {
        let mut scope = self.active_scope;
        loop {
            if let Some(id) = self.scopes[scope.0].names.get(path) {
                return Some((*id, scope));
            }
            if scope == ROOT_SCOPE {
                break;
            }
            scope = self.scopes[scope.0].parent;
        }
        None
    }
    fn slot_to_index(&self, slot: Slot) -> Result<usize> {
        let Some(value) = self.literals.get(&slot) else {
            bail!(
                "ICE - slot_to_index called with non-literal slot {:?}",
                slot
            );
        };
        let ndx = match value {
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
    fn initialize_local(&mut self, pat: &Pat, rhs: Slot) -> Result<()> {
        match &pat.kind {
            PatKind::Ident(ident) => {
                let Some((lhs, _)) = self.lookup_name(&ident.name) else {
                    bail!(
                        "ICE - attempt to intiialize unbound local variable {}",
                        ident.name
                    );
                };
                self.op(op_assign(lhs, rhs), pat.id);
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for (ndx, pat) in tuple.elements.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(
                            element_rhs,
                            rhs,
                            crate::path::Path::default().tuple_index(ndx),
                        ),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    let element_rhs = self.reg(field.pat.id);
                    let path = field.member.clone().into();
                    self.op(op_index(element_rhs, rhs, path), field.pat.id);
                    self.initialize_local(&field.pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct_pat) => {
                for (ndx, pat) in tuple_struct_pat.elems.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(
                            element_rhs,
                            rhs,
                            crate::path::Path::default().tuple_index(ndx),
                        ),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for (ndx, pat) in slice.elems.iter().enumerate() {
                    let element_rhs = self.reg(pat.id);
                    self.op(
                        op_index(element_rhs, rhs, crate::path::Path::default().index(ndx)),
                        pat.id,
                    );
                    self.initialize_local(pat, element_rhs)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => {
                // We can ignore the type information here because
                // initialize_local needs to be called after bind_pattern.
                self.initialize_local(&type_pat.pat, rhs)
            }
            PatKind::Wild | PatKind::Lit(_) | PatKind::Path(_) => Ok(()),
            _ => bail!("Pattern not supported"),
        }
    }
    fn stash(&mut self, func: ExternalFunction) -> Result<FuncId> {
        let ndx = self.stash.len();
        self.stash.push(func);
        Ok(FuncId(ndx))
    }
    fn op(&mut self, op: OpCode, node: NodeId) {
        self.ops.push(OpCodeWithSource { op, source: node });
    }
    fn insert_implicit_return(
        &mut self,
        id: NodeId,
        slot: Slot,
        early_return_name: &str,
    ) -> Result<()> {
        let (early_return_flag, _) = self.lookup_name(EARLY_RETURN_FLAG_NAME).ok_or(anyhow!(
            "ICE - no early return flag found in function {}",
            self.fn_id
        ))?;
        let early_return_slot = self.rebind(early_return_name, id)?;
        self.op(
            op_select(
                early_return_slot.to,
                early_return_flag,
                early_return_slot.from,
                slot,
            ),
            id,
        );
        Ok(())
    }
    fn array(&mut self, id: NodeId, array: &ExprArray) -> Result<Slot> {
        let lhs = self.reg(id);
        let elements = self.expr_list(&array.elems)?;
        self.op(op_array(lhs, elements), id);
        Ok(lhs)
    }
    fn arm(&mut self, target: Slot, lhs: Slot, arm: &Arm) -> Result<CaseArgument> {
        match &arm.kind {
            ArmKind::Wild => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                Ok(CaseArgument::Wild)
            }
            ArmKind::Constant(constant) => {
                self.wrap_expr_in_block(lhs, &arm.body)?;
                let value = self.lit(arm.id, constant.value.clone());
                let disc = self.reg(arm.id);
                self.op(
                    op_index(disc, value, crate::path::Path::default().discriminant()),
                    arm.id,
                );
                Ok(CaseArgument::Slot(disc))
            }
            ArmKind::Enum(arm_enum) => {
                self.new_scope();
                // Allocate the local bindings for the match pattern
                self.bind_pattern(&arm_enum.pat)?;
                let discriminant = arm_enum.template.discriminant()?;
                let discriminant_slot = self.lit(
                    arm.id,
                    ExprLit::TypedBits(ast_impl::ExprTypedBits {
                        path: Box::new(ast_impl::Path { segments: vec![] }),
                        value: discriminant,
                    }),
                );
                let disc_as_i64 = arm_enum.template.discriminant()?.as_i64()?;
                let variant_name = arm_enum
                    .template
                    .kind
                    .lookup_variant_name_by_discriminant(disc_as_i64)?;
                let path = crate::path::Path::default().payload(variant_name);
                let payload = self.reg(arm_enum.pat.id);
                self.op(op_index(payload, target, path), arm_enum.pat.id);
                self.initialize_local(&arm_enum.pat, payload)?;
                let result = self.expr(&arm.body)?;
                self.op(op_assign(lhs, result), arm_enum.pat.id);
                self.end_scope();
                Ok(CaseArgument::Slot(discriminant_slot))
            }
        }
    }
    fn assign(&mut self, id: NodeId, assign: &ExprAssign) -> Result<Slot> {
        let rhs = self.expr(&assign.rhs)?;
        let (rebind, path) = self.expr_lhs(&assign.lhs)?;
        self.ty_equate.insert((rebind.to, rebind.from));
        if path.is_empty() {
            self.op(op_assign(rebind.to, rhs), id);
        } else {
            self.op(op_splice(rebind.to, rebind.from, path, rhs), id);
        }
        Ok(Slot::Empty)
    }
    fn assign_binop(&mut self, id: NodeId, bin: &ExprBinary) -> Result<Slot> {
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let (dest, path) = self.expr_lhs(&bin.lhs)?;
        let temp = self.reg(bin.lhs.id);
        let result = Slot::Empty;
        let op = &bin.op;
        ensure!(op.is_self_assign(), "ICE - self_assign_binop {:?}", op);
        let alu = binop_to_alu(*op);
        match op {
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
        self.ty_equate.insert((dest.to, dest.from));
        self.op(op_binary(alu, temp, lhs, rhs), id);
        if path.is_empty() {
            self.op(op_assign(dest.to, temp), id);
        } else {
            self.op(op_splice(dest.to, dest.from, path, temp), id);
        }
        Ok(result)
    }
    fn block(&mut self, block_result: Slot, block: &Block) -> Result<()> {
        let statement_count = block.stmts.len();
        self.new_scope();
        if block.stmts.is_empty() {
            self.op(op_assign(block_result, Slot::Empty), block.id);
        } else {
            for (ndx, statement) in block.stmts.iter().enumerate() {
                let is_last = ndx == statement_count - 1;
                let result = self.stmt(statement)?;
                if is_last && (block_result != result) {
                    self.op(op_assign(block_result, result), statement.id);
                }
            }
        }
        self.end_scope();
        Ok(())
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
            return self.assign_binop(id, bin);
        }
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let result = self.reg(id);
        let alu = binop_to_alu(*op);
        assert!(!self_assign);
        self.op(op_binary(alu, result, lhs, rhs), id);
        Ok(result)
    }
    fn type_pattern(&mut self, pattern: &Pat, kind: &Kind) -> Result<()> {
        eprintln!("Type pattern {:?} {:?}", pattern, kind);
        match &pattern.kind {
            PatKind::Ident(ident) => {
                let (slot, _) = self.lookup_name(&ident.name).ok_or(anyhow!(
                    "ICE - no local variable found for pattern {:?}",
                    pattern
                ))?;
                eprintln!("Binding {:?} to {:?} via {}", ident, kind, slot);
                self.ty.insert(slot, kind.clone());
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for (ndx, element) in tuple.elements.iter().enumerate() {
                    self.type_pattern(element, &kind.get_tuple_kind(ndx)?)?;
                }
                Ok(())
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    self.type_pattern(&field.pat, &kind.get_field_kind(&field.member)?)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct) => {
                for (ndx, field) in tuple_struct.elems.iter().enumerate() {
                    self.type_pattern(field, &kind.get_field_kind(&Member::Unnamed(ndx as u32))?)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for element in &slice.elems {
                    self.type_pattern(element, &kind.get_base_kind()?)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.type_pattern(&paren.pat, kind),
            PatKind::Wild => Ok(()),
            _ => bail!("Pattern {:?} not supported for type_pattern", pattern),
        }
    }
    fn bind_pattern(&mut self, pattern: &Pat) -> Result<()> {
        match &pattern.kind {
            PatKind::Ident(ident) => {
                self.bind(&ident.name, pattern.id);
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                for element in &tuple.elements {
                    self.bind_pattern(element)?;
                }
                Ok(())
            }
            PatKind::Type(type_pat) => {
                self.bind_pattern(&type_pat.pat)?;
                self.type_pattern(&type_pat.pat, &type_pat.kind)
            }
            PatKind::Struct(struct_pat) => {
                for field in &struct_pat.fields {
                    self.bind_pattern(&field.pat)?;
                }
                Ok(())
            }
            PatKind::TupleStruct(tuple_struct) => {
                for field in &tuple_struct.elems {
                    self.bind_pattern(field)?;
                }
                Ok(())
            }
            PatKind::Slice(slice) => {
                for element in &slice.elems {
                    self.bind_pattern(element)?;
                }
                Ok(())
            }
            PatKind::Paren(paren) => self.bind_pattern(&paren.pat),
            PatKind::Wild => Ok(()),
            _ => bail!("Pattern not supported"),
        }
    }
    // Handle the case of a bits or signed call with inferred length
    fn bits(&mut self, id: NodeId, bits: &ExprBits) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&bits.arg)?;
        match bits.kind {
            BitsKind::Unsigned => self.op(op_as_bits_inferred(lhs, arg), id),
            BitsKind::Signed => self.op(op_as_signed_inferred(lhs, arg), id),
        };
        Ok(lhs)
    }
    fn call(&mut self, id: NodeId, call: &ExprCall) -> Result<Slot> {
        let lhs = self.reg(id);
        let path = collapse_path(&call.path);
        let args = self.expr_list(&call.args)?;
        let Some(code) = &call.code else {
            bail!("No code for call {:?}", call)
        };
        // inline calls to bits and signed
        match code {
            KernelFnKind::BitConstructor(len) => self.op(op_as_bits(lhs, args[0], *len), id),
            KernelFnKind::SignedBitsConstructor(len) => {
                self.ty.insert(args[0], Kind::make_signed(128));
                self.op(op_as_signed(lhs, args[0], *len), id)
            }
            KernelFnKind::TupleStructConstructor(tb) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| rhif::spec::FieldValue {
                        value: *x,
                        member: rhif::spec::Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_struct(lhs, fields, None, tb.clone()), id);
            }
            KernelFnKind::EnumTupleStructConstructor(template) => {
                let fields = args
                    .iter()
                    .enumerate()
                    .map(|(ndx, x)| rhif::spec::FieldValue {
                        value: *x,
                        member: rhif::spec::Member::Unnamed(ndx as u32),
                    })
                    .collect();
                self.op(op_enum(lhs, fields, template.clone()), id);
            }
            KernelFnKind::Kernel(kernel) => {
                let func = self.stash(ExternalFunction {
                    code: ExternalFunctionCode::Kernel(kernel.clone()),
                    path: path.clone(),
                    signature: call.signature.clone().unwrap(),
                })?;
                self.op(op_exec(lhs, func, args), id);
            }
            KernelFnKind::Extern(code) => {
                let func = self.stash(ExternalFunction {
                    code: ExternalFunctionCode::Extern(code.clone()),
                    path: path.clone(),
                    signature: call.signature.clone().unwrap(),
                })?;
                self.op(op_exec(lhs, func, args), id);
            }
            KernelFnKind::SignalConstructor(color) => {
                self.op(op_retime(lhs, args[0], color.clone()), id);
            }
        }
        Ok(lhs)
    }

    fn expr_list(&mut self, exprs: &[Box<Expr>]) -> Result<Vec<Slot>> {
        exprs.iter().map(|expr| self.expr(expr)).collect()
    }
    fn expr(&mut self, expr: &Expr) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Array(array) => self.array(expr.id, array),
            ExprKind::Binary(bin) => self.binop(expr.id, bin),
            ExprKind::Block(block) => {
                let block_result = self.reg(expr.id);
                self.block(block_result, &block.block)?;
                Ok(block_result)
            }
            ExprKind::If(if_expr) => self.if_expr(expr.id, if_expr),
            ExprKind::Lit(lit) => Ok(self.lit(expr.id, lit.clone())),
            ExprKind::Field(field) => self.field(expr.id, field),
            ExprKind::Group(group) => self.expr(&group.expr),
            ExprKind::Index(index) => self.index(expr.id, index),
            ExprKind::Paren(paren) => self.expr(&paren.expr),
            ExprKind::Path(path) => self.path(path),
            ExprKind::Struct(_struct) => self.struct_expr(expr.id, _struct),
            ExprKind::Tuple(tuple) => self.tuple(expr.id, tuple),
            ExprKind::Unary(unary) => self.unop(expr.id, unary),
            ExprKind::Match(_match) => self.match_expr(expr.id, _match),
            ExprKind::Ret(_return) => self.return_expr(expr.id, _return),
            ExprKind::ForLoop(for_loop) => self.for_loop( for_loop),
            ExprKind::Assign(assign) => self.assign(expr.id, assign),
            ExprKind::Range(_) => bail!("Ranges are only supported in for loops"),
            ExprKind::Let(_) => bail!("Fallible let expressions are not currently supported in rhdl.  Use a match instead"),
            ExprKind::Repeat(repeat) => self.repeat(expr.id, repeat),
            ExprKind::Call(call) => self.call(expr.id, call),
            ExprKind::MethodCall(method) => self.method_call(expr.id, method),
            ExprKind::Type(_) => Ok(Slot::Empty),
            ExprKind::Bits(bits) => self.bits(expr.id, bits),
        }
    }
    // We need three components
    //  - the original variable that holds the LHS
    //  - the path to change (if any)
    //  - the new place to write the value
    // So, for example, if we have
    //   a[n] = b
    // Then we need to know:
    //  - The original binding of `a`
    //  - The path corresponding to `[n]`
    //  - The place to store the result of splicing `a[n]<-b` in a
    // new binding of the name `a`.
    fn expr_lhs(&mut self, expr: &Expr) -> Result<(Rebind, crate::path::Path)> {
        match &expr.kind {
            ExprKind::Path(path) => {
                let name = collapse_path(&path.path);
                let rebind = self.rebind(&name, expr.id)?;
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
    fn field(&mut self, id: NodeId, field: &ExprField) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&field.expr)?;
        let path = field.member.clone().into();
        eprintln!("field path {:?}", path);
        self.op(op_index(lhs, arg, path), id);
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<rhif::spec::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(rhif::spec::FieldValue {
            member: element.member.clone().into(),
            value,
        })
    }
    fn for_loop(&mut self, for_loop: &ExprForLoop) -> Result<Slot> {
        self.new_scope();
        let PatKind::Ident(loop_var) = &for_loop.pat.kind else {
            bail!("for loop with non-ident pattern is not supported");
        };
        self.bind_pattern(&for_loop.pat)?;
        let ExprKind::Range(range) = &for_loop.expr.kind else {
            bail!("for loop with non-range expression is not supported");
        };
        let Some(start) = &range.start else {
            bail!("for loop with no start value is not supported");
        };
        let Some(end) = &range.end else {
            bail!("for loop with no end value is not supported");
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(ExprLit::Int(start_lit)),
        } = start.as_ref()
        else {
            bail!("for loop with non-integer start value is not supported");
        };
        let Expr {
            id: _,
            kind: ExprKind::Lit(ExprLit::Int(end_lit)),
        } = end.as_ref()
        else {
            bail!("for loop with non-integer end value is not supported");
        };
        let start_lit = start_lit.parse::<i32>()?;
        let end_lit = end_lit.parse::<i32>()?;
        for ndx in start_lit..end_lit {
            let value = self.literal_int(for_loop.pat.id, ndx);
            self.rebind(&loop_var.name, for_loop.pat.id)?;
            self.initialize_local(&for_loop.pat, value)?;
            self.block(Slot::Empty, &for_loop.body)?;
        }
        self.end_scope();
        Ok(Slot::Empty)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf) -> Result<Slot> {
        let op_result = self.reg(id);
        let then_result = self.reg(id);
        let else_result = self.reg(id);
        let cond = self.expr(&if_expr.cond)?;
        let locals_prior_to_branch = self.locals();
        eprintln!("Locals prior to branch {:?}", locals_prior_to_branch);
        self.block(then_result, &if_expr.then_branch)?;
        let locals_after_then_branch = self.locals().clone();
        eprintln!("Locals after then branch {:?}", locals_after_then_branch);
        self.set_locals(&locals_prior_to_branch)?;
        if let Some(expr) = if_expr.else_branch.as_ref() {
            self.wrap_expr_in_block(else_result, expr)?;
        } else {
            self.op(op_assign(else_result, Slot::Empty), id);
        }
        let locals_after_else_branch = self.locals();
        self.set_locals(&locals_prior_to_branch)?;
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
        let post_branch_bindings: BTreeMap<ScopeIndex, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind(&x.name, id).map(|r| (x.clone(), r)))
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
            self.op(op_select(new_binding, cond, then_binding, else_binding), id);
        }
        self.op(op_select(op_result, cond, then_result, else_result), id);
        Ok(op_result)
    }
    fn index(&mut self, id: NodeId, index: &ExprIndex) -> Result<Slot> {
        let lhs = self.reg(id);
        let arg = self.expr(&index.expr)?;
        let index = self.expr(&index.index)?;
        self.op(
            op_index(lhs, arg, crate::path::Path::default().dynamic(index)),
            id,
        );
        Ok(lhs)
    }
    fn local(&mut self, local: &Local) -> Result<()> {
        let mut rhs = None;
        if let Some(init) = &local.init {
            rhs = Some(self.expr(init)?);
        }
        self.bind_pattern(&local.pat)?;
        if let Some(rhs) = rhs {
            self.initialize_local(&local.pat, rhs)?;
        }
        Ok(())
    }
    fn match_expr(&mut self, id: NodeId, match_expr: &ExprMatch) -> Result<Slot> {
        let lhs = self.reg(id);
        let target = self.expr(&match_expr.expr)?;
        let discriminant = self.reg(id);
        self.op(
            op_index(
                discriminant,
                target,
                crate::path::Path::default().discriminant(),
            ),
            id,
        );
        // Need to handle local rebindings in the bodies of the arms.
        let locals_prior_to_match = self.locals().clone();
        let mut arguments = vec![];
        let mut arm_locals = vec![];
        let mut arm_lhs = vec![];
        for arm in &match_expr.arms {
            self.set_locals(&locals_prior_to_match)?;
            let lhs = self.reg(id);
            let disc = self.arm(target, lhs, arm)?;
            arm_lhs.push(lhs);
            arguments.push(disc);
            arm_locals.push(self.locals().clone());
        }
        self.set_locals(&locals_prior_to_match)?;
        let mut rebound_locals = BTreeSet::new();
        for branch_locals in &arm_locals {
            let branch_rebindings = get_locals_changed(&self.locals(), branch_locals)?;
            rebound_locals.extend(branch_rebindings);
        }
        // Next, for each local variable in rebindings, we need a new
        // binding for that variable in the current scope.
        let post_branch_bindings: BTreeMap<ScopeIndex, Rebind> = rebound_locals
            .iter()
            .map(|x| self.rebind(&x.name, id).map(|r| (x.clone(), r)))
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
            self.op(op_case(new_binding, discriminant, cases), id);
        }
        let match_expr_table = arguments.iter().cloned().zip(arm_lhs).collect::<Vec<_>>();
        self.op(op_case(lhs, discriminant, match_expr_table), id);
        Ok(lhs)
    }
    fn method_call(&mut self, id: NodeId, method_call: &ExprMethodCall) -> Result<Slot> {
        // The `val` method is a special case used to strip the clocking context
        // from a signal.
        if method_call.method.as_str() == "val" {
            let lhs = self.reg(id);
            let arg = self.expr(&method_call.receiver)?;
            self.op(op_assign(lhs, arg), id);
            /*
            self.op(
                op_index(lhs, arg, crate::path::Path::default().signal_value()),
                id,
            );*/
            return Ok(lhs);
        }
        // First handle unary ops only
        let op = match method_call.method.as_str() {
            "any" => AluUnary::Any,
            "all" => AluUnary::All,
            "xor" => AluUnary::Xor,
            "as_unsigned" => AluUnary::Unsigned,
            "as_signed" => AluUnary::Signed,
            _ => bail!("Unsupported method call {:?}", method_call),
        };
        let lhs = self.reg(id);
        let arg = self.expr(&method_call.receiver)?;
        self.op(op_unary(op, lhs, arg), id);
        Ok(lhs)
    }
    fn path(&mut self, path: &ExprPath) -> Result<Slot> {
        if path.path.segments.len() == 1 && path.path.segments[0].arguments.is_empty() {
            let name = &path.path.segments[0].ident;
            return self
                .lookup_name(name)
                .map(|x| x.0)
                .ok_or(anyhow!("ICE - name not found: {}", name));
        }
        bail!("ICE - path with arguments not supported {:?}", path)
    }
    fn repeat(&mut self, id: NodeId, repeat: &ExprRepeat) -> Result<Slot> {
        let lhs = self.reg(id);
        let value = self.expr(&repeat.value)?;
        self.op(op_repeat(lhs, value, repeat.len as _), id);
        Ok(lhs)
    }
    fn return_expr(&mut self, id: NodeId, return_expr: &ExprRet) -> Result<Slot> {
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

        let literal_true = self.literal_bool(id, true);
        let early_return_flag = self.rebind(EARLY_RETURN_FLAG_NAME, id)?;
        let name = self.name.clone();
        let return_slot = self.rebind(&name, id)?;
        let early_return_expr = if let Some(return_expr) = &return_expr.expr {
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
        self.op(
            op_select(
                return_slot.to,
                early_return_flag.from,
                return_slot.from,
                early_return_expr,
            ),
            id,
        );
        self.op(
            op_select(
                early_return_flag.to,
                early_return_flag.from,
                early_return_flag.from,
                literal_true,
            ),
            id,
        );
        Ok(Slot::Empty)
    }
    fn stmt(&mut self, statement: &Stmt) -> Result<Slot> {
        let statement_text = pretty_print_statement(statement)?;
        self.op(op_comment(statement_text), statement.id);
        match &statement.kind {
            StmtKind::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            StmtKind::Expr(expr) => self.expr(expr),
            StmtKind::Semi(expr) => {
                self.expr(expr)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn struct_expr(&mut self, id: NodeId, strukt: &ExprStruct) -> Result<Slot> {
        eprintln!("Struct expr {:?} template: {}", strukt, strukt.template);
        let lhs = self.reg(id);
        let fields = strukt
            .fields
            .iter()
            .map(|x| self.field_value(x))
            .collect::<Result<_>>()?;
        let rest = strukt.rest.as_ref().map(|x| self.expr(x)).transpose()?;
        if let Kind::Enum(_enum) = &strukt.template.kind {
            eprintln!("Emitting enum opcode");
            self.op(op_enum(lhs, fields, strukt.template.clone()), id);
        } else {
            eprintln!("Emitting struct opcode");
            self.op(op_struct(lhs, fields, rest, strukt.template.clone()), id);
        }
        Ok(lhs)
    }
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple) -> Result<Slot> {
        let elements = self.expr_list(&tuple.elements)?;
        let result = self.reg(id);
        self.op(op_tuple(result, elements), id);
        Ok(result)
    }
    fn unop(&mut self, id: NodeId, unary: &ExprUnary) -> Result<Slot> {
        let arg = self.expr(&unary.expr)?;
        let result = self.reg(id);
        let op = unary_op_to_alu(unary.op);
        self.op(op_unary(op, result, arg), id);
        Ok(result)
    }
    fn wrap_expr_in_block(&mut self, block_result: Slot, expr: &Expr) -> Result<()> {
        let result = self.expr(expr)?;
        if block_result != result {
            self.op(op_assign(block_result, result), expr.id);
        }
        Ok(())
    }
}

impl Visitor for MirContext {
    fn visit_kernel_fn(&mut self, node: &ast_impl::KernelFn) -> Result<()> {
        self.unpack_arguments(&node.inputs)?;
        let block_result = self.reg(node.id);
        self.return_slot = block_result;
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
        self.bind(EARLY_RETURN_FLAG_NAME, node.id);
        self.bind(&node.name, node.id);
        self.name.clone_from(&node.name);
        // Initialize the early exit flag in the main block
        let init_early_exit_op = op_assign(
            self.lookup_name(EARLY_RETURN_FLAG_NAME).unwrap().0,
            self.literal_bool(node.id, false),
        );
        // Initialize the return slot in the main block
        let init_return_slot = op_assign(
            self.lookup_name(&node.name).unwrap().0,
            self.literal_tb(node.id, &node.ret.place_holder()),
        );
        // Initialize the arguments in the main block
        for (arg, slot) in node.inputs.iter().zip(self.arguments.clone().iter()) {
            self.bind_pattern(arg)?;
            self.initialize_local(arg, *slot)?;
        }
        self.block(block_result, &node.body)?;
        self.ops.insert(0, (init_early_exit_op, node.id).into());
        self.ops.insert(1, (init_return_slot, node.id).into());
        self.insert_implicit_return(node.body.id, block_result, &node.name)?;
        self.return_slot = self
            .lookup_name(&node.name)
            .ok_or(anyhow!("ICE - return slot not found"))?
            .0;
        self.fn_id = node.fn_id;
        Ok(())
    }
}

fn get_locals_changed(from: &LocalsMap, to: &LocalsMap) -> Result<BTreeSet<ScopeIndex>> {
    from.iter()
        .filter_map(|(id, slot)| {
            {
                if let Some(to_slot) = to.get(id) {
                    Ok(if to_slot != slot {
                        Some(id.clone())
                    } else {
                        None
                    })
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

pub fn compile_mir(mut func: Kernel) -> Result<Mir> {
    let mut generator = NodeIdGenerator::default();
    generator.visit_mut_kernel_fn(func.inner_mut())?;
    let mut compiler = MirContext::default();
    compiler.visit_kernel_fn(func.inner())?;
    compiler
        .ty
        .insert(compiler.return_slot, func.inner().ret.clone());
    let source = build_spanned_source_for_kernel(func.inner());
    let opcode_map = compiler
        .ops
        .iter()
        .map(|x| (compiler.fn_id, x.source).into())
        .collect();
    let slot_map = compiler
        .reg_source_map
        .into_iter()
        .map(|(slot, node)| (slot, (compiler.fn_id, node).into()))
        .collect();
    Ok(Mir {
        symbols: SymbolMap {
            slot_map,
            opcode_map,
            source,
        },
        ops: compiler.ops,
        arguments: compiler.arguments,
        literals: compiler.literals,
        return_slot: compiler.return_slot,
        fn_id: compiler.fn_id,
        ty: compiler.ty,
        ty_equate: compiler.ty_equate,
        stash: compiler.stash,
        name: compiler.name,
    })
}
