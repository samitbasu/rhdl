use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::Write;

use crate::{
    ast::ast_impl::FunctionId,
    compiler::ty::Ty,
    rhif::spec::{ExternalFunction, Slot},
    TypedBits,
};

use super::spec::OpCode;

#[derive(Debug, Clone)]
pub struct Object {
    pub literals: Vec<TypedBits>,
    pub ty: BTreeMap<Slot, Ty>,
    pub return_slot: Slot,
    pub externals: Vec<ExternalFunction>,
    pub ops: Vec<OpCode>,
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
        for op in &self.ops {
            writeln!(body_str, "{}", op)?;
        }
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
