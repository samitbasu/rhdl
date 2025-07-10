use std::fmt::Write;
use std::hash::Hash;
use std::hash::Hasher;

use fnv::FnvHasher;

use crate::rhdl_core::TypedBits;
use crate::rhdl_core::ast::ast_impl::{FunctionId, NodeId};
use crate::rhdl_core::ast::source::source_location::SourceLocation;
use crate::rhdl_core::common::symtab::RegisterId;
use crate::rhdl_core::common::symtab::SymbolTable;
use crate::rhdl_core::rhif::object::SourceDetails;
use crate::rhdl_core::types::bit_string::BitString;
use crate::rhdl_core::{Digital, Kind};

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

#[derive(Clone, Copy, Hash)]
pub enum RegisterSize {
    Signed(usize),
    Unsigned(usize),
}

impl RegisterSize {
    pub fn is_signed(&self) -> bool {
        matches!(self, RegisterSize::Signed(_))
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(self, RegisterSize::Unsigned(_))
    }
    pub fn len(&self) -> usize {
        match self {
            RegisterSize::Signed(len) => *len,
            RegisterSize::Unsigned(len) => *len,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn from_digital(t: impl Digital) -> Self {
        match t.kind() {
            Kind::Signed(len) => RegisterSize::Signed(len),
            _ => RegisterSize::Unsigned(t.bin().len()),
        }
    }
}

impl From<&Kind> for RegisterSize {
    fn from(value: &Kind) -> Self {
        match value {
            Kind::Signed(len) => RegisterSize::Signed(*len),
            _ => RegisterSize::Unsigned(value.bits()),
        }
    }
}

impl From<Kind> for RegisterSize {
    fn from(value: Kind) -> Self {
        (&value).into()
    }
}

impl From<&BitString> for RegisterSize {
    fn from(bs: &BitString) -> Self {
        match bs {
            BitString::Signed(bits) => RegisterSize::Signed(bits.len()),
            BitString::Unsigned(bits) => RegisterSize::Unsigned(bits.len()),
        }
    }
}

impl std::fmt::Debug for RegisterSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterSize::Signed(width) => write!(f, "s{width}"),
            RegisterSize::Unsigned(width) => write!(f, "b{width}"),
        }
    }
}

#[derive(Clone, Hash)]
pub struct Object {
    pub symbols: SymbolMap,
    pub symtab: SymbolTable<TypedBits, Kind, SourceDetails>,
    pub return_register: Operand,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<Option<RegisterId>>,
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
    pub fn size(&self, op: Operand) -> RegisterSize {
        match op {
            Operand::Register(reg) => self.symtab[reg].into(),
            Operand::Literal(lit) => self.symtab[lit].kind.into(),
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
        writeln!(f, "  arguments {:?}", self.arguments)?;
        writeln!(f, "  return_register {:?}", self.return_register)?;
        for (rid, (kind, details)) in self.symtab.iter_reg() {
            let name = match details.name.as_ref() {
                Some(s) => s.as_str(),
                None => "",
            };
            let size: RegisterSize = kind.into();
            writeln!(f, "Reg {rid:?} : {size:?} // {name} {kind:?}")?;
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
