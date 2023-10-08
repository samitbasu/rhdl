// RHDL Intermediate Form (RHIF).
use crate::ast;
use crate::ast::{BinOp, UnOp};

use std::cell::Ref;
use std::{collections::HashMap, ops::Range};

use crate::{ast::ExprLit, digital::TypedBits};
use anyhow::{bail, Result};

pub enum OpCode {
    // x <- a op b
    Binary(BinaryOp),
    // x <- op a
    Unary(UnaryOp),
    // return a
    Return(Option<Slot>),
    // if cond { then_branch } else { else_branch }
    If(IfOp),
    // x <- {block}
    Block(BlockOp),
    // x <- a[i]
    Index(IndexOp),
    // x <- a
    Assign(AssignOp),
    // x <- a.field
    Field(FieldOp),
    // x <- [a; count]
    Repeat(RepeatOp),
    // x <- Struct { fields }
    Struct(StructOp),
    // x <- Tuple(fields)
    Tuple(TupleOp),
    // x <- match a { arms }
    Match(MatchOp),
    // x = &a
    Ref(RefOp),
    // x = &a.field
    FieldRef(FieldRefOp),
    // x = &a[i]
    IndexRef(IndexRefOp),
}

pub struct RefOp {
    pub lhs: Slot,
    pub arg: Slot,
}

pub struct IndexRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

pub struct FieldRefOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

pub struct MatchOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub arms: Vec<MatchArm>,
}

pub struct MatchArm {
    //    pub pattern: MatchPattern,
    pub body: BlockOp,
}

pub struct TupleOp {
    pub lhs: Slot,
    pub fields: Vec<Slot>,
}
pub struct StructOp {
    pub lhs: Slot,
    pub fields: Vec<FieldValue>,
    pub rest: Option<Slot>,
}

pub struct FieldValue {
    pub member: Member,
    pub value: Slot,
}

pub struct RepeatOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub count: Slot,
}
pub struct FieldOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub member: Member,
}

pub struct AssignOp {
    pub lhs: Slot,
    pub rhs: Slot,
}
pub struct IndexOp {
    pub lhs: Slot,
    pub arg: Slot,
    pub index: Slot,
}

pub struct BlockOp {
    pub lhs: Slot,
    pub body: Vec<OpCode>,
}

pub struct IfOp {
    pub lhs: Slot,
    pub cond: Slot,
    pub then_branch: BlockId,
    pub else_branch: BlockId,
}

pub struct BinaryOp {
    pub op: AluBinary,
    pub lhs: Slot,
    pub arg1: Slot,
    pub arg2: Slot,
}

pub struct UnaryOp {
    pub op: AluUnary,
    pub lhs: Slot,
    pub arg1: Slot,
}

