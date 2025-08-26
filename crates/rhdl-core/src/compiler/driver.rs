use std::any::type_name;

use crate::rhdl_core::{DigitalFn, KernelFnKind, RHDLError};

use super::stage1::CompilationMode;
use anyhow::anyhow;

pub fn compile_design_stage1<K: DigitalFn>(
    mode: CompilationMode,
) -> Result<crate::rhdl_core::rhif::Object, RHDLError> {
    let Some(KernelFnKind::Kernel(kernel)) = K::kernel_fn() else {
        return Err(anyhow!("Missing kernel function provided for {}", type_name::<K>()).into());
    };
    super::stage1::compile(kernel, mode)
}

pub fn compile_design_stage2(
    object: &crate::rhdl_core::rhif::Object,
) -> Result<crate::rhdl_core::rtl::Object, RHDLError> {
    super::stage2::compile(object)
}

pub fn compile_design<K: DigitalFn>(
    mode: CompilationMode,
) -> Result<crate::rhdl_core::rtl::Object, RHDLError> {
    let rhif = compile_design_stage1::<K>(mode)?;
    compile_design_stage2(&rhif)
}
