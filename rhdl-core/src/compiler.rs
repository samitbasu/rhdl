use std::collections::BTreeMap;
use std::{collections::HashMap, fmt::Display};

use crate::ast::{self, BinOp, UnOp};
use crate::rhif::{
    AluBinary, AluUnary, AssignOp, BinaryOp, BlockId, CopyOp, FieldOp, FieldRefOp, IfOp,
    IndexRefOp, Member, OpCode, RefOp, Slot, TupleOp, UnaryOp,
};
use crate::Kind;
use anyhow::bail;
use anyhow::Result;

const ROOT_BLOCK: BlockId = BlockId(0);

pub struct Block {
    pub id: BlockId,
    pub names: HashMap<String, Slot>,
    pub ops: Vec<OpCode>,
    pub result: Slot,
    pub children: Vec<BlockId>,
    pub parent: BlockId,
}

pub struct Compiler {
    pub blocks: Vec<Block>,
    pub reg_count: usize,
    pub active_block: BlockId,
    pub types: BTreeMap<usize, Kind>,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ndx, kind) in &self.types {
            writeln!(f, "Type r{} = {:?}", ndx, kind)?;
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

impl Default for Compiler {
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
            types: Default::default(),
        }
    }
}

impl Compiler {
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
        self.active_block = id;
        id
    }
    fn current_block(&self) -> BlockId {
        self.active_block
    }
    fn set_block(&mut self, id: BlockId) {
        self.active_block = id;
    }
    fn block_result(&self, id: BlockId) -> Slot {
        self.blocks[id.0].result.clone()
    }

    pub fn compile(&mut self, ast: crate::ast::Block) -> Result<Slot> {
        let lhs = self.reg();
        let block_id = self.expr_block(ast, lhs.clone())?;
        self.op(OpCode::Call(block_id));
        Ok(lhs)
    }

    fn expr(&mut self, expr_: ast::Expr) -> Result<Slot> {
        match expr_ {
            ast::Expr::Lit(lit) => self.expr_literal(lit),
            ast::Expr::Unary(unop) => self.expr_unop(unop),
            ast::Expr::Binary(binop) => self.expr_binop(binop),
            ast::Expr::Block(block) => {
                let lhs = self.reg();
                let block = self.expr_block(block, lhs.clone())?;
                self.op(OpCode::Call(block));
                Ok(lhs)
            }
            ast::Expr::If(if_expr) => self.expr_if(if_expr),
            ast::Expr::Assign(assign) => self.expr_assign(assign),
            ast::Expr::Paren(paren) => self.expr(*paren),
            ast::Expr::Path(path) => {
                if path.path.len() != 1 {
                    bail!("Invalid path for assignment in RHDL code");
                }
                Ok(self.get_reference(path.path.last().unwrap())?)
            }
            ast::Expr::Tuple(tuple) => self.tuple(&tuple),
            _ => todo!("expr {:?}", expr_),
        }
    }

    fn tuple(&mut self, tuple: &[ast::Expr]) -> Result<Slot> {
        let lhs = self.reg();
        let mut fields = vec![];
        for expr in tuple {
            fields.push(self.expr(expr.clone())?);
        }
        self.op(OpCode::Tuple(TupleOp {
            lhs: lhs.clone(),
            fields,
        }));
        Ok(lhs)
    }

    pub fn expr_lhs(&mut self, expr_: ast::Expr) -> Result<Slot> {
        Ok(match expr_ {
            ast::Expr::Path(path) => {
                if path.path.len() != 1 {
                    bail!("Invalid path for assignment in RHDL code");
                }
                let arg = self.get_reference(path.path.last().unwrap())?;
                let lhs = self.reg();
                self.op(OpCode::Ref(RefOp {
                    lhs: lhs.clone(),
                    arg,
                }));
                lhs
            }
            ast::Expr::Field(field) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(*field.expr)?;
                let member = match field.member {
                    ast::Member::Named(name) => Member::Named(name),
                    ast::Member::Unnamed(index) => Member::Unnamed(index),
                };
                self.op(OpCode::FieldRef(FieldRefOp {
                    lhs: lhs.clone(),
                    arg,
                    member,
                }));
                lhs
            }
            ast::Expr::Index(index) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(*index.expr)?;
                let index = self.expr(*index.index)?;
                self.op(OpCode::IndexRef(IndexRefOp {
                    lhs: lhs.clone(),
                    arg,
                    index,
                }));
                lhs
            }
            _ => todo!("expr_lhs {:?}", expr_),
        })
    }

    pub fn stmt(&mut self, statement: ast::Stmt) -> Result<Slot> {
        match statement {
            ast::Stmt::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            ast::Stmt::Expr(expr_) => self.expr(expr_.expr),
            ast::Stmt::Semi(expr_) => {
                self.expr(expr_.expr)?;
                Ok(Slot::Empty)
            }
        }
    }

    fn local(&mut self, local: ast::Local) -> Result<()> {
        let rhs = local.value.map(|x| self.expr(*x)).transpose()?;
        self.let_pattern(local.pattern, rhs)?;
        Ok(())
    }

    // Some observations.
    // A type designation must appear outermost.  So if we have
    // something like:
    //  let (a, b, c) : ty = foo
    // this is legal, but
    //  let (a: ty, b: ty, c: ty) = foo
    // is not legal.
    //
    // In some ways, (a, b, c) is sort of like a shadow type declaration.
    // We could just as well devise an anonymous Tuple Struct named "Foo",
    // and then write:
    //   let Foo(a, b, c) = foo

    fn let_pattern(&mut self, pattern: ast::Pattern, rhs: Option<Slot>) -> Result<()> {
        if let ast::Pattern::Type(ty) = pattern {
            self.let_pattern_inner(*ty.pattern, Some(ty.kind), rhs)
        } else {
            self.let_pattern_inner(pattern, None, rhs)
        }
    }

    fn let_pattern_inner(
        &mut self,
        pattern: ast::Pattern,
        ty: Option<Kind>,
        rhs: Option<Slot>,
    ) -> Result<()> {
        match pattern {
            ast::Pattern::Ident(ident) => {
                let lhs = self.bind(&ident.name);
                if let Some(ty) = ty {
                    self.types.insert(lhs.reg()?, ty);
                }
                if let Some(rhs) = rhs {
                    self.op(OpCode::Copy(CopyOp {
                        lhs: lhs.clone(),
                        rhs,
                    }));
                }
                Ok(())
            }
            ast::Pattern::Tuple(tuple) => {
                for (ndx, pat) in tuple.into_iter().enumerate() {
                    let element_lhs = self.reg();
                    if let Some(rhs) = rhs.clone() {
                        self.op(OpCode::Field(FieldOp {
                            lhs: element_lhs.clone(),
                            arg: rhs.clone(),
                            member: Member::Unnamed(ndx as u32),
                        }));
                    }
                    let element_ty = if let Some(ty) = ty.as_ref() {
                        let sub_ty = ty.get_tuple_kind(ndx)?;
                        self.types.insert(element_lhs.reg()?, sub_ty.clone());
                        Some(sub_ty)
                    } else {
                        None
                    };
                    if rhs.is_some() {
                        self.let_pattern_inner(pat, element_ty, Some(element_lhs))?;
                    } else {
                        self.let_pattern_inner(pat, element_ty, None)?;
                    }
                }
                Ok(())
            }
            _ => todo!("Unsupported let pattern {:?}", pattern),
        }
    }

    pub fn expr_if(&mut self, if_expr: crate::ast::ExprIf) -> Result<Slot> {
        let lhs = self.reg();
        let cond = self.expr(*if_expr.cond)?;
        let then_branch = self.expr_block(if_expr.then_branch, lhs.clone())?;
        // Create a block containing the else part of the if expression
        let else_block = if_expr
            .else_branch
            .map(|x| {
                ast::Block(vec![ast::Stmt::Expr(ast::ExprStatement {
                    expr: *x,
                    text: None,
                })])
            })
            .unwrap_or(ast::Block(vec![]));
        let else_branch = self.expr_block(else_block, lhs.clone())?;
        self.op(OpCode::If(IfOp {
            lhs: lhs.clone(),
            cond,
            then_branch,
            else_branch,
        }));
        Ok(lhs)
    }

    // Start simple.
    // If an expression is <ExprLit> then stuff it into a Slot
    pub fn expr_literal(&mut self, lit: crate::ast::ExprLit) -> Result<Slot> {
        Ok(Slot::Literal(lit))
    }

    pub fn expr_assign(&mut self, assign: ast::ExprAssign) -> Result<Slot> {
        let lhs = self.expr_lhs(*assign.lhs)?;
        let rhs = self.expr(*assign.rhs)?;
        self.op(OpCode::Assign(AssignOp { lhs, rhs }));
        Ok(Slot::Empty)
    }

    pub fn expr_unop(&mut self, unop: crate::ast::ExprUnary) -> Result<Slot> {
        let arg = self.expr(*unop.expr)?;
        let dest = self.reg();
        let alu = match unop.op {
            UnOp::Neg => AluUnary::Neg,
            UnOp::Not => AluUnary::Not,
        };
        self.op(OpCode::Unary(UnaryOp {
            op: alu,
            lhs: dest.clone(),
            arg1: arg,
        }));
        Ok(dest)
    }

    pub fn expr_binop(&mut self, binop: crate::ast::ExprBinary) -> Result<Slot> {
        // Allocate a register for the output
        let lhs = self.expr(*binop.lhs)?;
        let rhs = self.expr(*binop.rhs)?;
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
        let dest = if self_assign { lhs.clone() } else { self.reg() };
        self.op(OpCode::Binary(BinaryOp {
            op: alu,
            lhs: dest.clone(),
            arg1: lhs,
            arg2: rhs,
        }));
        Ok(dest)
    }

    pub fn expr_block(&mut self, block: crate::ast::Block, lhs: Slot) -> Result<BlockId> {
        let statement_count = block.0.len();
        let current_block = self.current_block();
        let id = self.new_block(lhs.clone());
        // process each statement in the block and return the last one
        for (ndx, stmt_) in block.0.iter().enumerate() {
            if ndx == statement_count - 1 {
                let rhs = self.stmt(stmt_.clone())?;
                self.op(OpCode::Copy(CopyOp {
                    lhs: lhs.clone(),
                    rhs,
                }));
            } else {
                self.stmt(stmt_.clone())?;
            }
        }
        self.set_block(current_block);
        Ok(id)
    }
}
