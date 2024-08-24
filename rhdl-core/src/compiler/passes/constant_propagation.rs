use crate::{
    ast::ast_impl::NodeId,
    rhif::{
        object::LocatedOpCode,
        spec::{AluBinary, Assign, Binary, LiteralId, OpCode, Slot},
        Object,
    },
    Digital,
};

use super::pass::Pass;

pub struct ConstantPropagation {}

fn propogate_binary(id: NodeId, binary: Binary, obj: &mut Object) -> OpCode {
    let Binary {
        op,
        lhs,
        arg1,
        arg2,
    } = binary;
    if let (Slot::Literal(arg1_lit), Slot::Literal(arg2_lit)) = (arg1, arg2) {
        let arg1_val = obj.literals[&arg1_lit].clone();
        let arg2_val = obj.literals[&arg2_lit].clone();
        let rhs = match op {
            AluBinary::Add => (arg1_val + arg2_val).unwrap(),
            AluBinary::Sub => (arg1_val - arg2_val).unwrap(),
            AluBinary::BitXor => (arg1_val ^ arg2_val).unwrap(),
            AluBinary::BitAnd => (arg1_val & arg2_val).unwrap(),
            AluBinary::BitOr => (arg1_val | arg2_val).unwrap(),
            AluBinary::Eq => (arg1_val == arg2_val).typed_bits(),
            AluBinary::Ne => (arg1_val != arg2_val).typed_bits(),
            AluBinary::Shl => (arg1_val << arg2_val).unwrap(),
            AluBinary::Shr => (arg1_val >> arg2_val).unwrap(),
            AluBinary::Lt => (arg1_val < arg2_val).typed_bits(),
            AluBinary::Le => (arg1_val <= arg2_val).typed_bits(),
            AluBinary::Gt => (arg1_val > arg2_val).typed_bits(),
            AluBinary::Ge => (arg1_val >= arg2_val).typed_bits(),
            _ => todo!(),
        };
        let literal = LiteralId(obj.literal_max_index().0 + 1);
        obj.literals.insert(literal, rhs);
        obj.symbols.slot_map.insert(Slot::Literal(literal), id);
        OpCode::Assign(Assign {
            lhs,
            rhs: Slot::Literal(literal),
        })
    } else {
        OpCode::Binary(binary)
    }
}

impl Pass for ConstantPropagation {
    fn name() -> &'static str {
        "constant_propagation"
    }

    fn run(mut input: crate::rhif::Object) -> Result<crate::rhif::Object, crate::RHDLError> {
        let ops = input
            .ops
            .clone()
            .into_iter()
            .map(|lop| match lop.op {
                OpCode::Binary(binary) => LocatedOpCode {
                    op: propogate_binary(lop.id, binary, &mut input),
                    id: lop.id,
                },
                _ => lop,
            })
            .collect();
        input.ops = ops;
        Ok(input)
    }
}
