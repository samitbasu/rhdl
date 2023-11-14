use std::collections::BTreeMap;
use std::{collections::HashMap, fmt::Display};

use crate::ast::{
    self, BinOp, Expr, ExprArray, ExprAssign, ExprBinary, ExprBlock, ExprCall, ExprField,
    ExprGroup, ExprIf, ExprIndex, ExprMatch, ExprParen, ExprPath, ExprRepeat, ExprStruct,
    ExprTuple, ExprUnary, PatIdent, PatLit, PatPath, PatStruct, PatTuple, PatTupleStruct, PatType,
    Path, UnOp,
};
use crate::rhif::{AluBinary, AluUnary, BlockId, CaseArgument, Member, OpCode, Slot};
use crate::rhif_type::Ty;
use crate::Kind;
use anyhow::bail;
use anyhow::Result;

const ROOT_BLOCK: BlockId = BlockId(0);

impl From<ast::Member> for crate::rhif::Member {
    fn from(member: ast::Member) -> Self {
        match member {
            ast::Member::Named(name) => crate::rhif::Member::Named(name),
            ast::Member::Unnamed(index) => crate::rhif::Member::Unnamed(index),
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

pub struct Compiler {
    pub blocks: Vec<Block>,
    pub literals: Vec<Literal>,
    pub reg_count: usize,
    active_block: BlockId,
    types: BTreeMap<usize, Ty>,
}

pub struct Literal {
    pub value: Box<ast::ExprLit>,
    pub ty: Option<Ty>,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ndx, kind) in &self.types {
            writeln!(f, "Type r{} = {:?}", ndx, kind)?;
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
        // List registers that are not typed
        for ndx in 0..self.reg_count {
            if !self.types.contains_key(&ndx) {
                writeln!(f, "Register r{} is not typed", ndx)?;
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
            literals: vec![],
            reg_count: 0,
            active_block: ROOT_BLOCK,
            types: Default::default(),
        }
    }
}

fn collapse_path(path: &Path) -> String {
    path.segments
        .iter()
        .map(|x| x.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

impl Compiler {
    pub fn ty(&self, slot: Slot) -> Option<&Ty> {
        match slot {
            Slot::Literal(ndx) => self.literals[ndx].ty.as_ref(),
            Slot::Register(ndx) => self.types.get(&ndx),
            Slot::Empty => Some(&Ty::Empty),
        }
    }
    pub fn set_ty(&mut self, arg: Slot, target_kind: Ty) {
        match arg {
            Slot::Literal(ndx) => {
                self.literals[ndx].ty = Some(target_kind);
            }
            Slot::Register(ndx) => {
                self.types.insert(ndx, target_kind);
            }
            Slot::Empty => {}
        }
    }
    pub fn types_known(&self) -> usize {
        self.types.len() + self.literals.iter().filter(|x| x.ty.is_some()).count()
    }
    pub fn reg(&mut self) -> Slot {
        let reg = self.reg_count;
        self.reg_count += 1;
        Slot::Register(reg)
    }
    pub fn bind(&mut self, name: &str) -> Slot {
        let reg = self.reg();
        self.blocks[self.active_block.0]
            .names
            .insert(name.to_string(), reg);
        reg
    }
    pub fn literal(&mut self, value: Box<ast::ExprLit>) -> Slot {
        let ndx = self.literals.len();
        self.literals.push(Literal { value, ty: None });
        Slot::Literal(ndx)
    }
    pub fn type_bind(&mut self, name: &str, kind: Kind) -> Slot {
        let reg = self.bind(name);
        self.types.insert(reg.reg().unwrap(), Ty::Kind(kind));
        reg
    }
    pub fn get_reference(&mut self, path: &str) -> Result<Slot> {
        let mut ip = self.active_block;
        loop {
            if let Some(slot) = self.blocks[ip.0].names.get(path) {
                return Ok(*slot);
            }
            if ip == ROOT_BLOCK {
                break;
            }
            ip = self.blocks[ip.0].parent;
        }
        bail!("Unknown path {}", path);
    }
    pub fn iter_ops(&self) -> impl Iterator<Item = &OpCode> {
        self.blocks.iter().flat_map(|x| x.ops.iter())
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
    pub fn compile(&mut self, ast: Box<crate::ast::Block>) -> Result<Slot> {
        let lhs = self.reg();
        let block_id = self.expr_block(ast, lhs)?;
        self.op(OpCode::Block(block_id));
        Ok(lhs)
    }

    fn expr_list(&mut self, exprs: Vec<Box<ast::Expr>>) -> Result<Vec<Slot>> {
        exprs
            .into_iter()
            .map(|x| self.expr(x))
            .collect::<Result<_>>()
    }

    fn expr(&mut self, expr_: Box<ast::Expr>) -> Result<Slot> {
        match expr_.kind {
            ast::ExprKind::Binary(ExprBinary { op, lhs, rhs }) => self.expr_binop(op, lhs, rhs),
            ast::ExprKind::Unary(ExprUnary { op, expr }) => self.expr_unop(op, expr),
            ast::ExprKind::If(ExprIf {
                cond,
                then_branch,
                else_branch,
            }) => self.expr_if(cond, then_branch, else_branch),
            ast::ExprKind::Index(ExprIndex { expr, index }) => self.expr_index(expr, index),
            ast::ExprKind::Assign(ExprAssign { lhs, rhs }) => self.expr_assign(lhs, rhs),
            ast::ExprKind::Lit(lit) => self.expr_literal(Box::new(lit)),
            ast::ExprKind::Repeat(ExprRepeat { value, len }) => self.expr_repeat(value, len),
            ast::ExprKind::Paren(ExprParen { expr }) => self.expr(expr),
            ast::ExprKind::Group(ExprGroup { expr }) => self.expr(expr),
            ast::ExprKind::Array(ExprArray { elems }) => self.expr_array(elems),
            ast::ExprKind::Field(ExprField { expr, member }) => self.expr_field(expr, member),
            ast::ExprKind::Path(ExprPath { path }) => {
                Ok(self.get_reference(&collapse_path(&path))?)
            }
            ast::ExprKind::Tuple(ExprTuple { elements }) => self.tuple(elements),
            ast::ExprKind::Struct(ExprStruct {
                path,
                fields,
                rest,
                kind: _,
            }) => self.expr_struct(path, fields, rest),
            ast::ExprKind::Call(ExprCall { path, args, .. }) => self.expr_call(path, args),
            ast::ExprKind::Match(ExprMatch { expr, arms }) => self.expr_match(expr, arms),
            ast::ExprKind::Block(ExprBlock { block }) => {
                let lhs = self.reg();
                let block = self.expr_block(block, lhs)?;
                self.op(OpCode::Block(block));
                Ok(lhs)
            }
            _ => todo!("expr {:?}", expr_),
        }
    }

    fn expr_repeat(&mut self, value: Box<Expr>, len: Box<Expr>) -> Result<Slot> {
        let lhs = self.reg();
        let value = self.expr(value)?;
        let len = self.expr(len)?;
        self.op(OpCode::Repeat { lhs, value, len });
        Ok(lhs)
    }

    fn expr_array(&mut self, elems: Vec<Box<Expr>>) -> Result<Slot> {
        let lhs = self.reg();
        let elements = self.expr_list(elems)?;
        self.op(OpCode::Array { lhs, elements });
        Ok(lhs)
    }

    fn expr_field(&mut self, expr: Box<Expr>, member: ast::Member) -> Result<Slot> {
        let lhs = self.reg();
        let arg = self.expr(expr)?;
        self.op(OpCode::Field {
            lhs,
            arg,
            member: member.into(),
        });
        Ok(lhs)
    }

    fn field_value(&mut self, element: Box<ast::FieldValue>) -> Result<crate::rhif::FieldValue> {
        let value = self.expr(element.value)?;
        Ok(crate::rhif::FieldValue {
            member: element.member.into(),
            value,
        })
    }

    fn expr_index(&mut self, expr: Box<Expr>, index: Box<Expr>) -> Result<Slot> {
        let lhs = self.reg();
        let arg = self.expr(expr)?;
        let index = self.expr(index)?;
        self.op(OpCode::Index { lhs, arg, index });
        Ok(lhs)
    }

    fn expr_struct(
        &mut self,
        path: Box<Path>,
        fields: Vec<Box<ast::FieldValue>>,
        rest: Option<Box<Expr>>,
    ) -> Result<Slot> {
        let lhs = self.reg();
        let fields = fields
            .into_iter()
            .map(|x| self.field_value(x))
            .collect::<Result<Vec<_>>>()?;
        self.op(OpCode::Struct {
            lhs,
            path: collapse_path(&path),
            fields,
            rest: None,
        });
        Ok(lhs)
    }

    fn tuple(&mut self, tuple: Vec<Box<ast::Expr>>) -> Result<Slot> {
        let lhs = self.reg();
        let fields = self.expr_list(tuple)?;
        self.op(OpCode::Tuple { lhs, fields });
        Ok(lhs)
    }

    fn expr_call(&mut self, path: Box<Path>, args: Vec<Box<Expr>>) -> Result<Slot> {
        let lhs = self.reg();
        let path = collapse_path(&path);
        let args = self.expr_list(args)?;
        self.op(OpCode::Exec { lhs, path, args });
        Ok(lhs)
    }

    fn expr_match(&mut self, expr: Box<Expr>, arms: Vec<Box<ast::Arm>>) -> Result<Slot> {
        // Only two supported cases of match arms
        // The first is all literals and possibly a wildcard
        // The second is all enums with no literals and possibly a wildcard
        for arm in &arms {
            if let Some(guard) = &arm.guard {
                bail!(
                    "RHDL does not currently support match guards in hardware {:?}",
                    guard
                );
            }
        }
        let all_literals_or_wild = arms.iter().all(|arm| {
            matches!(
                arm.pattern.kind,
                ast::PatKind::Lit { .. } | ast::PatKind::Wild
            )
        });
        let all_enum_or_wild = arms.iter().all(|arm| {
            matches!(
                arm.pattern.kind,
                ast::PatKind::Path { .. }
                    | ast::PatKind::Struct { .. }
                    | ast::PatKind::TupleStruct { .. }
                    | ast::PatKind::Wild
            )
        });
        if !all_literals_or_wild && !all_enum_or_wild {
            bail!("RHDL currently supports only match arms with all literals or all enums (and a wildcard '_' is allowed)");
        }
        self.expr_case(expr, arms)
    }

    fn expr_case(&mut self, expr: Box<Expr>, arms: Vec<Box<ast::Arm>>) -> Result<Slot> {
        let lhs = self.reg();
        let target = self.expr(expr)?;
        let table = arms
            .into_iter()
            .map(|arm| self.expr_arm(target, lhs, arm))
            .collect::<Result<_>>()?;
        self.op(OpCode::Case {
            lhs,
            expr: target,
            table,
        });
        Ok(lhs)
    }

    fn expr_arm_struct(
        &mut self,
        target: Slot,
        lhs: Slot,
        path: Box<Path>,
        fields: Vec<Box<ast::FieldPat>>,
        rest: bool,
        body: Box<ast::Expr>,
    ) -> Result<(CaseArgument, BlockId)> {
        // Collect the elements of the struct that are identifiers (and not wildcards)
        // For each element of the pattern, collect the name (this is the binding) and the
        // position within the tuple.
        let bindings: Vec<(Member, String)> = fields
            .into_iter()
            .map(|x| match x.pat.kind {
                ast::PatKind::Ident(PatIdent { name, .. }) => Ok(Some((x.member.into(), name))),
                ast::PatKind::Wild => Ok(None),
                _ => bail!("Unsupported match pattern {:?} in hardware", x),
            })
            .filter_map(|x| x.transpose())
            .collect::<Result<Vec<_>>>()?;
        // Create a new block for the struct match
        let current_id = self.current_block();
        let id = self.new_block(lhs);
        // For each binding, create a new register and bind it to the name
        // Then insert an opcode into the block to extract the field from the struct
        // that is the target of the match.
        bindings.into_iter().for_each(|(member, ident)| {
            let reg = self.bind(&ident);
            self.op(OpCode::Field {
                lhs: reg,
                arg: target,
                member,
            });
        });
        // Add the arm body to the block
        let expr_output = self.expr(body)?;
        // Copy the result of the arm body to the lhs
        self.op(OpCode::Copy {
            lhs,
            rhs: expr_output,
        });
        self.set_block(current_id);
        Ok((CaseArgument::Path(collapse_path(&path)), id))
    }

    fn expr_arm_tuple_struct(
        &mut self,
        target: Slot,
        lhs: Slot,
        path: Box<Path>,
        elements: Vec<Box<ast::Pat>>,
        body: Box<ast::Expr>,
    ) -> Result<(CaseArgument, BlockId)> {
        // Collect the elements of the tuple struct that are identifiers (and not wildcards)
        // For each element of the pattern, collect the name (this is the binding) and the
        // position within the tuple.
        let bindings = elements
            .into_iter()
            .enumerate()
            .map(|(ndx, x)| match x.kind {
                ast::PatKind::Ident(PatIdent { name, .. }) => Ok(Some((name, ndx))),
                ast::PatKind::Wild => Ok(None),
                _ => bail!("Unsupported match pattern {:?} in hardware", x),
            })
            .filter_map(|x| x.transpose())
            .collect::<Result<Vec<_>>>()?;
        // Create a new block for the tuple struct match
        let current_id = self.current_block();
        let id = self.new_block(lhs);
        // For each binding, create a new register and bind it to the name
        // Then insert an opcode into the block to extract the field from the tuple
        // that is the target of the match.
        bindings.into_iter().for_each(|(ident, index)| {
            let reg = self.bind(&ident);
            self.op(OpCode::Field {
                lhs: reg,
                arg: target,
                member: Member::Unnamed(index as u32),
            });
        });
        // Add the arm body to the block
        let expr_output = self.expr(body)?;
        // Copy the result of the arm body to the lhs
        self.op(OpCode::Copy {
            lhs,
            rhs: expr_output,
        });
        self.set_block(current_id);
        Ok((CaseArgument::Path(collapse_path(&path)), id))
    }

    fn expr_arm(
        &mut self,
        target: Slot,
        lhs: Slot,
        arm: Box<ast::Arm>,
    ) -> Result<(CaseArgument, BlockId)> {
        match arm.pattern.kind {
            ast::PatKind::Wild => Ok((
                CaseArgument::Wild,
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::PatKind::Lit(PatLit { lit }) => Ok((
                CaseArgument::Literal(self.literal(lit)),
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::PatKind::Path(PatPath { path }) => Ok((
                CaseArgument::Path(collapse_path(&path)),
                self.wrap_expr_in_block(Some(arm.body), lhs)?,
            )),
            ast::PatKind::TupleStruct(PatTupleStruct { path, elems }) => {
                self.expr_arm_tuple_struct(target, lhs, path, elems, arm.body)
            }
            ast::PatKind::Struct(PatStruct { path, fields, rest }) => {
                self.expr_arm_struct(target, lhs, path, fields, rest, arm.body)
            }
            _ => bail!("Unsupported match pattern {:?} in hardware", arm.pattern),
        }
    }

    pub fn expr_lhs(&mut self, expr_: Box<ast::Expr>) -> Result<Slot> {
        Ok(match expr_.kind {
            ast::ExprKind::Path(ExprPath { path }) => {
                let arg = self.get_reference(&collapse_path(&path))?;
                let lhs = self.reg();
                self.op(OpCode::Ref { lhs, arg });
                lhs
            }
            ast::ExprKind::Field(ExprField { expr, member }) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(expr)?;
                let member = member.into();
                self.op(OpCode::FieldRef { lhs, arg, member });
                lhs
            }
            ast::ExprKind::Index(ExprIndex { expr, index }) => {
                let lhs = self.reg();
                let arg = self.expr_lhs(expr)?;
                let index = self.expr(index)?;
                self.op(OpCode::IndexRef { lhs, arg, index });
                lhs
            }
            _ => todo!("expr_lhs {:?}", expr_),
        })
    }

    pub fn stmt(&mut self, statement: ast::Stmt) -> Result<Slot> {
        match statement.kind {
            ast::StmtKind::Local(local) => {
                self.local(local)?;
                Ok(Slot::Empty)
            }
            ast::StmtKind::Expr(expr_) => self.expr(expr_),
            ast::StmtKind::Semi(expr_) => {
                self.expr(expr_)?;
                Ok(Slot::Empty)
            }
        }
    }

    fn local(&mut self, local: Box<ast::Local>) -> Result<()> {
        let rhs = local.init.map(|x| self.expr(x)).transpose()?;
        self.let_pattern(local.pat, rhs)?;
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

    fn let_pattern(&mut self, pattern: Box<ast::Pat>, rhs: Option<Slot>) -> Result<()> {
        if let ast::PatKind::Type(PatType { pat, kind }) = pattern.kind {
            self.let_pattern_inner(pat, Some(kind), rhs)
        } else {
            self.let_pattern_inner(pattern, None, rhs)
        }
    }

    fn let_pattern_inner(
        &mut self,
        pattern: Box<ast::Pat>,
        ty: Option<Kind>,
        rhs: Option<Slot>,
    ) -> Result<()> {
        match pattern.kind {
            ast::PatKind::Ident(PatIdent { name, mutable }) => {
                let lhs = self.bind(&name);
                if let Some(ty) = ty {
                    self.types.insert(lhs.reg()?, Ty::Kind(ty));
                }
                if let Some(rhs) = rhs {
                    self.op(OpCode::Copy { lhs, rhs });
                }
                Ok(())
            }
            ast::PatKind::Tuple(PatTuple { elements }) => {
                for (ndx, pat) in elements.into_iter().enumerate() {
                    let element_lhs = self.reg();
                    if let Some(rhs) = rhs {
                        self.op(OpCode::Field {
                            lhs: element_lhs,
                            arg: rhs,
                            member: Member::Unnamed(ndx as u32),
                        });
                    }
                    let element_ty = if let Some(ty) = ty.as_ref() {
                        let sub_ty = ty.get_tuple_kind(ndx)?;
                        self.types
                            .insert(element_lhs.reg()?, Ty::Kind(sub_ty.clone()));
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

    pub fn expr_if(
        &mut self,
        cond: Box<Expr>,
        then_branch: Box<ast::Block>,
        else_branch: Option<Box<Expr>>,
    ) -> Result<Slot> {
        let lhs = self.reg();
        let cond = self.expr(cond)?;
        let then_branch = self.expr_block(then_branch, lhs)?;
        // Create a block containing the else part of the if expression
        let else_branch = self.wrap_expr_in_block(else_branch, lhs)?;
        self.op(OpCode::If {
            lhs,
            cond,
            then_branch,
            else_branch,
        });
        Ok(lhs)
    }

    // Start simple.
    // If an expression is <ExprLit> then stuff it into a Slot
    pub fn expr_literal(&mut self, lit: Box<crate::ast::ExprLit>) -> Result<Slot> {
        Ok(self.literal(lit))
    }

    pub fn expr_assign(&mut self, lhs: Box<Expr>, rhs: Box<Expr>) -> Result<Slot> {
        let lhs = self.expr_lhs(lhs)?;
        let rhs = self.expr(rhs)?;
        self.op(OpCode::Assign { lhs, rhs });
        Ok(Slot::Empty)
    }

    pub fn expr_unop(&mut self, op: UnOp, expr: Box<Expr>) -> Result<Slot> {
        let arg = self.expr(expr)?;
        let dest = self.reg();
        let alu = match op {
            UnOp::Neg => AluUnary::Neg,
            UnOp::Not => AluUnary::Not,
        };
        self.op(OpCode::Unary {
            op: alu,
            lhs: dest,
            arg1: arg,
        });
        Ok(dest)
    }

    pub fn expr_binop(&mut self, op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>) -> Result<Slot> {
        // Allocate a register for the output
        let lhs = self.expr(lhs)?;
        let rhs = self.expr(rhs)?;
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
        let dest = if self_assign { lhs } else { self.reg() };
        self.op(OpCode::Binary {
            op: alu,
            lhs: dest,
            arg1: lhs,
            arg2: rhs,
        });
        Ok(dest)
    }

    // Add a set of statements to the current block with capturing of lhs for the last
    // statement in the list.
    fn expr_block_inner(&mut self, statements: &[Box<ast::Stmt>], lhs: Slot) -> Result<()> {
        let statement_count = statements.len();
        for (ndx, stmt) in statements.iter().enumerate() {
            if ndx == statement_count - 1 {
                let rhs = self.stmt(*stmt.clone())?;
                self.op(OpCode::Copy { lhs, rhs });
            } else {
                self.stmt(*stmt.clone())?;
            }
        }
        Ok(())
    }

    pub fn expr_block(&mut self, block: Box<crate::ast::Block>, lhs: Slot) -> Result<BlockId> {
        let current_block = self.current_block();
        let id = self.new_block(lhs);
        self.expr_block_inner(&block.stmts, lhs)?;
        self.set_block(current_block);
        Ok(id)
    }

    // There are places where Rust allows either an expression or a block.  For example in the
    // else branch of an if, or in each arm of a match.  Because these have different behaviors
    // in RHIF (a block is executed when jumped to, while an expression is immediate), we need
    // to be able to "wrap" an expression into a block so that it can be invoked conditionally.
    // As a special case, if the expression is empty (None), we create an empty block.
    pub fn wrap_expr_in_block(
        &mut self,
        expr: Option<Box<crate::ast::Expr>>,
        lhs: Slot,
    ) -> Result<BlockId> {
        let block = if let Some(expr) = expr {
            let stmt = ast::Stmt {
                id: None,
                kind: ast::StmtKind::Expr(expr),
            };
            ast::Block {
                id: None,
                stmts: vec![Box::new(stmt)],
            }
        } else {
            ast::Block {
                id: None,
                stmts: vec![],
            }
        };
        self.expr_block(Box::new(block), lhs)
    }
}
