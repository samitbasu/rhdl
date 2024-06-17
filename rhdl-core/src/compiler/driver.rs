use anyhow::anyhow;
use std::any::type_name;

use crate::{
    compiler::{
        mir::{compiler::compile_mir, infer::infer},
        passes::{
            //check_clock_coherence::CheckClockCoherence,
            check_clock_coherence::CheckClockCoherence,
            check_rhif_flow::DataFlowCheckPass,
            check_rhif_type::TypeCheckPass,
            lower_inferred_casts::LowerInferredCastsPass,
            pass::Pass,
            pre_cast_literals::PreCastLiterals,
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
    rhif::{spec::ExternalFunctionCode, Object},
    DigitalFn, KernelFnKind, Module,
};

type Result<T> = std::result::Result<T, RHDLError>;

fn wrap_pass<P: Pass>(obj: Object) -> Result<Object> {
    eprintln!("Running pass: {}", P::name());
    let obj = P::run(obj)?;
    let obj = SymbolTableIsComplete::run(obj)?;
    Ok(obj)
}

fn compile_kernel(kernel: Kernel) -> Result<Object> {
    let mir = compile_mir(kernel)?;
    let mut obj = infer(mir)?;
    //    let ctx = infer(&kernel)?;
    //    let _ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
    //    check_inference(&kernel, &ctx)?;
    //let mut obj = compile(kernel.inner(), ctx)?;
    let mut obj = SymbolTableIsComplete::run(obj)?;
    for _pass in 0..2 {
        eprintln!("{:?}", obj);
        obj = wrap_pass::<RemoveUnusedRegistersPass>(obj.clone())?;
        obj = wrap_pass::<RemoveUnneededMuxesPass>(obj.clone())?;
        obj = wrap_pass::<RemoveExtraRegistersPass>(obj.clone())?;
        obj = wrap_pass::<RemoveUnusedLiterals>(obj.clone())?;
        obj = wrap_pass::<PreCastLiterals>(obj.clone())?;
        obj = wrap_pass::<RemoveUselessCastsPass>(obj.clone())?;
        obj = wrap_pass::<RemoveEmptyCasesPass>(obj.clone())?;
        obj = wrap_pass::<RemoveUnusedRegistersPass>(obj.clone())?;
        obj = wrap_pass::<PrecomputeDiscriminantPass>(obj.clone())?;
        obj = wrap_pass::<LowerInferredCastsPass>(obj.clone())?;
    }
    obj = CheckClockCoherence::run(obj)?;
    obj = TypeCheckPass::run(obj)?;
    obj = DataFlowCheckPass::run(obj)?;
    eprintln!("Final code:\n{:?}", obj);
    Ok(obj)
}

fn elaborate_design(design: &mut Module) -> Result<()> {
    // Check for any uncompiled kernels
    let external_kernels = design
        .objects
        .values()
        .flat_map(|obj| obj.externals.iter())
        .filter_map(|func| {
            if let ExternalFunctionCode::Kernel(kernel) = &func.code {
                Some(kernel)
            } else {
                None
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    for kernel in external_kernels {
        if let std::collections::hash_map::Entry::Vacant(e) =
            design.objects.entry(kernel.inner().fn_id)
        {
            eprintln!("Compiling kernel {:?}", kernel.inner().fn_id);
            let obj = compile_kernel(kernel.clone())?;
            e.insert(obj);
        }
    }
    Ok(())
}

pub fn compile_design<K: DigitalFn>() -> Result<Module> {
    let Some(KernelFnKind::Kernel(kernel)) = K::kernel_fn() else {
        return Err(anyhow!("Missing kernel function provided for {}", type_name::<K>()).into());
    };
    let main = compile_kernel(kernel)?;
    let mut design = Module {
        objects: [(main.fn_id, main.clone())].into_iter().collect(),
        top: main.fn_id,
    };
    let mut object_count = design.objects.len();
    loop {
        elaborate_design(&mut design)?;
        if design.objects.len() == object_count {
            break;
        }
        object_count = design.objects.len();
    }
    Ok(design)
}
