use std::collections::{HashMap, HashSet};

use crate::{
    object::Object,
    rhif::{BlockId, CaseArgument, OpCode, Slot},
};

use anyhow::{bail, Result};

#[derive(Default, Debug, Clone)]
struct InitSet {
    set: HashSet<Slot>,
    alias: HashMap<Slot, Slot>,
}

impl InitSet {
    fn read_all(&self, slots: &[Slot]) -> Result<()> {
        for slot in slots {
            self.read(slot)?;
        }
        Ok(())
    }
    fn alias(&mut self, pointer: &Slot, target: &Slot) -> Result<()> {
        if let Some(prev) = self.alias.insert(*pointer, *target) {
            bail!("{} is already aliased to {}", pointer, prev);
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
        if let Some(alias) = self.alias.get(slot).cloned() {
            return self.write(&alias);
        }
        match slot {
            Slot::Empty => {}
            Slot::Literal(ndx) => {
                bail!("Cannot write to literal {}", ndx);
            }
            Slot::Register(_) => {
                self.set.insert(*slot);
            }
        }
        Ok(())
    }
    fn intersect(&self, other: &InitSet) -> InitSet {
        Self {
            set: self.set.intersection(&other.set).cloned().collect(),
            alias: self
                .alias
                .clone()
                .into_iter()
                .chain(other.alias.clone())
                .collect(),
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
            OpCode::Binary {
                op: _,
                lhs,
                arg1,
                arg2,
            } => {
                init_set.read(arg1)?;
                init_set.read(arg2)?;
                init_set.write(lhs)?;
            }
            OpCode::Unary { op, lhs, arg1 } => {
                init_set.read(arg1)?;
                init_set.write(lhs)?;
            }
            OpCode::Array { lhs, elements }
            | OpCode::Tuple {
                lhs,
                fields: elements,
            } => {
                init_set.read_all(elements)?;
                init_set.write(lhs)?;
            }
            OpCode::If {
                lhs,
                cond,
                then_branch,
                else_branch,
            } => {
                init_set.read(cond)?;
                let base_set = init_set.clone();
                init_set = check_flow(obj, *then_branch, base_set.clone())?;
                init_set = init_set.intersect(&check_flow(obj, *else_branch, base_set.clone())?);
                init_set.write(lhs)?;
            }
            OpCode::Index { lhs, arg, index } => {
                init_set.read(arg)?;
                init_set.read(index)?;
                init_set.write(lhs)?;
            }
            OpCode::Field {
                lhs,
                arg,
                member: _,
            } => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
            OpCode::Ref { lhs, arg }
            | OpCode::FieldRef {
                lhs,
                arg,
                member: _,
            }
            | OpCode::IndexRef { lhs, arg, index: _ } => {
                init_set.alias(lhs, arg)?;
            }
            OpCode::Assign { lhs, rhs } | OpCode::Copy { lhs, rhs } => {
                init_set.read(rhs)?;
                init_set.write(lhs)?;
            }
            OpCode::Struct {
                lhs,
                path: _,
                fields,
                rest,
            } => {
                for field in fields {
                    init_set.read(&field.value)?;
                }
                if let Some(rest) = rest {
                    init_set.read(rest)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Enum {
                lhs,
                path: _,
                discriminant,
                fields,
            } => {
                init_set.read(discriminant)?;
                for field in fields {
                    init_set.read(&field.value)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Repeat { lhs, value, len } => {
                init_set.read(len)?;
                init_set.read(value)?;
                init_set.write(lhs)?;
            }
            OpCode::Comment(_) => {}
            OpCode::Return { result } => {
                if let Some(result) = result {
                    init_set.read(result)?;
                    init_set.write(&obj.return_slot)?;
                }
            }
            OpCode::Block(id) => {
                init_set = check_flow(obj, *id, init_set)?;
            }
            OpCode::Payload {
                lhs,
                arg,
                discriminant,
            } => {
                init_set.read(arg)?;
                init_set.read(discriminant)?;
                init_set.write(lhs)?;
            }
            OpCode::Case {
                discriminant,
                table,
            } => {
                init_set.read(discriminant)?;
                let base_set = init_set.clone();
                let mut first_branch = true;
                for entry in table {
                    if let CaseArgument::Literal(lit) = &entry.0 {
                        init_set.read(lit)?;
                    }
                    if first_branch {
                        init_set = check_flow(obj, entry.1, init_set.clone())?;
                        first_branch = false;
                    } else {
                        init_set = init_set.intersect(&check_flow(obj, entry.1, base_set.clone())?);
                    }
                }
            }
            OpCode::Discriminant { lhs, arg } => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
            OpCode::Exec { lhs, id: _, args } => {
                init_set.read_all(args)?;
                init_set.write(lhs)?;
            }
            OpCode::AsBits { lhs, arg, len } | OpCode::AsSigned { lhs, arg, len } => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
        }
    }
    Ok(init_set)
}
