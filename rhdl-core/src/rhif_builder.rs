use crate::{
    path::Path,
    rhif::{
        AluBinary, AluUnary, Array, Assign, Binary, BlockId, Case, CaseArgument, Cast,
        Discriminant, Enum, Exec, FieldValue, FuncId, If, Index, OpCode, Payload, Repeat, Return,
        Slot, Struct, Tuple, Unary,
    },
    TypedBits,
};

pub fn op_binary(op: AluBinary, lhs: Slot, arg1: Slot, arg2: Slot) -> OpCode {
    OpCode::Binary(Binary {
        op,
        lhs,
        arg1,
        arg2,
    })
}

pub fn op_unary(op: AluUnary, lhs: Slot, arg1: Slot) -> OpCode {
    OpCode::Unary(Unary { op, lhs, arg1 })
}

pub fn op_return(result: Option<Slot>) -> OpCode {
    OpCode::Return(Return { result })
}

pub fn op_if(lhs: Slot, cond: Slot, then_branch: BlockId, else_branch: BlockId) -> OpCode {
    OpCode::If(If {
        lhs,
        cond,
        then_branch,
        else_branch,
    })
}

pub fn op_index(lhs: Slot, arg: Slot, path: Path) -> OpCode {
    OpCode::Index(Index { lhs, arg, path })
}

pub fn op_assign(lhs: Slot, rhs: Slot, path: Path) -> OpCode {
    OpCode::Assign(Assign { lhs, rhs, path })
}

pub fn op_repeat(lhs: Slot, value: Slot, len: Slot) -> OpCode {
    OpCode::Repeat(Repeat { lhs, value, len })
}

pub fn op_struct(
    lhs: Slot,
    fields: Vec<FieldValue>,
    rest: Option<Slot>,
    template: TypedBits,
) -> OpCode {
    OpCode::Struct(Struct {
        lhs,
        fields,
        rest,
        template,
    })
}

pub fn op_tuple(lhs: Slot, fields: Vec<Slot>) -> OpCode {
    OpCode::Tuple(Tuple { lhs, fields })
}

pub fn op_block(block: BlockId) -> OpCode {
    OpCode::Block(block)
}

pub fn op_case(discriminant: Slot, table: Vec<(CaseArgument, BlockId)>) -> OpCode {
    OpCode::Case(Case {
        discriminant,
        table,
    })
}

pub fn op_exec(lhs: Slot, id: FuncId, args: Vec<Slot>) -> OpCode {
    OpCode::Exec(Exec { lhs, id, args })
}

pub fn op_array(lhs: Slot, elements: Vec<Slot>) -> OpCode {
    OpCode::Array(Array { lhs, elements })
}

pub fn op_discriminant(lhs: Slot, arg: Slot) -> OpCode {
    OpCode::Discriminant(Discriminant { lhs, arg })
}

pub fn op_enum(lhs: Slot, fields: Vec<FieldValue>, template: TypedBits) -> OpCode {
    OpCode::Enum(Enum {
        lhs,
        fields,
        template,
    })
}

pub fn op_as_bits(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::AsBits(Cast { lhs, arg, len })
}

pub fn op_as_signed(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::AsSigned(Cast { lhs, arg, len })
}

pub fn op_comment(comment: String) -> OpCode {
    OpCode::Comment(comment)
}
