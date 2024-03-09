use crate::{ast::ast_impl::FunctionId, diagnostic::SpannedSource, rhif::Object};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Module {
    pub objects: HashMap<FunctionId, Object>,
    pub top: FunctionId,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Design {}", self.top)?;
        for obj in self.objects.values() {
            write!(f, "\n  Object {}", obj.name)?;
        }
        Ok(())
    }
}

impl Module {
    pub fn func_name(&self, fn_id: FunctionId) -> Result<String> {
        let obj = self
            .objects
            .get(&fn_id)
            .ok_or(anyhow::anyhow!("Function {fn_id} not found"))?;
        Ok(format!("{}_{:x}", obj.name, fn_id))
    }
    pub fn source_map(&self) -> HashMap<FunctionId, SpannedSource> {
        self.objects
            .iter()
            .filter_map(|(fn_id, obj)| obj.source.as_ref().map(|source| (*fn_id, source.clone())))
            .collect()
    }
}
