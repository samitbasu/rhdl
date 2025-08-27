use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;

use fnv::FnvHasher;

use crate::Kind;
use crate::TypedBits;
use crate::ast::ast_impl::{FunctionId, NodeId};
use crate::ast::source::source_location::SourceLocation;
use crate::common::symtab::RegisterId;
use crate::common::symtab::SymbolTable;
use crate::rhif::object::SourceDetails;
use crate::rtl::spec::OperandKind;
use crate::types::bit_string::BitString;

use super::spec::{OpCode, Operand};
use super::symbols::SymbolMap;

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

impl From<(OpCode, SourceLocation)> for LocatedOpCode {
    fn from((op, loc): (OpCode, SourceLocation)) -> Self {
        Self { op, loc }
    }
}

pub fn lop(op: OpCode, id: NodeId, func: FunctionId) -> LocatedOpCode {
    LocatedOpCode::new(op, id, func)
}

#[derive(Clone, Hash)]
pub struct Object {
    pub symbols: SymbolMap,
    pub symtab: SymbolTable<TypedBits, Kind, SourceDetails, OperandKind>,
    pub return_register: Operand,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<Option<RegisterId<OperandKind>>>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn op_name(&self, op: Operand) -> String {
        format!("{op:?}")
    }
    pub fn op_alias(&self, op: Operand) -> Option<String> {
        self.symtab[op].name.clone()
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn kind(&self, op: Operand) -> Kind {
        match op {
            Operand::Literal(lid) => self.symtab[lid].kind,
            Operand::Register(rid) => self.symtab[rid],
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {:?}", self.fn_id)?;
        writeln!(f, "  arguments {:?}", self.arguments)?;
        writeln!(f, "  return_register {:?}", self.return_register)?;
        for (rid, (kind, details)) in self.symtab.iter_reg() {
            let name = match details.name.as_ref() {
                Some(s) => s.as_str(),
                None => "",
            };
            let reg_type = if kind.is_signed() { "s" } else { "b" };
            let reg_bits = kind.bits();
            writeln!(f, "Reg {rid:?} : {reg_type}{reg_bits} // {name} {kind:?}")?;
        }
        for (lid, (literal, _)) in self.symtab.iter_lit() {
            let bs: BitString = literal.into();
            let kind = literal.kind;
            writeln!(f, "Lit {lid:?} : {bs:?} // {kind:?}")?;
        }
        let mut body_str = String::new();
        for lop in &self.ops {
            writeln!(body_str, "{:?}", lop.op)?;
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
