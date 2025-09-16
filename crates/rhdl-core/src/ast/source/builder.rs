use crate::{
    RHDLError,
    ast::ast_impl::{KernelFn, NodeId},
};
use anyhow::anyhow;

use super::spanned_source::SpannedSource;

pub fn build_spanned_source_for_kernel(kernel: &KernelFn) -> Result<SpannedSource, RHDLError> {}
