use crate::rhif::spec::{OpCode, Slot};
use crate::rhif::Object;
use std::collections::HashMap;

// Given an object, computes a mapping from each slot that is written to the
// index of the opcode that writes to it.
pub fn slot_to_opcode(object: &Object) -> HashMap<Slot, usize> {
    let mut slot_to_opcode = HashMap::new();
    for (ndx, lop) in object.ops.iter().enumerate() {
        let op = &lop.op;
        match op {
            OpCode::Array(array) => {
                slot_to_opcode.insert(array.lhs, ndx);
            }
            OpCode::AsBits(cast) | OpCode::AsSigned(cast) => {
                slot_to_opcode.insert(cast.lhs, ndx);
            }
            OpCode::Assign(assign) => {
                slot_to_opcode.insert(assign.lhs, ndx);
            }
            OpCode::Binary(binary) => {
                slot_to_opcode.insert(binary.lhs, ndx);
            }
            OpCode::Case(case) => {
                slot_to_opcode.insert(case.lhs, ndx);
            }
            OpCode::Enum(enumerate) => {
                slot_to_opcode.insert(enumerate.lhs, ndx);
            }
            OpCode::Exec(exec) => {
                slot_to_opcode.insert(exec.lhs, ndx);
            }
            OpCode::Index(index) => {
                slot_to_opcode.insert(index.lhs, ndx);
            }
            OpCode::Repeat(repeat) => {
                slot_to_opcode.insert(repeat.lhs, ndx);
            }
            OpCode::Retime(retime) => {
                slot_to_opcode.insert(retime.lhs, ndx);
            }
            OpCode::Select(select) => {
                slot_to_opcode.insert(select.lhs, ndx);
            }
            OpCode::Splice(splice) => {
                slot_to_opcode.insert(splice.lhs, ndx);
            }
            OpCode::Struct(strukt) => {
                slot_to_opcode.insert(strukt.lhs, ndx);
            }
            OpCode::Tuple(tuple) => {
                slot_to_opcode.insert(tuple.lhs, ndx);
            }
            OpCode::Unary(unary) => {
                slot_to_opcode.insert(unary.lhs, ndx);
            }
            OpCode::Noop | OpCode::Comment(_) => {}
        }
    }
    slot_to_opcode
}
