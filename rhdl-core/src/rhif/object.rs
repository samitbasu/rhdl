use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::ops::Range;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::spec::{ExternalFunction, Slot},
    Kind, TypedBits,
};

use super::spec::{FuncId, LiteralId, RegisterId};
use super::{spanned_source::SpannedSource, spec::OpCode};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
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
    pub slot_map: BTreeMap<Slot, NodeId>,
    pub opcode_map: Vec<SourceLocation>,
    pub slot_names: BTreeMap<Slot, String>,
    pub aliases: BTreeMap<Slot, BTreeSet<Slot>>,
}

impl SymbolMap {
    pub fn slot_span(&self, slot: Slot) -> Option<Range<usize>> {
        self.slot_map.get(&slot).map(|loc| self.source.span(*loc))
    }
    pub fn node_span(&self, node: NodeId) -> Range<usize> {
        self.source.span(node)
    }
    pub fn best_span_for_slot_in_expression(&self, slot: Slot, expression: NodeId) -> Range<usize> {
        let expression_span = self.node_span(expression);
        let mut best_range = self.slot_span(slot).unwrap_or(expression_span);
        let mut best_range_len = best_range.len();
        if let Some(equivalent) = self.aliases.get(&slot) {
            for alias in equivalent {
                let alias_range = self.best_span_for_slot_in_expression(*alias, expression);
                let alias_range_len = alias_range.len();
                if alias_range_len < best_range_len
                    || (alias_range_len == best_range_len && alias_range.start > best_range.start)
                {
                    best_range = alias_range;
                    best_range_len = alias_range_len;
                }
            }
        }
        best_range
    }
    pub fn alias(&mut self, from_slot: Slot, to_slot: Slot) {
        self.aliases.entry(from_slot).or_default().insert(to_slot);
    }
}

#[derive(Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<LiteralId, TypedBits>,
    pub kind: BTreeMap<RegisterId, Kind>,
    pub return_slot: Slot,
    pub externals: BTreeMap<FuncId, ExternalFunction>,
    pub ops: Vec<OpCode>,
    pub arguments: Vec<RegisterId>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn reg_max_index(&self) -> RegisterId {
        self.kind.keys().max().copied().unwrap_or(RegisterId(0))
    }
    pub fn literal_max_index(&self) -> LiteralId {
        self.literals.keys().max().copied().unwrap_or(LiteralId(0))
    }
    pub fn kind(&self, slot: Slot) -> Kind {
        match slot {
            Slot::Register(reg) => self.kind[&reg].clone(),
            Slot::Literal(lit) => self.literals[&lit].kind.clone(),
            Slot::Empty => Kind::Empty,
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {:?}", self.fn_id)?;
        writeln!(f, "  return_slot {:?}", self.return_slot)?;
        for regs in self.kind.keys() {
            let slot_name = self
                .symbols
                .slot_names
                .get(&Slot::Register(*regs))
                .map(|s| s.as_str())
                .unwrap_or("");
            writeln!(f, "Reg {:?} : {:?} // {}", regs, self.kind[regs], slot_name)?;
        }
        for (slot, literal) in self.literals.iter() {
            writeln!(f, "Literal {:?} : {:?} = {:?}", slot, literal.kind, literal)?;
        }
        for (ndx, func) in self.externals.iter() {
            writeln!(
                f,
                "Function {:?} name: {} code: {:?} signature: {:?}",
                ndx, func.path, func.code, func.signature
            )?;
        }
        let mut body_str = String::new();
        for op in &self.ops {
            if !matches!(op, OpCode::Noop) {
                writeln!(body_str, "{:?}", op)?;
            }
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
