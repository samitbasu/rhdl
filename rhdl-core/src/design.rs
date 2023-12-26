use std::collections::HashMap;

use crate::object::Object;

#[derive(Clone, Debug)]
pub struct Design {
    pub objects: HashMap<String, Object>,
    pub top: String,
}

impl std::fmt::Display for Design {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Design {}", self.top)?;
        for (name, obj) in &self.objects {
            write!(f, "\n  Object {}", obj.name)?;
        }
        Ok(())
    }
}
