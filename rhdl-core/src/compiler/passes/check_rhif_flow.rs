use std::collections::HashSet;

use super::pass::Pass;
use crate::{
    compiler::mir::error::ICE,
    error::RHDLError,
    rhif::{
        spec::{
            Array, Assign, Binary, Case, Cast, Enum, Exec, Index, OpCode, Repeat, Retime, Select,
            Slot, Splice, Struct, Tuple, Unary,
        },
        Object,
    },
};

#[derive(Debug)]
struct InitSet<'a> {
    obj: &'a Object,
    set: HashSet<Slot>,
}

pub struct DataFlowCheckPass;

impl Pass for DataFlowCheckPass {
    fn name() -> &'static str {
        "check_rhif_flow"
    }
    fn description() -> &'static str {
        "Check that all registers are initialized before use"
    }
    fn run(input: Object) -> Result<Object, RHDLError> {
        check_rhif_flow(&input)?;
        Ok(input)
    }
}

impl<'a> InitSet<'a> {
    fn read_all(&self, slots: &[Slot]) -> Result<(), RHDLError> {
        for slot in slots {
            self.read(slot)?;
        }
        Ok(())
    }
    fn read(&self, slot: &Slot) -> Result<(), RHDLError> {
        match slot {
            Slot::Empty | Slot::Literal(_) => {}
            Slot::Register(_) => {
                if !self.set.contains(slot) {
                    return Err(DataFlowCheckPass::raise_ice(
                        self.obj,
                        ICE::SlotIsReadBeforeBeingWritten { slot: *slot },
                        self.obj.symbols.slot_map[slot].node,
                    ));
                }
            }
        }
        Ok(())
    }
    fn write(&mut self, slot: &Slot) -> Result<(), RHDLError> {
        match slot {
            Slot::Empty => {}
            Slot::Literal(ndx) => {
                return Err(DataFlowCheckPass::raise_ice(
                    &self.obj,
                    ICE::CannotWriteToLiteral { ndx: *ndx },
                    self.obj.symbols.slot_map[slot].node,
                ));
            }
            Slot::Register(_) => {
                if self.set.contains(slot) {
                    return Err(DataFlowCheckPass::raise_ice(
                        &self.obj,
                        ICE::SlotIsWrittenTwice { slot: *slot },
                        self.obj.symbols.slot_map[slot].node,
                    ));
                }
                self.set.insert(*slot);
            }
        }
        Ok(())
    }
}

fn check_rhif_flow(obj: &Object) -> Result<(), RHDLError> {
    let mut init_set = InitSet {
        obj,
        set: HashSet::new(),
    };
    for arg in &obj.arguments {
        init_set.write(arg)?;
    }
    check_flow(obj, init_set)?;
    Ok(())
}

fn check_flow<'a>(obj: &'a Object, mut init_set: InitSet<'a>) -> Result<InitSet<'a>, RHDLError> {
    for op in &obj.ops {
        eprintln!("Check flow for {:?}", op);
        match op {
            OpCode::Noop => {}
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
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                init_set.read(discriminant)?;
                for (_, slot) in table {
                    init_set.read(slot)?;
                }
                init_set.write(lhs)?;
            }
            OpCode::Exec(Exec { lhs, id: _, args }) => {
                init_set.read_all(args)?;
                init_set.write(lhs)?;
            }
            OpCode::AsBits(Cast { lhs, arg, len: _ })
            | OpCode::AsSigned(Cast { lhs, arg, len: _ })
            | OpCode::Retime(Retime { lhs, arg, color: _ }) => {
                init_set.read(arg)?;
                init_set.write(lhs)?;
            }
        }
    }
    Ok(init_set)
}
