use std::any::type_name;

use crate::{DigitalFn, KernelFnKind, RHDLError};

use super::{
    lower_rhif_to_rtl::compile_to_rtl,
    stage1::{compile_kernel, CompilationMode},
};
use anyhow::anyhow;

pub fn compile_design_stage1<K: DigitalFn>(
    mode: CompilationMode,
) -> Result<crate::rhif::Object, RHDLError> {
    let Some(KernelFnKind::Kernel(kernel)) = K::kernel_fn() else {
        return Err(anyhow!("Missing kernel function provided for {}", type_name::<K>()).into());
    };
    compile_kernel(kernel, mode)
}

pub fn compile_design_stage2(object: crate::rhif::Object) -> Result<crate::rtl::Object, RHDLError> {
    compile_to_rtl(&object)
}

pub fn compile_design<K: DigitalFn>(
    mode: CompilationMode,
) -> Result<crate::rtl::Object, RHDLError> {
    let rhif = compile_design_stage1::<K>(mode)?;
    compile_design_stage2(rhif)
}
