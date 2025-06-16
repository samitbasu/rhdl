use crate::{
    prelude::HDLDescriptor,
    rhdl_core::{
        ast::{
            ast_impl::{FunctionId, NodeId},
            source::{source_location::SourceLocation, spanned_source_set::SpannedSourceSet},
        },
        ntl::{
            spec::{OpCode, Operand, RegisterId},
            visit::{visit_operands, visit_operands_mut},
        },
    },
};
use std::hash::Hash;
use std::hash::Hasher;

use fnv::FnvHasher;

#[derive(Clone, Hash, PartialEq, Copy, Debug)]
pub enum BlackBoxMode {
    Synchronous,
    Asynchronous,
}

#[derive(Clone, Hash)]
pub struct BlackBox {
    pub code: HDLDescriptor,
    pub mode: BlackBoxMode,
}

#[derive(Clone, Default, Hash)]
pub struct Object {
    pub name: String,
    pub inputs: Vec<Vec<RegisterId>>,
    pub outputs: Vec<Operand>,
    pub ops: Vec<LocatedOpCode>,
    pub code: SpannedSourceSet,
    pub black_boxes: Vec<BlackBox>,
}

impl Object {
    pub fn max_reg(&self) -> u32 {
        let mut max_reg: u32 = 0;
        for inputs in &self.inputs {
            for input in inputs {
                max_reg = max_reg.max(input.raw())
            }
        }
        for output in self.outputs.iter().flat_map(Operand::reg) {
            max_reg = max_reg.max(output.raw())
        }
        for lop in &self.ops {
            visit_operands(&lop.op, |_sense, op| {
                if let Some(reg) = op.reg() {
                    max_reg = max_reg.max(reg.raw())
                }
            });
        }
        max_reg
    }

    /// Link another netlist, and return the offset added
    /// to registers
    pub fn link(&mut self, other: &Object) -> u32 {
        log::info!("Root {:?}", self);
        log::info!("Other {:?}", other);
        let max_reg = self.max_reg() + 1;
        let mut other_ops = other.ops.clone();
        for lop in &mut other_ops {
            visit_operands_mut(&mut lop.op, |op| {
                if let Some(reg) = op.reg() {
                    *op = Operand::Register(reg.offset(max_reg));
                }
            });
        }
        // Fix up black box references
        let bb_offset = self.black_boxes.len();
        for lop in &mut other_ops {
            if let OpCode::BlackBox(blackbox) = &mut lop.op {
                blackbox.code = blackbox.code.offset(bb_offset);
            }
        }
        self.ops.extend(other_ops);
        self.code.extend(other.code.sources.clone());
        self.black_boxes.extend(other.black_boxes.clone());
        log::info!("Linked {:?}", self);
        max_reg
    }
    pub fn hash_value(&self) -> u64 {
        let mut hasher = FnvHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Hash)]
pub struct LocatedOpCode {
    pub op: OpCode,
    pub loc: Option<SourceLocation>,
}

impl LocatedOpCode {
    pub fn new(op: OpCode, id: NodeId, func: FunctionId) -> Self {
        Self {
            op,
            loc: Some(SourceLocation { node: id, func }),
        }
    }
}
