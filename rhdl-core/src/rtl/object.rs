use std::collections::BTreeMap;
use std::fmt::Write;

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::{
        object::SymbolMap,
        spec::{ExternalFunction, FuncId, Slot},
    },
    Kind, TypedBits,
};

use super::spec::{LiteralId, OpCode, Operand, RegisterId};

#[derive(Clone)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub id: NodeId,
}

impl LocatedOpCode {
    pub fn new(op: OpCode, id: NodeId) -> Self {
        Self { op, id }
    }
}

impl From<(OpCode, NodeId)> for LocatedOpCode {
    fn from((op, id): (OpCode, NodeId)) -> Self {
        Self::new(op, id)
    }
}

#[derive(Clone)]
pub enum BitString {
    Signed(Vec<bool>),
    Unsigned(Vec<bool>),
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitString::Signed(bits) => {
                write!(f, "s")?;
                for bit in bits {
                    write!(f, "{}", if *bit { "1" } else { "0" })?;
                }
                Ok(())
            }
            BitString::Unsigned(bits) => {
                write!(f, "b")?;
                for bit in bits {
                    write!(f, "{}", if *bit { "1" } else { "0" })?;
                }
                Ok(())
            }
        }
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

#[derive(Clone)]
pub enum RegisterKind {
    Signed(usize),
    Unsigned(usize),
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
    pub operand_map: BTreeMap<Operand, Slot>,
    pub register_kind: BTreeMap<RegisterId, RegisterKind>,
    pub return_register: Option<Operand>,
    pub externals: BTreeMap<FuncId, ExternalFunction>,
    pub ops: Vec<LocatedOpCode>,
    pub arguments: Vec<Option<RegisterId>>,
    pub name: String,
    pub fn_id: FunctionId,
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Object {}", self.name)?;
        writeln!(f, "  fn_id {:?}", self.fn_id)?;
        writeln!(f, "  return_register {:?}", self.return_register)?;
        for (reg, kind) in &self.register_kind {
            writeln!(f, "Reg {reg:?} : {kind:?}")?;
        }
        for (id, literal) in &self.literals {
            writeln!(f, "Lit {id:?} : {literal:?}")?;
        }
        for (ndx, func) in self.externals.iter() {
            writeln!(
                f,
                "Function {:?} name: {} code: {:?} signature: {:?}",
                ndx, func.path, func.code, func.signature
            )?;
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
