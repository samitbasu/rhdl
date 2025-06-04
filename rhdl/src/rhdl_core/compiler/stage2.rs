use crate::rhdl_core::{
    compiler::{
        lower_rhif_to_rtl::compile_to_rtl,
        rtl_passes::{
            check_no_zero_resize::CheckNoZeroResize, constant_propagation::ConstantPropagationPass,
            dead_code_elimination::DeadCodeEliminationPass,
            lower_empty_splice_to_copy::LowerEmptySpliceToCopy,
            lower_index_all_to_copy::LowerIndexAllToCopy,
            lower_multiply_to_shift::LowerMultiplyToShift,
            lower_not_equal_zero_to_any::LowerNotEqualZeroToAny,
            lower_shift_by_constant::LowerShiftByConstant,
            lower_shifts_by_zero_to_copy::LowerShiftsByZeroToCopy,
            lower_signal_casts::LowerSignalCasts,
            lower_single_concat_to_copy::LowerSingleConcatToCopy, pass::Pass,
            remove_empty_function_arguments::RemoveEmptyFunctionArguments,
            remove_extra_registers::RemoveExtraRegistersPass,
            remove_unused_operands::RemoveUnusedOperandsPass,
            strip_empty_args_from_concat::StripEmptyArgsFromConcat,
            symbol_table_is_complete::SymbolTableIsComplete,
        },
    },
    rtl, RHDLError,
};
use log::{debug, info};

type Result<T> = std::result::Result<T, RHDLError>;

fn wrap_pass<P: Pass>(obj: rtl::Object) -> Result<rtl::Object> {
    info!("Running Stage 2 compiler Pass {}", P::description());
    P::run(obj)
}

pub(crate) fn compile(object: &crate::rhdl_core::rhif::Object) -> Result<rtl::Object> {
    let mut rtl = compile_to_rtl(object)?;
    let mut hash = rtl.hash_value();
    loop {
        rtl = wrap_pass::<LowerSignalCasts>(rtl)?;
        rtl = wrap_pass::<RemoveExtraRegistersPass>(rtl)?;
        rtl = wrap_pass::<SymbolTableIsComplete>(rtl)?;
        rtl = wrap_pass::<RemoveUnusedOperandsPass>(rtl)?;
        rtl = wrap_pass::<StripEmptyArgsFromConcat>(rtl)?;
        rtl = wrap_pass::<DeadCodeEliminationPass>(rtl)?;
        rtl = wrap_pass::<LowerEmptySpliceToCopy>(rtl)?;
        rtl = wrap_pass::<LowerSingleConcatToCopy>(rtl)?;
        rtl = wrap_pass::<LowerIndexAllToCopy>(rtl)?;
        rtl = wrap_pass::<RemoveEmptyFunctionArguments>(rtl)?;
        rtl = wrap_pass::<LowerMultiplyToShift>(rtl)?;
        rtl = wrap_pass::<LowerShiftByConstant>(rtl)?;
        rtl = wrap_pass::<LowerShiftsByZeroToCopy>(rtl)?;
        rtl = wrap_pass::<LowerNotEqualZeroToAny>(rtl)?;
        rtl = wrap_pass::<ConstantPropagationPass>(rtl)?;
        let new_hash = rtl.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    rtl = wrap_pass::<CheckNoZeroResize>(rtl)?;
    debug!("{rtl:?}");
    Ok(rtl)
}