pub enum AluBinary {
    Add,
    Sub,
    Mul,
    And,
    Or,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

pub enum AluUnary {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Slot {
    Literal(ExprLit),
    Register(usize),
    Empty,
}

pub enum Member {
    Named(String),
    Unnamed(u32),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

const ROOT_BLOCK: BlockId = BlockId(0);

pub struct Block {
    pub id: BlockId,
    pub names: HashMap<String, Slot>,
    pub ops: Vec<OpCode>,
    pub result: Slot,
    pub children: Vec<BlockId>,
    pub parent: BlockId,
}

pub struct Context {
    pub blocks: Vec<Block>,
    pub reg_count: usize,
    pub active_block: BlockId,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            blocks: vec![Block {
                id: ROOT_BLOCK,
                names: Default::default(),
                ops: vec![],
                result: Slot::Empty,
                children: vec![],
                parent: ROOT_BLOCK,
            }],
            reg_count: 0,
            active_block: ROOT_BLOCK,
        }
    }
}

impl Context {
    pub fn reg(&mut self) -> Slot {
        let reg = self.reg_count;
        self.reg_count += 1;
        Slot::Register(reg)
    }
    pub fn bind(&mut self, name: &str) -> Slot {
        let reg = self.reg();
        self.blocks[self.active_block.0]
            .names
            .insert(name.to_string(), reg.clone());
        reg
    }
    pub fn get_reference(&mut self, path: &str) -> Result<Slot> {
        let mut ip = self.active_block;
        loop {
            if let Some(slot) = self.blocks[ip.0].names.get(path) {
                return Ok(slot.clone());
            }
            if self.active_block == ROOT_BLOCK {
                break;
            }
            ip = self.blocks[ip.0].parent;
        }
        bail!("Unknown path {}", path);
    }
    pub fn op(&mut self, op: OpCode) {
        self.blocks[self.active_block.0].ops.push(op);
    }
    pub fn new_block(&mut self, result: Slot) -> BlockId {
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
    fn block_result(&self, id: BlockId) -> Slot {
        self.blocks[id.0].result.clone()
    }
}

pub fn compile(ast: crate::ast::Block) -> Result<Context> {
    let mut context = Context::default();
    let lhs = context.reg();
    expr_block(&mut context, ast, lhs)?;
    Ok(context)
}

pub fn expr(ctx: &mut Context, expr_: ast::Expr) -> Result<Slot> {
    match expr_ {
        ast::Expr::Lit(lit) => expr_literal(ctx, lit),
        ast::Expr::Unary(unop) => expr_unop(ctx, unop),
        ast::Expr::Binary(binop) => expr_binop(ctx, binop),
        ast::Expr::Block(block) => {
            let lhs = ctx.reg();
            expr_block(ctx, block, lhs.clone())?;
            Ok(lhs)
        }
        //        ast::Expr::If(if_expr) => expr_if(ctx, if_expr),
        ast::Expr::Assign(assign) => expr_assign(ctx, assign),
        ast::Expr::Paren(paren) => expr(ctx, *paren),
        _ => todo!(),
    }
}

pub fn expr_lhs(ctx: &mut Context, expr_: ast::Expr) -> Result<Slot> {
    match expr_ {
        ast::Expr::Path(path) => {
            if path.path.len() != 1 {
                bail!("Invalid path for assignment in RHDL code");
            }
            let arg = ctx.get_reference(path.path.last().unwrap())?;
            let lhs = ctx.reg();
            ctx.op(OpCode::Ref(RefOp {
                lhs: lhs.clone(),
                arg,
            }));
            return Ok(lhs);
        }
        ast::Expr::Field(field) => {
            let lhs = ctx.reg();
            let arg = expr_lhs(ctx, *field.expr)?;
            let member = match field.member {
                ast::Member::Named(name) => Member::Named(name),
                ast::Member::Unnamed(index) => Member::Unnamed(index),
            };
            ctx.op(OpCode::FieldRef(FieldRefOp {
                lhs: lhs.clone(),
                arg,
                member,
            }));
            return Ok(lhs);
        }
        ast::Expr::Index(index) => {
            let lhs = ctx.reg();
            let arg = expr_lhs(ctx, *index.expr)?;
            let index = expr(ctx, *index.index)?;
            ctx.op(OpCode::IndexRef(IndexRefOp {
                lhs: lhs.clone(),
                arg,
                index,
            }));
            return Ok(lhs);
        }
        _ => todo!(),
    }
}

pub fn stmt(ctx: &mut Context, statement: ast::Stmt) -> Result<Slot> {
    dbg!(&statement);
    match statement {
        ast::Stmt::Local(local) => {
            todo!()
        }
        ast::Stmt::Expr(expr_) => expr(ctx, expr_),
        ast::Stmt::Semi(expr_) => {
            expr(ctx, expr_)?;
            Ok(Slot::Empty)
        }
    }
}

pub fn expr_if(ctx: &mut Context, if_expr: crate::ast::ExprIf) -> Result<Slot> {
    let lhs = ctx.reg();
    let cond = expr(ctx, *if_expr.cond)?;
    let then_branch = expr_block(ctx, if_expr.then_branch, lhs.clone())?;
    // Create a block containing the else part of the if expression
    let else_block = if_expr
        .else_branch
        .map(|x| ast::Block(vec![ast::Stmt::Expr(*x)]))
        .unwrap_or(ast::Block(vec![]));
    let else_branch = expr_block(ctx, else_block, lhs.clone())?;
    ctx.op(OpCode::If(IfOp {
        lhs: lhs.clone(),
        cond,
        then_branch,
        else_branch,
    }));
    Ok(lhs)
}

// Start simple.
// If an expression is <ExprLit> then stuff it into a Slot
pub fn expr_literal(ctx: &mut Context, lit: crate::ast::ExprLit) -> Result<Slot> {
    Ok(Slot::Literal(lit))
}

pub fn expr_assign(ctx: &mut Context, assign: ast::ExprAssign) -> Result<Slot> {
    let lhs = expr_lhs(ctx, *assign.lhs)?;
    let rhs = expr(ctx, *assign.rhs)?;
    ctx.op(OpCode::Assign(AssignOp { lhs, rhs }));
    Ok(Slot::Empty)
}

pub fn expr_unop(ctx: &mut Context, unop: crate::ast::ExprUnary) -> Result<Slot> {
    let arg = expr(ctx, *unop.expr)?;
    let dest = ctx.reg();
    let alu = match unop.op {
        UnOp::Neg => AluUnary::Neg,
        UnOp::Not => AluUnary::Not,
    };
    ctx.op(OpCode::Unary(UnaryOp {
        op: alu,
        lhs: dest.clone(),
        arg1: arg,
    }));
    Ok(dest)
}

pub fn expr_binop(ctx: &mut Context, binop: crate::ast::ExprBinary) -> Result<Slot> {
    // Allocate a register for the output
    let lhs = expr(ctx, *binop.lhs)?;
    let rhs = expr(ctx, *binop.rhs)?;
    let self_assign = matches!(
        binop.op,
        BinOp::AddAssign
            | BinOp::SubAssign
            | BinOp::MulAssign
            | BinOp::BitXorAssign
            | BinOp::BitAndAssign
            | BinOp::ShlAssign
            | BinOp::BitOrAssign
            | BinOp::ShrAssign
    );
    let alu = match binop.op {
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
    let dest = if self_assign { lhs.clone() } else { ctx.reg() };
    ctx.op(OpCode::Binary(BinaryOp {
        op: alu,
        lhs: dest.clone(),
        arg1: lhs,
        arg2: rhs,
    }));
    Ok(dest)
}

pub fn expr_block(ctx: &mut Context, block: crate::ast::Block, lhs: Slot) -> Result<BlockId> {
    let statement_count = block.0.len();
    let id = ctx.new_block(lhs.clone());
    // process each statement in the block and return the last one
    for (ndx, stmt_) in block.0.iter().enumerate() {
        if ndx == statement_count - 1 {
            let rhs = stmt(ctx, stmt_.clone())?;
            let lhs_addr = ctx.reg();
            ctx.op(OpCode::Ref(RefOp {
                lhs: lhs_addr.clone(),
                arg: lhs.clone(),
            }));
            ctx.op(OpCode::Assign(AssignOp {
                lhs: lhs_addr.clone(),
                rhs,
            }));
        } else {
            stmt(ctx, stmt_.clone())?;
        }
    }
    Ok(id)
}
