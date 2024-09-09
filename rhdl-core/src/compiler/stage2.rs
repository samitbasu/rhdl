use crate::{
    compiler::{
        lower_rhif_to_rtl::compile_to_rtl,
        rtl_passes::{
            check_no_zero_resize::CheckNoZeroResize,
            dead_code_elimination::DeadCodeEliminationPass,
            lower_empty_splice_to_copy::LowerEmptySpliceToCopy,
            lower_index_all_to_copy::LowerIndexAllToCopy,
            lower_multiply_to_shift::LowerMultiplyToShift,
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

type Result<T> = std::result::Result<T, RHDLError>;

pub(crate) fn compile(object: &crate::rhif::Object) -> Result<rtl::Object> {
    let mut rtl = compile_to_rtl(object)?;
    let mut hash = rtl.hash_value();
    loop {
        rtl = LowerSignalCasts::run(rtl)?;
        rtl = RemoveExtraRegistersPass::run(rtl)?;
        rtl = SymbolTableIsComplete::run(rtl)?;
        rtl = RemoveUnusedOperandsPass::run(rtl)?;
        rtl = StripEmptyArgsFromConcat::run(rtl)?;
        rtl = DeadCodeEliminationPass::run(rtl)?;
        rtl = LowerEmptySpliceToCopy::run(rtl)?;
        rtl = LowerSingleConcatToCopy::run(rtl)?;
        rtl = LowerIndexAllToCopy::run(rtl)?;
        rtl = RemoveEmptyFunctionArguments::run(rtl)?;
        rtl = LowerMultiplyToShift::run(rtl)?;
        rtl = LowerShiftByConstant::run(rtl)?;
        rtl = LowerShiftsByZeroToCopy::run(rtl)?;
        let new_hash = rtl.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    rtl = CheckNoZeroResize::run(rtl)?;
    eprintln!("{rtl:?}");
    Ok(rtl)
}
