use std::collections::HashSet;

use crate::{
    rhif::spec::{
        Array, Assign, Binary, BlockId, Case, Cast, Discriminant, Enum, Exec, Index, OpCode,
        Repeat, Select, Slot, Splice, Struct, Tuple, Unary,
    },
    rhif::Object,
};

use anyhow::{bail, Result};

#[derive(Default, Debug, Clone)]
struct InitSet {
    set: HashSet<Slot>,
}

impl InitSet {
    fn read_all(&self, slots: &[Slot]) -> Result<()> {
        for slot in slots {
            self.read(slot)?;
        }
        Ok(())
    }
    fn read(&self, slot: &Slot) -> Result<()> {
        match slot {
            Slot::Empty | Slot::Literal(_) => {}
            Slot::Register(_) => {
                if !self.set.contains(slot) {
                    bail!("{} is not initialized", slot);
                }
            }
        }
        Ok(())
    }
    fn write(&mut self, slot: &Slot) -> Result<()> {
        match slot {
            Slot::Empty => {}
            Slot::Literal(ndx) => {
                bail!("Cannot write to literal {}", ndx);
            }
            Slot::Register(_) => {
                if self.set.contains(slot) {
                    bail!("{} is already initialized", slot);
                }
                self.set.insert(*slot);
            }
        }
        Ok(())
    }
    fn intersect(&self, other: &InitSet) -> InitSet {
        Self {
            set: self.set.intersection(&other.set).cloned().collect(),
        }
    }
}

pub fn check_rhif_flow(obj: &Object) -> Result<()> {
    let mut init_set = InitSet::default();
    for arg in &obj.arguments {
        init_set.write(arg)?;
    }
    check_flow(obj, obj.main_block, init_set)?;
    Ok(())
}

fn check_flow(obj: &Object, block: BlockId, mut init_set: InitSet) -> Result<InitSet> {
    for op in &obj.blocks[block.0].ops {
        eprintln!("Check flow for {}", op);
        match op {
            OpCode::Binary(Binary {
                op: _,
                lhs,
                arg1,
                arg2,
            }) => {
                init_set.read(arg1)?;
                init_set.read(arg2)?;
                init_set.write(lhs)?;
            }
            OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
                init_set.read(arg1)?;
                init_set.write(lhs)?;
            }
            OpCode::Array(Array { lhs, elements })
            | OpCode::Tuple(Tuple {
                lhs,
                fields: elements,
            }) => {
                init_set.read_all(elements)?;
                init_set.write(lhs)?;
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                init_set.read(cond)?;
                init_set.read(true_value)?;
                init_set.read(false_value)?;
                init_set.write(lhs)?;
            }
            OpCode::Index(Index { lhs, arg, path }) => {
                init_set.read(arg)?;
                for slot in path.dynamic_slots() {
                    init_set.read(slot)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Assign(Assign { lhs, rhs }) => {
                init_set.read(rhs)?;
                init_set.write(lhs)?;
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                path,
                subst,
            }) => {
                init_set.read(orig)?;
                init_set.read(subst)?;
                for slot in path.dynamic_slots() {
                    init_set.read(slot)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Struct(Struct {
                lhs,
                fields,
                rest,
                template: _,
            }) => {
                for field in fields {
                    init_set.read(&field.value)?;
                }
                if let Some(rest) = rest {
                    init_set.read(rest)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Enum(Enum {
                lhs,
                fields,
                template: _,
            }) => {
                for field in fields {
                    init_set.read(&field.value)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Repeat(Repeat { lhs, value, len: _ }) => {
                init_set.read(value)?;
                init_set.write(lhs)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Block(id) => {
                init_set = check_flow(obj, *id, init_set)?;
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                init_set.read(discriminant)?;
                for (argument, slot) in table {
                    init_set.read(slot)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Discriminant(Discriminant { lhs, arg }) => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
            OpCode::Exec(Exec { lhs, id: _, args }) => {
                init_set.read_all(args)?;
                init_set.write(lhs)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len: _ })
            | OpCode::AsSigned(Cast { lhs, arg, len: _ }) => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
        }
    }
    Ok(init_set)
}
