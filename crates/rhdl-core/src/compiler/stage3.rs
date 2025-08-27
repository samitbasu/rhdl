use log::info;

use crate::{
    RHDLError,
    {
        compiler::ntl_passes::{
            check_for_undriven::CheckForUndriven, constant_propagation::ConstantPropagationPass,
            constant_reg_elimination::ConstantRegisterElimination,
            dead_code_elimination::DeadCodeElimination, lower_any_all::LowerAnyAll,
            lower_bitwise_op_with_constant::LowerBitwiseOpWithConstant, lower_case::LowerCase,
            lower_selects::LowerSelects, pass::Pass,
            remove_extra_literals::RemoveExtraLiteralsPass,
            remove_extra_registers::RemoveExtraRegistersPass,
            reorder_instructions::ReorderInstructions, single_write::SingleRegisterWrite,
            symbol_table_is_complete::SymbolTableIsComplete,
        },
        ntl::Object,
    },
};

fn wrap_pass<P: Pass>(obj: Object) -> Result<Object, RHDLError> {
    info!("Running Stage 3 compiler Pass {}", P::description());
    P::run(obj)
}

pub fn optimize_ntl(mut input: Object) -> Result<Object, RHDLError> {
    let mut hash = input.hash_value();
    input = wrap_pass::<SymbolTableIsComplete>(input)?;
    loop {
        input = wrap_pass::<RemoveExtraLiteralsPass>(input)?;
        input = wrap_pass::<ConstantRegisterElimination>(input)?;
        input = wrap_pass::<LowerCase>(input)?;
        input = wrap_pass::<LowerSelects>(input)?;
        input = wrap_pass::<RemoveExtraRegistersPass>(input)?;
        input = wrap_pass::<ConstantPropagationPass>(input)?;
        input = wrap_pass::<LowerBitwiseOpWithConstant>(input)?;
        input = wrap_pass::<LowerAnyAll>(input)?;
        input = wrap_pass::<DeadCodeElimination>(input)?;
        let new_hash = input.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    input = wrap_pass::<SymbolTableIsComplete>(input)?;
    input = wrap_pass::<SingleRegisterWrite>(input)?;
    input = wrap_pass::<ReorderInstructions>(input)?;
    input = wrap_pass::<CheckForUndriven>(input)?;
    Ok(input)
}
