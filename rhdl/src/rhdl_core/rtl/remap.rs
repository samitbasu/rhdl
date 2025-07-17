use crate::rhdl_core::rtl::visit::visit_operands_mut;

use super::spec::{OpCode, Operand};

pub fn remap_operands<F: FnMut(Operand) -> Operand>(mut op: OpCode, mut f: F) -> OpCode {
    visit_operands_mut(&mut op, |_sense, operand| *operand = f(*operand));
    op
}

pub fn rename_read_operands(op: OpCode, old: Operand, new: Operand) -> OpCode {
    remap_operands(op, |op| if op == old { new } else { op })
}
