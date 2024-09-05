use std::collections::HashMap;

use crate::{ast::source_location::SourceLocation, rtl::object::RegisterKind};

use super::rtl_components::ComponentKind;

#[derive(Clone, Debug, Default)]
pub struct PartsDB {
    pins: Vec<Pin>,
    components: Vec<ComponentKind>,
    component_locations: HashMap<ComponentIx, SourceLocation>,
}

impl PartsDB {
    pub fn new_pin(&mut self, kind: RegisterKind, name: &str) -> PinIx {
        let len = self.pins.len();
        self.pins.push(Pin {
            kind,
            name: name.into(),
        });
        PinIx(len)
    }
    pub fn new_component(
        &mut self,
        component: ComponentKind,
        location: SourceLocation,
    ) -> ComponentIx {
        let len = self.components.len();
        self.components.push(component);
        self.component_locations.insert(ComponentIx(len), location);
        ComponentIx(len)
    }
}

impl std::ops::Index<PinIx> for PartsDB {
    type Output = Pin;

    fn index(&self, index: PinIx) -> &Self::Output {
        &self.pins[index.0]
    }
}

impl std::ops::IndexMut<PinIx> for PartsDB {
    fn index_mut(&mut self, index: PinIx) -> &mut Self::Output {
        &mut self.pins[index.0]
    }
}

impl std::ops::Index<ComponentIx> for PartsDB {
    type Output = ComponentKind;

    fn index(&self, index: ComponentIx) -> &Self::Output {
        &self.components[index.0]
    }
}

impl std::ops::IndexMut<ComponentIx> for PartsDB {
    fn index_mut(&mut self, index: ComponentIx) -> &mut Self::Output {
        &mut self.components[index.0]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentIx(usize);

impl std::fmt::Debug for ComponentIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinIx(usize);

impl std::fmt::Debug for PinIx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "p{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub kind: RegisterKind,
    pub name: String,
}
