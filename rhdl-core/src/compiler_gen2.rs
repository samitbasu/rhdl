use crate::{
    ast::{self, BinOp, Expr, ExprBinary, ExprIf, ExprKind, Local, NodeId, Pat, PatKind},
    display_ast::pretty_print_statement,
    infer_types::id_to_var,
    rhif::{AluBinary, AluUnary, BlockId, OpCode, Slot},
    ty::{Ty, TypeId},
    unify::UnifyContext,
    visit::{self, visit_block, Visitor},
};
use anyhow::{bail, Result};
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

pub struct CompilerContext {
    pub blocks: Vec<Block>,
    pub literals: Vec<Literal>,
    pub reg_count: usize,
    active_block: BlockId,
    type_context: UnifyContext,
    ty: HashMap<Slot, Ty>,
    regs: HashMap<NodeId, Slot>,
    rev_regs: HashMap<Slot, NodeId>,
}

pub struct Literal {
    pub value: Box<ast::ExprLit>,
    pub ty: Ty,
}

impl std::fmt::Display for CompilerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for regs in self.ty.keys() {
            writeln!(
                f,
                "Reg r{} = {} {}",
                regs.reg().unwrap(),
                self.ty[regs],
                self.rev_regs[regs]
            )?;
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(
                f,
                "Literal l{} = {:?} <{:?}>",
                ndx, literal.value, literal.ty
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
            regs: Default::default(),
            rev_regs: Default::default(),
        }
    }
    fn reg(&mut self, id: Option<NodeId>) -> Result<Slot> {
        let id = id.ok_or(anyhow::anyhow!("No node id"))?;
        if self.regs.contains_key(&id) {
            return Ok(self.regs[&id]);
        }
        let var = id_to_var(Some(id))?;
        let ty = self.type_context.apply(var);
        let reg = Slot::Register(self.reg_count);
        self.ty.insert(reg, ty);
        self.regs.insert(id, reg);
        self.rev_regs.insert(reg, id);
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
    fn ty(&self, id: Option<NodeId>) -> Result<Ty> {
        let var = id_to_var(id)?;
        Ok(self.type_context.apply(var))
    }
    fn lookup(&mut self, child: Option<NodeId>) -> Result<Slot> {
        let child_id = id_to_var(child)?;
        if let Ty::Var(child_id) = child_id {
            if let Some(parent) = self.type_context.get_parent(child_id) {
                let parent_node: Option<NodeId> = Some(parent.into());
                return self.reg(parent_node);
            }
        }
        bail!("No parent for {:?}", child_id)
    }
    fn unop(&mut self, id: Option<NodeId>, unary: &ast::ExprUnary) -> Result<Slot> {
        let arg = self.expr(&unary.expr)?;
        let result = self.reg(id)?;
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
            ast::StmtKind::Local(local) => self.local(&local),
            ast::StmtKind::Expr(expr) => self.expr(&expr),
            ast::StmtKind::Semi(expr) => {
                self.expr(&expr)?;
                Ok(Slot::Empty)
            }
        }
    }
    fn local(&mut self, local: &Local) -> Result<Slot> {
        todo!()
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
    fn if_expr(&mut self, id: Option<NodeId>, if_expr: &ExprIf) -> Result<Slot> {
        let result = self.reg(id)?;
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
    fn binop(&mut self, id: Option<NodeId>, bin: &ExprBinary) -> Result<Slot> {
        let lhs = self.expr(&bin.lhs)?;
        let rhs = self.expr(&bin.rhs)?;
        let result = self.reg(id)?;
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
    fn expr_lhs(&mut self, expr: &Expr) {}
    fn expr(&mut self, expr: &Expr) -> Result<Slot> {
        match &expr.kind {
            ExprKind::Binary(bin) => self.binop(expr.id, bin),
            ExprKind::Unary(unary) => self.unop(expr.id, unary),
            ExprKind::Path(_path) => self.lookup(expr.id),
            ExprKind::Block(block) => {
                let block_result = self.reg(expr.id)?;
                let block_id = self.block(block_result, &block.block)?;
                self.op(OpCode::Block(block_id));
                Ok(block_result)
            }
            ExprKind::If(if_expr) => self.if_expr(expr.id, if_expr),
            _ => todo!("expr {:?}", expr),
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
}

fn argument_id(arg_pat: &Pat) -> Result<Option<NodeId>> {
    match &arg_pat.kind {
        PatKind::Ident(name) => Ok(arg_pat.id),
        PatKind::Type(ty) => argument_id(&ty.pat),
        _ => {
            bail!("Arguments to kernel functions must be identifiers, instead got {arg_pat:?}")
        }
    }
}

impl Visitor for CompilerContext {
    fn visit_kernel_fn(&mut self, node: &ast::KernelFn) -> Result<()> {
        // Allocate a register for each argument
        for arg in &node.inputs {
            let arg_reg = self.reg(argument_id(arg)?)?;
            eprintln!("Argument {:?} is in register {:?}", arg, arg_reg);
        }
        let block_result = self.reg(node.id)?;
        let _ = self.block(block_result, &node.body)?;
        Ok(())
    }
}
