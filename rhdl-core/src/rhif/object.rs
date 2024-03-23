use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::Write;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    diagnostic::SpannedSource,
    rhif::spec::{ExternalFunction, Slot},
    Kind, TypedBits,
};

use super::spec::OpCode;

#[derive(Debug, Clone, Copy)]
pub struct SourceLocation {
    pub func: FunctionId,
    pub node: NodeId,
}

impl From<(FunctionId, NodeId)> for SourceLocation {
    fn from((func, node): (FunctionId, NodeId)) -> Self {
        Self { func, node }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolMap {
    pub source: SpannedSource,
    pub slot_map: BTreeMap<Slot, SourceLocation>,
    pub opcode_map: Vec<SourceLocation>,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<Slot, TypedBits>,
    pub kind: BTreeMap<Slot, Kind>,
    pub return_slot: Slot,
    pub externals: Vec<ExternalFunction>,
    pub ops: Vec<OpCode>,
    pub arguments: Vec<Slot>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn literal(&self, slot: Slot) -> Result<&TypedBits> {
        self.literals
            .get(&slot)
            .ok_or_else(|| anyhow::anyhow!("Not a literal"))
    }
    pub fn reg_max_index(&self) -> usize {
        self.kind
            .keys()
            .filter_map(|slot| match slot {
                Slot::Register(ndx) => Some(ndx),
                _ => None,
            })
            .max()
            .copied()
            .unwrap_or(0)
    }
    pub fn literal_max_index(&self) -> usize {
        self.kind
            .keys()
            .filter_map(|slot| match slot {
                Slot::Literal(ndx) => Some(ndx),
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
        for regs in self.kind.keys() {
            if let Slot::Register(ndx) = regs {
                writeln!(f, "Reg r{} : {}", ndx, self.kind[regs])?;
            }
        }
        for (slot, literal) in self.literals.iter() {
            writeln!(f, "Literal {} : {} = {}", slot, self.kind[slot], literal)?;
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
