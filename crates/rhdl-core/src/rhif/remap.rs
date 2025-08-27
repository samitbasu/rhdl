use crate::rhif::{
    spec::{OpCode, Slot},
    visit::visit_slots_mut,
};

pub fn remap_slots<F: FnMut(Slot) -> Slot>(mut op: OpCode, mut f: F) -> OpCode {
    visit_slots_mut(&mut op, |_sense, slot| *slot = f(*slot));
    op
}

pub fn rename_read_register(op: OpCode, old: Slot, new: Slot) -> OpCode {
    remap_slots(op, |slot| if slot == old { new } else { slot })
}
