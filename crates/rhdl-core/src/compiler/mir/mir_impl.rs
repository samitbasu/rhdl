use log::debug;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
};

use crate::{
    Kind,
    ast::{
        KernelFlags,
        ast_impl::{ExprLit, FunctionId},
        source::source_location::SourceLocation,
    },
    common::symtab::SymbolTable,
    rhif::{
        Object,
        object::{LocatedOpCode, SymbolMap},
        spec::{FuncId, OpCode, Slot, SlotKind},
    },
};

#[derive(Clone, Debug, PartialEq, Hash, Eq)]
pub struct TypeEquivalence {
    pub lhs: Slot,
    pub rhs: Slot,
    pub loc: SourceLocation,
}

pub struct Mir {
    pub symbols: SymbolMap,
    pub ops: Vec<LocatedOpCode>,
    pub symtab: SymbolTable<ExprLit, Option<String>, SourceLocation, SlotKind>,
    pub ty: BTreeMap<Slot, Kind>,
    pub ty_equate: HashSet<TypeEquivalence>,
    pub stash: BTreeMap<FuncId, Box<Object>>,
    pub return_slot: Slot,
    pub arguments: Vec<Slot>,
    pub fn_id: FunctionId,
    pub name: String,
    pub flags: Vec<KernelFlags>,
}

impl Mir {
    pub fn build_slot_equivalence_map(&self) -> HashMap<Slot, Slot> {
        self.ops
            .iter()
            .filter_map(|op| {
                if let OpCode::Assign(assign) = &op.op {
                    Some((assign.lhs, assign.rhs))
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn slot_span(&self, slot: Slot) -> Range<usize> {
        self.symbols.span(self.symtab[slot])
    }
    pub fn find_root_for_slot(&self, context: SourceLocation, slot: Slot) -> Slot {
        let context_span = self.symbols.span(context);
        debug!("Context span: {:?}", context_span);
        let eq_map = self.build_slot_equivalence_map();
        let mut slot = slot;
        debug!("Initial slot: {:?}", slot);
        debug!("Initial span: {:?}", self.slot_span(slot));
        while let Some(&next) = eq_map.get(&slot) {
            debug!("Next slot: {:?}", next);
            let next_span = self.slot_span(next);
            debug!("Next span: {:?}", next_span);
            if context_span.contains(&next_span.start)
                && context_span.contains(&next_span.end.saturating_sub(1))
            {
                slot = next;
            } else {
                break;
            }
        }
        debug!("Final slot: {:?}", slot);
        slot
    }
}
