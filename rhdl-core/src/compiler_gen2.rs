use crate::{
    ast::{
        self, BinOp, Expr, ExprBinary, ExprIf, ExprKind, ExprTuple, FieldValue, Local, NodeId, Pat,
        PatKind, Path,
    },
    display_ast::pretty_print_statement,
    infer_types::id_to_var,
    rhif::{self, AluBinary, AluUnary, BlockId, OpCode, Slot},
    ty::{Ty, TypeId},
    unify::UnifyContext,
    visit::{self, visit_block, Visitor},
};
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

const ROOT_BLOCK: BlockId = BlockId(0);

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
    pub literals: Vec<Literal>,
    pub reg_count: usize,
    active_block: BlockId,
    type_context: UnifyContext,
    ty: HashMap<Slot, Ty>,
    locals: HashMap<TypeId, NamedSlot>,
}

pub struct Literal {
    pub value: Box<ast::ExprLit>,
    pub ty: Ty,
}

impl std::fmt::Display for CompilerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for regs in self.ty.keys() {
            writeln!(f, "Reg r{} : {}", regs.reg().unwrap(), self.ty[regs],)?;
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(f, "Literal l{} : {} = {:?}", ndx, literal.ty, literal.value)?;
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
    pub fn new(type_context: UnifyContext) -> Self {
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
            ty: Default::default(),
            locals: Default::default(),
        }
    }
    fn node_ty(&self, id: NodeId) -> Result<Ty> {
        let var = id_to_var(id)?;
        Ok(self.type_context.apply(var))
    }
    fn reg(&mut self, ty: Ty) -> Result<Slot> {
        let reg = Slot::Register(self.reg_count);
        self.ty.insert(reg, ty);
        self.reg_count += 1;
        Ok(reg)
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
                        "No local variable for {:?} in {:?}",
                        child_id,
                        self.locals
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
    fn unop(&mut self, id: NodeId, unary: &ast::ExprUnary) -> Result<Slot> {
        let arg = self.expr(&unary.expr)?;
        let result = self.reg(self.node_ty(id)?)?;
        let op = match unary.op {
            ast::UnOp::Neg => AluUnary::Neg,
            ast::UnOp::Not => AluUnary::Not,
        };
        self.op(OpCode::Unary {
            op,
            lhs: result,
            arg1: arg,
        });
        Ok(result)
    }
    fn stmt(&mut self, statement: &ast::Stmt) -> Result<Slot> {
        let statement_text = pretty_print_statement(statement, &self.type_context)?;
        self.op(OpCode::Comment(statement_text));
        match &statement.kind {
            ast::StmtKind::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            ast::StmtKind::Expr(expr) => self.expr(expr),
            ast::StmtKind::Semi(expr) => {
                self.expr(expr)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn local(&mut self, local: &Local) -> Result<()> {
        self.bind_pattern(&local.pat)?;
        match &local.pat.kind {
            PatKind::Ident(ident) => {
                if let Some(expr) = &local.init {
                    let result = self.expr(expr)?;
                    let lhs = self.resolve_local(local.pat.id)?;
                    self.op(OpCode::Copy { lhs, rhs: result });
                }
                Ok(())
            }
            PatKind::Tuple(tuple) => {
                if let Some(expr) = &local.init {
                    let rhs = self.expr(expr)?;
                    for (ndx, pat) in tuple.elements.iter().enumerate() {
                        let element_lhs = self.resolve_local(pat.id)?;
                        self.op(OpCode::Field {
                            lhs: element_lhs,
                            arg: rhs,
                            member: rhif::Member::Unnamed(ndx as u32),
                        });
                    }
                }
                Ok(())
            }
            _ => todo!("Unsupported let pattern: {:?}", local.pat),
        }
    }
    fn block(&mut self, block_result: Slot, block: &ast::Block) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(block_result);
        let statement_count = block.stmts.len();
        for (ndx, statement) in block.stmts.iter().enumerate() {
            let is_last = ndx == statement_count - 1;
            let result = self.stmt(statement)?;
            if is_last {
                self.op(OpCode::Copy {
                    lhs: block_result,
                    rhs: result,
                })
            }
        }
        self.set_active_block(current_block);
        Ok(id)
    }
    fn expr_list(&mut self, exprs: &[Box<Expr>]) -> Result<Vec<Slot>> {
        exprs
            .into_iter()
            .map(|x| self.expr(&x))
            .collect::<Result<_>>()
    }
    fn tuple(&mut self, id: NodeId, tuple: &ExprTuple) -> Result<Slot> {
        let result = self.reg(self.node_ty(id)?)?;
        let fields = self.expr_list(&tuple.elements)?;
        self.op(OpCode::Tuple {
            lhs: result,
            fields,
        });
        Ok(result)
    }
    fn if_expr(&mut self, id: NodeId, if_expr: &ExprIf) -> Result<Slot> {
        let result = self.reg(self.node_ty(id)?)?;
        let cond = self.expr(&if_expr.cond)?;
        let then_branch = self.block(result, &if_expr.then_branch)?;
        let else_branch = self.wrap_expr_in_block(result, &if_expr.else_branch)?;
        self.op(OpCode::If {
            cond,
            then_branch,
            else_branch,
            lhs: result,
        });
        Ok(result)
    }
    fn index(&mut self, id: NodeId, index: &ast::ExprIndex) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let arg = self.expr(&index.expr)?;
        let index = self.expr(&index.index)?;
        self.op(OpCode::Index { lhs, arg, index });
        Ok(lhs)
    }
    fn array(&mut self, id: NodeId, array: &ast::ExprArray) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let elements = self.expr_list(&array.elems)?;
        self.op(OpCode::Array { lhs, elements });
        Ok(lhs)
    }
    fn field(&mut self, id: NodeId, field: &ast::ExprField) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let arg = self.expr(&field.expr)?;
        let member = match &field.member {
            ast::Member::Named(name) => rhif::Member::Named(name.to_string()),
            ast::Member::Unnamed(ndx) => rhif::Member::Unnamed(*ndx),
        };
        self.op(OpCode::Field { lhs, arg, member });
        Ok(lhs)
    }
    fn field_value(&mut self, element: &FieldValue) -> Result<rhif::FieldValue> {
        let value = self.expr(&element.value)?;
        Ok(rhif::FieldValue {
            value,
            member: element.member.clone().into(),
        })
    }
    fn struct_expr(&mut self, id: NodeId, _struct: &ast::ExprStruct) -> Result<Slot> {
        let lhs = self.reg(self.node_ty(id)?)?;
        let fields = _struct
            .fields
            .iter()
            .map(|x| self.field_value(&x))
            .collect::<Result<_>>()?;
        self.op(OpCode::Struct {
            lhs,
            path: collapse_path(&_struct.path),
            fields,
            rest: None,
        });
        Ok(lhs)
    }
    fn binop(&mut self, id: NodeId, bin: &ExprBinary) -> Result<Slot> {
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let result = self.reg(self.node_ty(id)?)?;
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
        let alu = match op {
            BinOp::Add | BinOp::AddAssign => AluBinary::Add,
            BinOp::Sub | BinOp::SubAssign => AluBinary::Sub,
            BinOp::Mul | BinOp::MulAssign => AluBinary::Mul,
            BinOp::And => AluBinary::And,
            BinOp::Or => AluBinary::Or,
            BinOp::BitXor | BinOp::BitXorAssign => AluBinary::BitXor,
            BinOp::BitAnd | BinOp::BitAndAssign => AluBinary::BitAnd,
            BinOp::BitOr | BinOp::BitOrAssign => AluBinary::BitOr,
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
        self.op(OpCode::Binary {
            op: alu,
            lhs: result,
            arg1: lhs,
            arg2: rhs,
        });
        Ok(result)
    }
    fn expr_lhs(&mut self, expr: &Expr) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Path(_path) => {
                todo!(); //self.lookup(expr.id);
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
                self.literals.push(Literal {
                    value: Box::new(lit.clone()),
                    ty,
                });
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
        }
    }
    fn wrap_expr_in_block(
        &mut self,
        block_result: Slot,
        expr: &Option<Box<Expr>>,
    ) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(block_result);
        let result = if let Some(expr) = expr {
            self.expr(expr)?
        } else {
            Slot::Empty
        };
        self.op(OpCode::Copy {
            lhs: block_result,
            rhs: result,
        });
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
            _ => bail!("Unsupported pattern {:?}", pattern),
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
    fn visit_kernel_fn(&mut self, node: &ast::KernelFn) -> Result<()> {
        // Allocate local binding for each argument
        for arg in &node.inputs {
            let (arg_name, arg_id) = argument_id(arg)?;
            self.bind(arg_id, &arg_name)?;
        }
        let block_result = self.reg(self.ty(node.id)?)?;
        let _ = self.block(block_result, &node.body)?;
        Ok(())
    }
}
