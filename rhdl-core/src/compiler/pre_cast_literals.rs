use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;

use crate::rhif::{
    spec::{OpCode, Slot},
    Object,
};

use super::{pass::Pass, utils::remap_slots};

#[derive(Default, Debug, Clone)]
pub struct PreCastLiterals {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CastCandidate {
    slot: Slot,
    len: usize,
    signed: bool,
}

impl Pass for PreCastLiterals {
    fn name(&self) -> &'static str {
        "pre_cast_literals"
    }
    fn description(&self) -> &'static str {
        "Pre-cast literals to the requested length"
    }
    fn run(mut input: Object) -> Result<Object> {
        // Collect a candidate list of literals to cast
        let mut candidates: HashSet<CastCandidate> = Default::default();
        let mut use_count: HashMap<Slot, usize> = Default::default();
        for op in input.ops.iter_mut() {
            match op {
                OpCode::AsBits(cast) => {
                    if let Slot::Literal(ndx) = cast.arg {
                        candidates.insert(CastCandidate {
                            slot: cast.arg,
                            len: cast.len,
                            signed: false,
                        });
                    }
                }
                OpCode::AsSigned(cast) => {
                    if let Slot::Literal(ndx) = cast.arg {
                        candidates.insert(CastCandidate {
                            slot: cast.arg,
                            len: cast.len,
                            signed: true,
                        });
                    }
                }
                _ => {}
            }
            remap_slots(op.clone(), |slot| {
                *use_count.entry(slot).or_default() += 1;
                slot
            });
        }
        // Check that each candidate is referenced exactly once
        let candidates: HashMap<Slot, CastCandidate> = candidates
            .into_iter()
            .filter(|candidate| use_count.get(&candidate.slot) == Some(&1))
            .map(|candidate| (candidate.slot, candidate))
            .collect();
        // Because each literal is used exactly once, we can safely cast them
        input.literals = input
            .literals
            .into_iter()
            .map(|(slot, v)| {
                if let Some(candidate) = candidates.get(&slot) {
                    if candidate.signed {
                        v.signed_cast(candidate.len)
                    } else {
                        v.unsigned_cast(candidate.len)
                    }
                } else {
                    Ok(v)
                }
                .map(|res| (slot, res))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        input.kind = input
            .kind
            .into_iter()
            .map(|(k, v)| {
                if let Some(val) = input.literals.get(&k) {
                    (k, val.kind.clone())
                } else {
                    (k, v)
                }
            })
            .collect();

        Ok(input)
    }
}
