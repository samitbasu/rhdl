use anyhow::{bail, Result};
use fnv::{FnvHashMap, FnvHashSet};

use crate::schematic::schematic_impl::{PinIx, PinPath, Schematic};

use super::{
    downstream::follow_pin_downstream,
    source_pool::{SharedSourcePool, SourcePool},
};

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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClockId(usize);

impl From<usize> for ClockId {
    fn from(val: usize) -> Self {
        ClockId(val)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Timing {
    Constant,
    Asynchronous,
    Synchronous(ClockId),
}

pub struct IndexedSchematic {
    pub schematic: Schematic,
    pub index: Index,
    pub pool: SharedSourcePool,
    pub timing: FnvHashMap<PinPath, Timing>,
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
            timing: FnvHashMap::default(),
        }
    }
}

impl IndexedSchematic {
    pub fn set_timing(&mut self, pin_path: PinPath, timing: Timing) -> Result<()> {
        match self.timing.insert(pin_path.clone(), timing) {
            None => (),
            Some(existing) => {
                if existing != timing {
                    bail!(
                        "Pin path {:?} already has a different timing source",
                        pin_path
                    );
                }
            }
        }
        Ok(())
    }
    pub fn add_synchronous_source(&mut self, pin_path: PinPath, id: ClockId) -> Result<()> {
        self.set_timing(pin_path.clone(), Timing::Synchronous(id))?;
        let trace = follow_pin_downstream(self, pin_path)?;
        for pin_path in trace.all_pin_paths() {
            self.set_timing(pin_path, Timing::Synchronous(id))?;
        }
        Ok(())
    }
    pub fn add_asynchronous_source(&mut self, pin_path: PinPath) -> Result<()> {
        self.set_timing(pin_path.clone(), Timing::Asynchronous)?;
        let trace = follow_pin_downstream(self, pin_path)?;
        for pin_path in trace.all_pin_paths() {
            self.set_timing(pin_path, Timing::Asynchronous)?;
        }
        Ok(())
    }
    pub fn add_constant_source(&mut self, pin_path: PinPath) -> Result<()> {
        self.set_timing(pin_path.clone(), Timing::Constant)?;
        let trace = follow_pin_downstream(self, pin_path)?;
        for pin_path in trace.all_pin_paths() {
            if !self.timing.contains_key(&pin_path) {
                self.set_timing(pin_path, Timing::Constant)?;
            }
        }
        Ok(())
    }
    pub fn is_synchronous(&self, pin_path: &PinPath) -> bool {
        matches!(self.timing.get(pin_path), Some(Timing::Synchronous(_)))
    }
    pub fn clock_id(&self, sink: &PinPath) -> Option<ClockId> {
        if let Some(Timing::Synchronous(id)) = self.timing.get(sink) {
            Some(*id)
        } else {
            None
        }
    }
}
