use crate::{ast::FunctionId, object::Object};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Design {
    pub objects: HashMap<FunctionId, Object>,
    pub top: FunctionId,
}

impl std::fmt::Display for Design {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Design {}", self.top)?;
        for obj in self.objects.values() {
            write!(f, "\n  Object {}", obj.name)?;
        }
        Ok(())
    }
}

impl Design {
    pub fn func_name(&self, fn_id: FunctionId) -> Result<String> {
        let obj = self
            .objects
            .get(&fn_id)
            .ok_or(anyhow::anyhow!("Function {fn_id} not found"))?;
        Ok(format!("{}_{:x}", obj.name, fn_id))
    }
}
