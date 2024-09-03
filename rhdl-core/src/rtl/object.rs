use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::iter::repeat;

use crate::error::rhdl_error;
use crate::rhif::object::SourceLocation;
use crate::types::bit_string::BitString;
use crate::types::error::DynamicTypeError;
use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    util::binary_string,
    TypedBits,
};
use crate::{Digital, Kind, RHDLError};

use super::spec::{LiteralId, OpCode, Operand, RegisterId};
use super::symbols::SymbolMap;

#[derive(Clone)]
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

pub fn lop(op: OpCode, id: NodeId, func: FunctionId) -> LocatedOpCode {
    LocatedOpCode::new(op, id, func)
}

#[derive(Clone, Copy)]
pub enum RegisterKind {
    Signed(usize),
    Unsigned(usize),
}

impl RegisterKind {
    pub fn is_signed(&self) -> bool {
        matches!(self, RegisterKind::Signed(_))
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(self, RegisterKind::Unsigned(_))
    }
    pub fn len(&self) -> usize {
        match self {
            RegisterKind::Signed(len) => *len,
            RegisterKind::Unsigned(len) => *len,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn from_digital(t: impl Digital) -> Self {
        match t.kind() {
            Kind::Signed(len) => RegisterKind::Signed(len),
            _ => RegisterKind::Unsigned(t.bin().len()),
        }
    }
}

impl From<&Kind> for RegisterKind {
    fn from(value: &Kind) -> Self {
        match value {
            Kind::Signed(len) => RegisterKind::Signed(*len),
            _ => RegisterKind::Unsigned(value.bits()),
        }
    }
}

impl From<Kind> for RegisterKind {
    fn from(value: Kind) -> Self {
        (&value).into()
    }
}

impl From<&BitString> for RegisterKind {
    fn from(bs: &BitString) -> Self {
        match bs {
            BitString::Signed(bits) => RegisterKind::Signed(bits.len()),
            BitString::Unsigned(bits) => RegisterKind::Unsigned(bits.len()),
        }
    }
}

impl std::fmt::Debug for RegisterKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterKind::Signed(width) => write!(f, "s{}", width),
            RegisterKind::Unsigned(width) => write!(f, "b{}", width),
        }
    }
}

#[derive(Clone)]
pub struct Object {
    pub symbols: SymbolMap,
    pub literals: BTreeMap<LiteralId, BitString>,
    pub register_kind: BTreeMap<RegisterId, RegisterKind>,
    pub return_register: Operand,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<Option<RegisterId>>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl Object {
    pub fn reg_max_index(&self) -> RegisterId {
        self.register_kind
            .keys()
            .max()
            .copied()
            .unwrap_or(RegisterId(0))
    }
    pub fn literal_max_index(&self) -> LiteralId {
        self.literals.keys().max().copied().unwrap_or(LiteralId(0))
    }
    pub fn op_name(&self, op: Operand) -> String {
        format!("{op:?}")
    }
    pub fn op_alias(&self, op: Operand) -> Option<String> {
        self.symbols.operand_names.get(&op).cloned()
    }
    pub fn kind(&self, op: Operand) -> RegisterKind {
        match op {
            Operand::Register(reg) => self.register_kind[&reg],
            Operand::Literal(lit) => (&self.literals[&lit]).into(),
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {:?}", self.fn_id)?;
        writeln!(f, "  arguments {:?}", self.arguments)?;
        writeln!(f, "  return_register {:?}", self.return_register)?;
        for (reg, kind) in &self.register_kind {
            writeln!(f, "Reg {reg:?} : {kind:?}")?;
        }
        for (id, literal) in &self.literals {
            writeln!(f, "Lit {id:?} : {literal:?}")?;
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
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}
