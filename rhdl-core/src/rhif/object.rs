use fnv::FnvHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Range;

use crate::ast::source_location::SourceLocation;
use crate::ast::spanned_source::SpannedSourceSet;
use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::spec::Slot,
    Kind, TypedBits,
};

use super::spec::OpCode;
use super::spec::{FuncId, LiteralId, RegisterId};

#[derive(Debug, Clone, Hash)]
pub struct SymbolMap {
    pub source_set: SpannedSourceSet,
    pub slot_map: BTreeMap<Slot, SourceLocation>,
    pub slot_names: BTreeMap<Slot, String>,
    pub aliases: BTreeMap<Slot, BTreeSet<Slot>>,
}

impl SymbolMap {
    pub fn source(&self) -> String {
        self.source_set.source()
    }
    pub fn slot_span(&self, slot: Slot) -> Option<Range<usize>> {
        self.slot_map
            .get(&slot)
            .map(|loc| self.source_set.span(*loc))
    }
    pub fn span(&self, loc: SourceLocation) -> Range<usize> {
        self.source_set.span(loc)
    }
    pub fn best_span_for_slot_in_expression(
        &self,
        slot: Slot,
        expression: SourceLocation,
    ) -> Range<usize> {
        let expression_span = self.span(expression);
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
    pub fn fallback(&self, func: FunctionId) -> SourceLocation {
        self.source_set.fallback(func)
    }
}

#[derive(Clone, Hash)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub loc: SourceLocation,
}

impl LocatedOpCode {
    pub fn new(op: OpCode, id: NodeId, func: FunctionId) -> Self {
        Self {
            op,
            loc: SourceLocation { node: id, func },
        }
    }
}

impl From<(OpCode, NodeId, FunctionId)> for LocatedOpCode {
    fn from((op, id, func): (OpCode, NodeId, FunctionId)) -> Self {
        Self::new(op, id, func)
    }
}

impl From<(OpCode, SourceLocation)> for LocatedOpCode {
    fn from((op, loc): (OpCode, SourceLocation)) -> Self {
        Self { op, loc }
    }
}

#[derive(Clone, Hash)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<LiteralId, TypedBits>,
    pub kind: BTreeMap<RegisterId, Kind>,
    pub return_slot: Slot,
    pub externals: BTreeMap<FuncId, Box<Object>>,
    pub ops: Vec<LocatedOpCode>,
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
            Slot::Register(reg) => self.kind[&reg],
            Slot::Literal(lit) => self.literals[&lit].kind,
            Slot::Empty => Kind::Empty,
        }
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
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
            writeln!(f, "Function {:?} object: {:?}", ndx, func)?;
        }
        let mut body_str = String::new();
        for lop in &self.ops {
            if !matches!(lop.op, OpCode::Noop) {
                writeln!(body_str, "{:?}", lop.op)?;
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
