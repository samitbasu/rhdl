pub mod check_for_undriven;
pub mod constant_propagation;
pub mod constant_reg_elimination;
pub mod dead_code_elimination;
pub mod lower_any_all;
pub mod lower_bitwise_op_with_constant;
pub mod lower_case;
pub mod lower_selects;
pub mod pass;
pub mod remove_extra_literals;
pub mod remove_extra_registers;
pub mod reorder_instructions;
pub mod single_write;
pub mod symbol_table_is_complete;
/*
pub mod check_for_unconnected_clock_reset;
pub mod check_for_undriven;
*/
