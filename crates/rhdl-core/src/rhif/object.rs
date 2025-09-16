use fnv::FnvHasher;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Range;

use crate::ast::KernelFlags;
use crate::ast::SourceLocation;
use crate::ast::SourcePool;
use crate::ast::spanned_source::SpannedSourceSet;
use crate::common::symtab::{RegisterId, SymbolTable};
use crate::rhif::spec::SlotKind;
use crate::{
    Kind, TypedBits,
    ast::ast_impl::{FunctionId, NodeId},
    rhif::spec::Slot,
};

use super::spec::FuncId;
use super::spec::OpCode;

#[derive(Debug, Clone, Hash, Default)]
pub struct SymbolMap {
    pub source_set: SpannedSourceSet,
}

impl SymbolMap {
    pub fn source(&self) -> SourcePool {
        self.source_set.source()
    }
    pub fn span(&self, loc: SourceLocation) -> Range<usize> {
        self.source_set.span(loc)
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

#[derive(Clone, Hash, Debug)]
pub struct SourceDetails {
    pub location: SourceLocation,
    pub name: Option<String>,
}

impl From<SourceLocation> for SourceDetails {
    fn from(val: SourceLocation) -> Self {
        Self {
            location: val,
            name: None,
        }
    }
}

#[derive(Clone, Hash)]
pub struct Object {
    pub symbols: SymbolMap,
    pub symtab: SymbolTable<TypedBits, Kind, SourceDetails, SlotKind>,
    pub return_slot: Slot,
    pub externals: BTreeMap<FuncId, Box<Object>>,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<RegisterId<SlotKind>>,
    pub name: String,
    pub fn_id: FunctionId,
    pub flags: Vec<KernelFlags>,
}

impl Object {
    pub fn kind(&self, slot: Slot) -> Kind {
        match slot {
            Slot::Register(reg) => self.symtab[reg],
            Slot::Literal(lit) => self.symtab[lit].kind,
        }
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn filename(&self) -> &str {
        self.symbols.source_set.filename(self.fn_id)
    }
    pub fn slot_span(&self, slot: Slot) -> Range<usize> {
        let loc = &self.symtab[slot];
        self.symbols.span(loc.location)
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {:?}", self.fn_id)?;
        writeln!(f, "  return_slot {:?}", self.return_slot)?;
        for (reg, (kind, details)) in self.symtab.iter_reg() {
            let slot_name = match details.name.as_ref() {
                Some(x) => x.as_str(),
                None => "",
            };
            writeln!(f, "Reg {reg:?} : {kind:?} // {slot_name}")?;
        }
        for (lit, (tb, _)) in self.symtab.iter_lit() {
            let kind = tb.kind;
            writeln!(f, "Literal {lit:?} : {kind:?} = {tb:?}")?;
        }
        for (ndx, func) in self.externals.iter() {
            writeln!(f, "Function {ndx:?} object: {func:?}")?;
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
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
