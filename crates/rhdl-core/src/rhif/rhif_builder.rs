use crate::rhdl_core::{
    Color, TypedBits,
    ast::ast_impl::WrapOp,
    rhif::spec::{
        AluBinary, AluUnary, Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec,
        FieldValue, FuncId, Index, OpCode, Repeat, Slot, Struct, Tuple, Unary,
    },
    types::path::Path,
};

use super::spec::{Retime, Select, Splice, Wrap};

pub fn op_binary(op: AluBinary, lhs: Slot, arg1: Slot, arg2: Slot) -> OpCode {
    OpCode::Binary(Binary {
        op,
        lhs,
        arg1,
        arg2,
    })
}

pub fn op_retime(lhs: Slot, arg: Slot, color: Option<Color>) -> OpCode {
    OpCode::Retime(Retime { lhs, arg, color })
}

pub fn op_unary(op: AluUnary, lhs: Slot, arg1: Slot) -> OpCode {
    OpCode::Unary(Unary { op, lhs, arg1 })
}

pub fn op_select(lhs: Slot, cond: Slot, true_value: Slot, false_value: Slot) -> OpCode {
    OpCode::Select(Select {
        lhs,
        cond,
        true_value,
        false_value,
    })
}

pub fn op_index(lhs: Slot, arg: Slot, path: Path) -> OpCode {
    OpCode::Index(Index { lhs, arg, path })
}

pub fn op_assign(lhs: Slot, rhs: Slot) -> OpCode {
    OpCode::Assign(Assign { lhs, rhs })
}

pub fn op_splice(lhs: Slot, rhs: Slot, path: Path, arg: Slot) -> OpCode {
    OpCode::Splice(Splice {
        lhs,
        orig: rhs,
        path,
        subst: arg,
    })
}

pub fn op_repeat(lhs: Slot, value: Slot, len: u64) -> OpCode {
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

pub fn op_case(lhs: Slot, discriminant: Slot, table: Vec<(CaseArgument, Slot)>) -> OpCode {
    OpCode::Case(Case {
        lhs,
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

pub fn op_enum(lhs: Slot, fields: Vec<FieldValue>, template: TypedBits) -> OpCode {
    OpCode::Enum(Enum {
        lhs,
        fields,
        template,
    })
}

pub fn op_as_bits(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::AsBits(Cast {
        lhs,
        arg,
        len: Some(len),
    })
}

pub fn op_resize_inferred(lhs: Slot, arg: Slot) -> OpCode {
    OpCode::Resize(Cast {
        lhs,
        arg,
        len: None,
    })
}

pub fn op_resize(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::Resize(Cast {
        lhs,
        arg,
        len: Some(len),
    })
}

pub fn op_as_bits_inferred(lhs: Slot, arg: Slot) -> OpCode {
    OpCode::AsBits(Cast {
        lhs,
        arg,
        len: None,
    })
}

pub fn op_as_signed(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::AsSigned(Cast {
        lhs,
        arg,
        len: Some(len),
    })
}

pub fn op_as_signed_inferred(lhs: Slot, arg: Slot) -> OpCode {
    OpCode::AsSigned(Cast {
        lhs,
        arg,
        len: None,
    })
}

pub fn op_comment(comment: String) -> OpCode {
    OpCode::Comment(comment)
}

pub fn op_wrap(lhs: Slot, arg: Slot, op: WrapOp) -> OpCode {
    OpCode::Wrap(Wrap {
        lhs,
        arg,
        op,
        kind: None,
    })
}

pub fn op_cast(lhs: Slot, arg: Slot, len: usize) -> OpCode {
    OpCode::Resize(Cast {
        lhs,
        arg,
        len: Some(len),
    })
}
