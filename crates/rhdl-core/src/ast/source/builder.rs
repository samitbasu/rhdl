use crate::{
    RHDLError,
    ast::ast_impl::{KernelFn, NodeId},
};
use anyhow::anyhow;

use super::spanned_source::SpannedSource;

pub fn build_spanned_source_for_kernel(kernel: &KernelFn) -> Result<SpannedSource, RHDLError> {
    let Some(filename) = kernel.text else {
        return Err(anyhow!("Kernel function has no source text").into());
    };
    let source = std::fs::read_to_string(filename)
        .map_err(|err| anyhow!("Failed to read source file {}: {}", filename, err))?;
    let span_map = kernel
        .meta_db
        .iter()
        .map(|(id, meta)| {
            let node_id = NodeId::new(*id);
            let start_col = meta.span.start_col;
            let start_line = meta.span.start_line;
            let end_col = meta.span.end_col;
            let end_line = meta.span.end_line;
            let start_source_offset =
                miette::SourceOffset::from_location(&source, start_line, start_col);
            let end_source_offset = miette::SourceOffset::from_location(&source, end_line, end_col);
            (
                node_id,
                start_source_offset.offset()..end_source_offset.offset(),
            )
        })
        .collect();
    Ok(SpannedSource {
        source,
        name: kernel.name.into(),
        span_map,
        fallback: kernel.id,
        filename: filename.into(),
        function_id: kernel.fn_id,
    })
}
