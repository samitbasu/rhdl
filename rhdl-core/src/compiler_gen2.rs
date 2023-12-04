use crate::{
    ast::{self, BinOp, Expr, ExprBinary, ExprKind, NodeId, Pat, PatKind},
    infer_types::id_to_var,
    rhif::{AluBinary, BlockId, OpCode, Slot},
    ty::{Ty, TypeId},
    unify::UnifyContext,
    visit::{self, Visitor},
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
    fn binop(&mut self, id: Option<NodeId>, bin: &ExprBinary) -> Result<()> {
        let lhs = self.reg(bin.lhs.id)?;
        let rhs = self.reg(bin.rhs.id)?;
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
        self.op(OpCode::Binary {
            op: alu,
            lhs: result,
            arg1: lhs,
            arg2: rhs,
        });
        assert!(!self_assign);
        Ok(())
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
        // Allocate a register to hold the return type
        let ret_reg = self.reg(node.id)?;
        // Create a new block with ret_reg set as the
        // return slot for the block
        let id = self.new_block(ret_reg);
        self.set_active_block(id);
        // Allocate a register for each argument
        for arg in &node.inputs {
            let arg_reg = self.reg(argument_id(arg)?)?;
            eprintln!("Argument {:?} is in register {:?}", arg, arg_reg);
        }
        for stmt in &node.body.stmts {
            self.visit_stmt(stmt)?;
        }
        Ok(())
    }
    fn visit_expr(&mut self, node: &Expr) -> Result<()> {
        visit::visit_expr(self, node)?;
        match &node.kind {
            ExprKind::Binary(bin) => self.binop(node.id, bin),
            ExprKind::Path(_path) => {
                let parent = self.lookup(node.id)?;
                let result = self.reg(node.id)?;
                self.op(OpCode::Copy {
                    lhs: result,
                    rhs: parent,
                });
                Ok(())
            }
            _ => bail!("Unsupported expression: {:?}", node),
        }
    }
    fn visit_local(&mut self, node: &ast::Local) -> Result<()> {
        visit::visit_local(self, node)?;
        let reg = self.reg(node.pat.id)?;
        if let Some(init) = &node.init {
            let rhs = self.reg(init.id)?;
            self.op(OpCode::Copy { lhs: reg, rhs })
        }
        Ok(())
    }
}
