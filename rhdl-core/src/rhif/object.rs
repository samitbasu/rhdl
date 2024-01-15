use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::Write;

use crate::{
    ast::ast_impl::FunctionId,
    compiler::ty::Ty,
    rhif::spec::{Block, BlockId, ExternalFunction, Slot},
    TypedBits,
};

use super::spec::{Case, OpCode};

#[derive(Debug, Clone)]
pub struct Object {
    pub literals: Vec<TypedBits>,
    pub ty: BTreeMap<Slot, Ty>,
    pub blocks: Vec<Block>,
    pub return_slot: Slot,
    pub externals: Vec<ExternalFunction>,
    pub main_block: BlockId,
    pub arguments: Vec<Slot>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn literal(&self, slot: Slot) -> Result<&TypedBits> {
        match slot {
            Slot::Literal(l) => Ok(&self.literals[l]),
            _ => Err(anyhow::anyhow!("Not a literal")),
        }
    }
    pub fn reg_count(&self) -> usize {
        self.ty
            .keys()
            .filter_map(|slot| match slot {
                Slot::Register(ndx) => Some(ndx),
                _ => None,
            })
            .max()
            .copied()
            .unwrap_or(0)
    }
    pub fn display_block(&self, s: &mut String, block: BlockId) {
        for op in &self.blocks[block.0].ops {
            self.display_op(s, op);
        }
    }
    pub fn display_op(&self, s: &mut String, op: &OpCode) {
        match op {
            OpCode::Case(Case {
                discriminant: expr,
                table,
            }) => {
                writeln!(s, " case {} {{", expr).unwrap();
                for (cond, val) in table {
                    writeln!(s, "{} => {{", cond).unwrap();
                    self.display_block(s, *val);
                    writeln!(s, "}}").unwrap();
                }
                writeln!(s, "}}").unwrap();
            }
            OpCode::Block(block) => {
                writeln!(s, " block {{").unwrap();
                self.display_block(s, *block);
                writeln!(s, "}}").unwrap();
            }
            _ => writeln!(s, "{}", op).unwrap(),
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {}", self.fn_id)?;
        for regs in self.ty.keys() {
            if let Slot::Register(ndx) = regs {
                writeln!(f, "Reg r{} : {}", ndx, self.ty[regs])?;
            }
        }
        for (ndx, literal) in self.literals.iter().enumerate() {
            writeln!(
                f,
                "Literal l{} : {} = {}",
                ndx,
                self.ty[&Slot::Literal(ndx)],
                literal
            )?;
        }
        for (ndx, func) in self.externals.iter().enumerate() {
            writeln!(
                f,
                "Function f{} name: {} code: {} signature: {}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        let mut body_str = String::new();
        self.display_block(&mut body_str, self.main_block);
        let mut indent = 0;
        for line in body_str.lines() {
            let line = line.trim();
            if line.contains('}') {
                indent -= 1;
            }
            for _ in 0..indent {
                write!(f, "   ")?;
            }
            if line.contains('{') {
                indent += 1;
            }
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}
