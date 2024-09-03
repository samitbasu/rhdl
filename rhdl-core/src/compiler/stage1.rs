use crate::{
    compiler::{
        mir::{compiler::compile_mir, infer::infer},
        rhif_passes::{
            check_clock_coherence::CheckClockCoherence,
            check_for_rolled_types::CheckForRolledTypesPass, check_rhif_flow::DataFlowCheckPass,
            check_rhif_type::TypeCheckPass, constant_propagation::ConstantPropagation,
            dead_code_elimination::DeadCodeEliminationPass,
            lower_dynamic_indices_with_constant_arguments::LowerDynamicIndicesWithConstantArguments,
            lower_inferred_casts::LowerInferredCastsPass,
            lower_inferred_retimes::LowerInferredRetimesPass, pass::Pass,
            pre_cast_literals::PreCastLiterals,
            precast_integer_literals_in_binops::PrecastIntegerLiteralsInBinops,
            precompute_discriminants::PrecomputeDiscriminantPass,
            remove_empty_cases::RemoveEmptyCasesPass,
            remove_extra_registers::RemoveExtraRegistersPass,
            remove_unneeded_muxes::RemoveUnneededMuxesPass,
            remove_unused_literals::RemoveUnusedLiterals,
            remove_unused_registers::RemoveUnusedRegistersPass,
            remove_useless_casts::RemoveUselessCastsPass,
            symbol_table_is_complete::SymbolTableIsComplete,
        },
    },
    error::RHDLError,
    kernel::Kernel,
    rhif::Object,
};

type Result<T> = std::result::Result<T, RHDLError>;

fn wrap_pass<P: Pass>(obj: Object) -> Result<Object> {
    eprintln!("Running pass: {}", P::name());
    let obj = P::run(obj)?;
    let obj = SymbolTableIsComplete::run(obj)?;
    Ok(obj)
}

#[derive(Debug, Clone, Copy)]
pub enum CompilationMode {
    Asynchronous,
    Synchronous,
}

pub(crate) fn compile(kernel: Kernel, mode: CompilationMode) -> Result<Object> {
    let mir = compile_mir(kernel, mode)?;
    let mut obj = infer(mir)?;
    obj = SymbolTableIsComplete::run(obj)?;
    obj = wrap_pass::<CheckForRolledTypesPass>(obj)?;
    let mut hash = obj.hash_value();
    loop {
        obj = wrap_pass::<RemoveUnneededMuxesPass>(obj)?;
        obj = wrap_pass::<RemoveExtraRegistersPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedLiterals>(obj)?;
        obj = wrap_pass::<RemoveUselessCastsPass>(obj)?;
        obj = wrap_pass::<RemoveEmptyCasesPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedRegistersPass>(obj)?;
        obj = wrap_pass::<DeadCodeEliminationPass>(obj)?;
        let new_hash = obj.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    if matches!(mode, CompilationMode::Asynchronous) {
        obj = CheckClockCoherence::run(obj)?;
    }
    let mut hash = obj.hash_value();
    loop {
        obj = wrap_pass::<RemoveUnneededMuxesPass>(obj)?;
        obj = wrap_pass::<RemoveExtraRegistersPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedLiterals>(obj)?;
        obj = wrap_pass::<PreCastLiterals>(obj)?;
        obj = wrap_pass::<RemoveUselessCastsPass>(obj)?;
        obj = wrap_pass::<RemoveEmptyCasesPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedRegistersPass>(obj)?;
        obj = wrap_pass::<DeadCodeEliminationPass>(obj)?;
        obj = wrap_pass::<PrecomputeDiscriminantPass>(obj)?;
        obj = wrap_pass::<LowerInferredCastsPass>(obj)?;
        obj = wrap_pass::<PrecastIntegerLiteralsInBinops>(obj)?;
        obj = wrap_pass::<LowerInferredRetimesPass>(obj)?;
        obj = wrap_pass::<LowerDynamicIndicesWithConstantArguments>(obj)?;
        obj = wrap_pass::<ConstantPropagation>(obj)?;
        let new_hash = obj.hash_value();
        if new_hash == hash {
            break;
        }
        hash = new_hash;
    }
    obj = TypeCheckPass::run(obj)?;
    obj = DataFlowCheckPass::run(obj)?;
    eprintln!("Final code:\n{:?}", obj);
    Ok(obj)
}
