use fnv::{FnvHashMap, FnvHashSet};

use crate::schematic::schematic_impl::{PinIx, Schematic};

use super::source_pool::{SharedSourcePool, SourcePool};

pub struct Index {
    pub forward: IndexType,
    pub reverse: IndexType,
}

type IndexType = FnvHashMap<PinIx, FnvHashSet<PinIx>>;

fn make_index(schematic: &Schematic) -> Index {
    let mut forward = IndexType::default();
    let mut reverse = IndexType::default();
    for wire in &schematic.wires {
        forward.entry(wire.source).or_default().insert(wire.dest);
        reverse.entry(wire.dest).or_default().insert(wire.source);
    }
    Index { forward, reverse }
}

pub struct IndexedSchematic {
    pub schematic: Schematic,
    pub index: Index,
    pub pool: SharedSourcePool,
}

impl From<Schematic> for IndexedSchematic {
    fn from(schematic: Schematic) -> Self {
        let schematic = schematic.inlined();
        let index = make_index(&schematic);
        let pool = SharedSourcePool::from(SourcePool::new(schematic.source.clone()));
        IndexedSchematic {
            schematic,
            index,
            pool,
        }
    }
}
