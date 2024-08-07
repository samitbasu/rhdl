use anyhow::anyhow;
use std::any::type_name;

use crate::{
    compiler::{
        mir::{compiler::compile_mir, infer::infer},
        passes::{
            check_clock_coherence::CheckClockCoherence,
            check_for_rolled_types::CheckForRolledTypesPass, check_rhif_flow::DataFlowCheckPass,
            check_rhif_type::TypeCheckPass, dead_code_elimination::DeadCodeEliminationPass,
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
    DigitalFn, KernelFnKind, Module,
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

fn compile_kernel(kernel: Kernel, mode: CompilationMode) -> Result<Object> {
    let mir = compile_mir(kernel)?;
    let mut obj = infer(mir)?;
    obj = SymbolTableIsComplete::run(obj)?;
    obj = wrap_pass::<CheckForRolledTypesPass>(obj)?;
    // TODO - Remove the iteration count
    for _pass in 0..5 {
        eprintln!("{:?}", obj);
        obj = wrap_pass::<RemoveUnneededMuxesPass>(obj)?;
        obj = wrap_pass::<RemoveExtraRegistersPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedLiterals>(obj)?;
        obj = wrap_pass::<RemoveUselessCastsPass>(obj)?;
        obj = wrap_pass::<RemoveEmptyCasesPass>(obj)?;
        obj = wrap_pass::<RemoveUnusedRegistersPass>(obj)?;
        obj = wrap_pass::<DeadCodeEliminationPass>(obj)?;
    }
    if matches!(mode, CompilationMode::Asynchronous) {
        obj = CheckClockCoherence::run(obj)?;
    }
    for _pass in 0..2 {
        eprintln!("{:?}", obj);
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
    }
    obj = TypeCheckPass::run(obj)?;
    obj = DataFlowCheckPass::run(obj)?;
    eprintln!("Final code:\n{:?}", obj);
    Ok(obj)
}

fn elaborate_design(design: &mut Module, mode: CompilationMode) -> Result<()> {
    // Check for any uncompiled kernels
    let external_kernels = design
        .objects
        .values()
        .flat_map(|obj| obj.externals.iter())
        .map(|(_, func)| func.code.clone())
        .collect::<Vec<_>>();
    for kernel in external_kernels {
        if let std::collections::hash_map::Entry::Vacant(e) =
            design.objects.entry(kernel.inner().fn_id)
        {
            eprintln!("Compiling kernel {:?}", kernel.inner().fn_id);
            let obj = compile_kernel(kernel.clone(), mode)?;
            e.insert(obj);
        }
    }
    Ok(())
}

pub fn compile_design<K: DigitalFn>(mode: CompilationMode) -> Result<Module> {
    let Some(KernelFnKind::Kernel(kernel)) = K::kernel_fn() else {
        return Err(anyhow!("Missing kernel function provided for {}", type_name::<K>()).into());
    };
    let main = compile_kernel(kernel, mode)?;
    let mut design = Module {
        objects: [(main.fn_id, main.clone())].into_iter().collect(),
        top: main.fn_id,
    };
    let mut object_count = design.objects.len();
    loop {
        elaborate_design(&mut design, mode)?;
        if design.objects.len() == object_count {
            break;
        }
        object_count = design.objects.len();
    }
    Ok(design)
}
