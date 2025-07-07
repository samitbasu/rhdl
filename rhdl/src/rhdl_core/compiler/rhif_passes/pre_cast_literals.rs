use std::collections::{BTreeMap, HashMap, HashSet};

use crate::rhdl_core::{
    common::symtab::LiteralId,
    error::RHDLError,
    rhif::{
        Object,
        remap::remap_slots,
        spec::{OpCode, Slot},
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PreCastLiterals {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CastCandidate {
    id: LiteralId,
    len: usize,
    signed: bool,
}

impl Pass for PreCastLiterals {
    fn description() -> &'static str {
        "Pre-cast literals to final type"
    }
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // Collect a candidate list of literals to cast
        let mut candidates: HashSet<CastCandidate> = Default::default();
        let mut use_count: HashMap<Slot, usize> = Default::default();
        for lop in input.ops.iter_mut() {
            match &lop.op {
                OpCode::AsBits(cast) => {
                    if let Some(len) = cast.len {
                        if let Ok(id) = cast.arg.as_literal() {
                            candidates.insert(CastCandidate {
                                id,
                                len,
                                signed: false,
                            });
                        }
                    }
                }
                OpCode::AsSigned(cast) => {
                    if let Some(len) = cast.len {
                        if let Ok(id) = cast.arg.as_literal() {
                            candidates.insert(CastCandidate {
                                id,
                                len,
                                signed: true,
                            });
                        }
                    }
                }
                _ => {}
            }
            remap_slots(lop.op.clone(), |slot| {
                *use_count.entry(slot).or_default() += 1;
                slot
            });
        }
        // Check that each candidate is referenced exactly once
        let candidates: HashMap<LiteralId, CastCandidate> = candidates
            .into_iter()
            .filter(|candidate| use_count.get(&Slot::Literal(candidate.id)) == Some(&1))
            .map(|candidate| (candidate.id, candidate))
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
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(input)
    }
}
