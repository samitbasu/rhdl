use crate::{
    ast::source_location::SourceLocation,
    rtl::{
        object::RegisterKind,
        spec::{LiteralId, Operand, RegisterId},
        Object,
    },
    types::bit_string::BitString,
};

pub(crate) mod check_no_zero_resize;
pub(crate) mod dead_code_elimination;
pub(crate) mod lower_empty_splice_to_copy;
pub(crate) mod lower_index_all_to_copy;
pub(crate) mod lower_multiply_to_shift;
pub(crate) mod lower_not_equal_zero_to_any;
pub(crate) mod lower_shift_by_constant;
pub(crate) mod lower_shifts_by_zero_to_copy;
pub(crate) mod lower_signal_casts;
pub(crate) mod lower_single_concat_to_copy;
pub(crate) mod pass;
pub(crate) mod remove_empty_function_arguments;
pub(crate) mod remove_extra_registers;
pub(crate) mod remove_unused_operands;
pub(crate) mod strip_empty_args_from_concat;
pub(crate) mod symbol_table_is_complete;

fn allocate_register(input: &mut Object, kind: RegisterKind, loc: SourceLocation) -> RegisterId {
    let reg = input.reg_max_index().next();
    input.register_kind.insert(reg, kind);
    input
        .symbols
        .operand_map
        .insert(Operand::Register(reg), loc);
    reg
}

fn allocate_literal(input: &mut Object, loc: SourceLocation, bs: BitString) -> LiteralId {
    let lit = input.literal_max_index().next();
    input.literals.insert(lit, bs);
    input.symbols.operand_map.insert(Operand::Literal(lit), loc);
    lit
}
