use std::collections::{HashMap, HashSet};

use crate::{
    common::symtab::LiteralId,
    error::RHDLError,
    rhif::{
        Object,
        spec::{OpCode, Slot, SlotKind},
        visit::visit_slots,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct PreCastLiterals {}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CastCandidate {
    id: LiteralId<SlotKind>,
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
                        if let Some(id) = cast.arg.lit() {
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
                        if let Some(id) = cast.arg.lit() {
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
            visit_slots(&lop.op, |sense, slot| {
                if sense.is_read() {
                    *use_count.entry(*slot).or_default() += 1;
                }
            });
        }
        // Check that each candidate is referenced exactly once
        let candidates: HashMap<LiteralId<_>, CastCandidate> = candidates
            .into_iter()
            .filter(|candidate| use_count.get(&Slot::Literal(candidate.id)) == Some(&1))
            .map(|candidate| (candidate.id, candidate))
            .collect();
        // Because each literal is used exactly once, we can safely cast them
        for (slot, (value, _loc)) in input.symtab.iter_lit_mut() {
            if let Some(candidate) = candidates.get(&slot) {
                if candidate.signed {
                    *value = value.signed_cast(candidate.len)?;
                } else {
                    *value = value.unsigned_cast(candidate.len)?;
                }
            }
        }
        Ok(input)
    }
}
