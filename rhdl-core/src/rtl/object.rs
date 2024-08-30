use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::iter::repeat;

use crate::error::rhdl_error;
use crate::rhif::object::SourceLocation;
use crate::types::error::DynamicTypeError;
use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::{object::SymbolMap, spec::Slot},
    util::binary_string,
    TypedBits,
};
use crate::{Digital, Kind, RHDLError};

use super::spec::{LiteralId, OpCode, Operand, RegisterId};

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

pub fn lop(op: OpCode, id: NodeId, func: FunctionId) -> LocatedOpCode {
    LocatedOpCode::new(op, id, func)
}

#[derive(Clone, PartialEq, Eq)]
pub enum BitString {
    Signed(Vec<bool>),
    Unsigned(Vec<bool>),
}

impl BitString {
    pub fn is_signed(&self) -> bool {
        matches!(self, BitString::Signed(_))
    }
    pub fn is_unsigned(&self) -> bool {
        matches!(self, BitString::Unsigned(_))
    }
    pub fn len(&self) -> usize {
        match self {
            BitString::Signed(bits) => bits.len(),
            BitString::Unsigned(bits) => bits.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn bits(&self) -> &[bool] {
        match self {
            BitString::Signed(bits) => bits,
            BitString::Unsigned(bits) => bits,
        }
    }
    pub fn unsigned_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        if len > self.len() {
            return Ok(BitString::Unsigned(
                self.bits()
                    .iter()
                    .copied()
                    .chain(repeat(false))
                    .take(len)
                    .collect(),
            ));
        }
        let (base, rest) = self.bits().split_at(len);
        if rest.iter().any(|b| *b) {
            return Err(rhdl_error(DynamicTypeError::UnsignedCastWithWidthFailed {
                value: self.into(),
                bits: len,
            }));
        }
        Ok(BitString::Unsigned(base.to_vec()))
    }
    pub fn signed_cast(&self, len: usize) -> Result<BitString, RHDLError> {
        if len > self.len() {
            let sign_bit = self.bits().last().copied().unwrap_or(false);
            return Ok(BitString::Signed(
                self.bits()
                    .iter()
                    .copied()
                    .chain(repeat(sign_bit))
                    .take(len)
                    .collect(),
            ));
        }
        let (base, rest) = self.bits().split_at(len);
        let new_sign_bit = base.last().cloned().unwrap_or_default();
        if rest.iter().any(|b| *b != new_sign_bit) {
            return Err(rhdl_error(DynamicTypeError::SignedCastWithWidthFailed {
                value: self.into(),
                bits: len,
            }));
        }
        Ok(BitString::Signed(base.to_vec()))
    }
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitString::Signed(bits) => {
                write!(f, "s{}", binary_string(bits))?;
                Ok(())
            }
            BitString::Unsigned(bits) => {
                write!(f, "b{}", binary_string(bits))?;
                Ok(())
            }
        }
    }
}

impl From<&BitString> for TypedBits {
    fn from(bs: &BitString) -> Self {
        if bs.is_signed() {
            {
                TypedBits {
                    bits: bs.bits().to_owned(),
                    kind: Kind::make_signed(bs.len()),
                }
            }
        } else {
            {
                TypedBits {
                    bits: bs.bits().to_owned(),
                    kind: Kind::make_bits(bs.len()),
                }
            }
        }
    }
}

impl From<BitString> for TypedBits {
    fn from(bs: BitString) -> Self {
        (&bs).into()
    }
}

impl From<&TypedBits> for BitString {
    fn from(tb: &TypedBits) -> Self {
        if tb.kind.is_signed() {
            BitString::Signed(tb.bits.clone())
        } else {
            BitString::Unsigned(tb.bits.clone())
        }
    }
}

impl From<TypedBits> for BitString {
    fn from(tb: TypedBits) -> Self {
        (&tb).into()
    }
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
    pub symbols: HashMap<FunctionId, SymbolMap>,
    pub literals: BTreeMap<LiteralId, BitString>,
    pub operand_map: BTreeMap<Operand, (FunctionId, Slot)>,
    pub register_kind: BTreeMap<RegisterId, RegisterKind>,
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
        if let Some((func, slot)) = self.operand_map.get(&op) {
            if let Some(name) = self.symbols[func].slot_names.get(slot) {
                Some(format!("{slot:?}_{name}"))
            } else {
                Some(format!("{slot:?}"))
            }
        } else {
            None
        }
    }
    pub fn kind(&self, op: Operand) -> RegisterKind {
        match op {
            Operand::Register(reg) => self.register_kind[&reg],
            Operand::Literal(lit) => (&self.literals[&lit]).into(),
        }
    }
    pub fn op_loc(&self, op: Operand) -> SourceLocation {
        let (fn_id, slot) = self.operand_map[&op];
        (fn_id, self.symbols[&fn_id].slot_map[&slot]).into()
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
