use crate::{
    ast::ascii::render_ast_to_string,
    compiler::{
        assign_node_ids, check_inference::check_inference, check_rhif_flow::DataFlowCheckPass,
        check_rhif_type::TypeCheckPass, compile, infer, lower_index_to_copy::LowerIndexToCopy,
        pass::Pass, remove_extra_registers::RemoveExtraRegistersPass,
    },
    diagnostic::report::show_source,
    kernel::Kernel,
    rhif::Object,
    Design, KernelFnKind,
};

use anyhow::Result;

pub fn compile_kernel(mut kernel: Kernel) -> Result<Object> {
    assign_node_ids(&mut kernel)?;
    let ctx = infer(&kernel)?;
    let ast_ascii = render_ast_to_string(&kernel, &ctx).unwrap();
    //eprintln!("{}", ast_ascii);
    check_inference(&kernel, &ctx)?;
    let obj = compile(&kernel.ast, ctx)?;
    //    let obj = LowerIndexToCopy::run(obj)?;
    //eprintln!("{}", obj);
    let obj = RemoveExtraRegistersPass::run(obj)?;
    //eprintln!("{}", obj);
    let obj = TypeCheckPass::run(obj)?;
    let obj = DataFlowCheckPass::run(obj)?;
    //eprintln!("{}", obj);
    /*
    if let Some(source) = obj.source.as_ref() {
        for (reg, loc) in &obj.register_map {
            show_source(source, &reg.to_string(), loc.node);
        }
    }*/
    Ok(obj)
}

fn elaborate_design(design: &mut Design) -> Result<()> {
    // Check for any uncompiled kernels
    let external_kernels = design
        .objects
        .values()
        .flat_map(|obj| obj.externals.iter())
        .filter_map(|func| {
            if let KernelFnKind::Kernel(kernel) = &func.code {
                Some(kernel)
            } else {
                None
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    for kernel in external_kernels {
        if let std::collections::hash_map::Entry::Vacant(e) = design.objects.entry(kernel.fn_id) {
            eprintln!("Compiling kernel {}", kernel.fn_id);
            let obj = compile_kernel(Kernel {
                ast: kernel.clone(),
            })?;
            e.insert(obj);
        }
    }
    Ok(())
}

pub fn compile_design(top: Kernel) -> Result<Design> {
    let main = compile_kernel(top)?;
    let mut design = Design {
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
