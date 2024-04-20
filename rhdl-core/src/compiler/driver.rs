use crate::{
    compiler::{
        ascii::render_ast_to_string,
        assign_node_ids, compile, infer,
        passes::{
            check_inference::check_inference, check_rhif_flow::DataFlowCheckPass,
            check_rhif_type::TypeCheckPass, pass::Pass, pre_cast_literals::PreCastLiterals,
            remove_extra_registers::RemoveExtraRegistersPass,
            remove_unneeded_muxes::RemoveUnneededMuxesPass,
            remove_unused_literals::RemoveUnusedLiterals,
            remove_useless_casts::RemoveUselessCastsPass,
        },
    },
    kernel::Kernel,
    rhif::{spec::ExternalFunctionCode, Object},
    Module,
};

use anyhow::Result;

pub fn compile_kernel(mut kernel: Kernel) -> Result<Object> {
    assign_node_ids(&mut kernel)?;
    let ctx = infer(&kernel)?;
    let _ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
    check_inference(&kernel, &ctx)?;
    let mut obj = compile(kernel.inner(), ctx)?;
    eprintln!("{}", obj);
    for _pass in 0..2 {
        obj = RemoveExtraRegistersPass::run(obj)?;
        obj = RemoveUnneededMuxesPass::run(obj)?;
        obj = RemoveExtraRegistersPass::run(obj)?;
        obj = RemoveUnusedLiterals::run(obj)?;
        obj = PreCastLiterals::run(obj)?;
        obj = RemoveUselessCastsPass::run(obj)?;
    }
    let obj = TypeCheckPass::run(obj)?;
    let obj = DataFlowCheckPass::run(obj)?;
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
            eprintln!("Compiling kernel {}", kernel.inner().fn_id);
            let obj = compile_kernel(kernel.clone())?;
            e.insert(obj);
        }
    }
    Ok(())
}

pub fn compile_design(top: Kernel) -> Result<Module> {
    let main = compile_kernel(top)?;
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
