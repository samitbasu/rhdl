use crate::rhdl_core::rhif::Object;

use super::spec::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Read,
    Write,
}

impl Sense {
    pub fn is_read(&self) -> bool {
        matches!(self, Sense::Read)
    }
    pub fn is_write(&self) -> bool {
        matches!(self, Sense::Write)
    }
}

pub fn visit_slots<F: FnMut(Sense, &Slot)>(op: &OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Binary(Binary {
            op: __,
            lhs,
            arg1,
            arg2,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
            f(Sense::Read, arg2);
        }
        OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
        }
        OpCode::Select(Select {
            lhs,
            cond,
            true_value,
            false_value,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, cond);
            f(Sense::Read, true_value);
            f(Sense::Read, false_value);
        }
        OpCode::Index(Index { lhs, arg, path }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
            for dyn_slot in path.dynamic_slots() {
                f(Sense::Read, dyn_slot);
            }
        }
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, rhs);
        }
        OpCode::Splice(Splice {
            lhs,
            orig,
            path,
            subst,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, orig);
            f(Sense::Read, subst);
            for dyn_slot in path.dynamic_slots() {
                f(Sense::Read, dyn_slot);
            }
        }
        OpCode::Repeat(Repeat { lhs, value, len: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, value);
        }
        OpCode::Struct(Struct {
            lhs,
            fields,
            rest,
            template: _,
        }) => {
            f(Sense::Write, lhs);
            for val in fields {
                f(Sense::Read, &val.value);
            }
            rest.as_ref().map(|x| f(Sense::Read, x));
        }
        OpCode::Tuple(Tuple { lhs, fields }) => {
            f(Sense::Write, lhs);
            for field in fields {
                f(Sense::Read, field);
            }
        }
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, discriminant);
            for (arg, val) in table {
                f(Sense::Read, val);
                if let CaseArgument::Slot(slot) = arg {
                    f(Sense::Read, slot);
                }
            }
        }
        OpCode::Exec(Exec { lhs, id: _, args }) => {
            f(Sense::Write, lhs);
            for arg in args {
                f(Sense::Read, arg);
            }
        }
        OpCode::Array(Array { lhs, elements }) => {
            f(Sense::Write, lhs);
            for arg in elements {
                f(Sense::Read, arg);
            }
        }
        OpCode::Enum(Enum {
            lhs,
            fields,
            template: _,
        }) => {
            f(Sense::Write, lhs);
            for field in fields {
                f(Sense::Read, &field.value);
            }
        }
        OpCode::AsBits(Cast { lhs, arg, len: _ })
        | OpCode::AsSigned(Cast { lhs, arg, len: _ })
        | OpCode::Resize(Cast { lhs, arg, len: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Retime(Retime { lhs, arg, color: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Wrap(Wrap {
            op: __,
            lhs,
            arg,
            kind: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Comment(_) => {}
    }
}

pub fn visit_slots_mut<F: FnMut(Sense, &mut Slot)>(op: &mut OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Binary(Binary {
            op: __,
            lhs,
            arg1,
            arg2,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
            f(Sense::Read, arg2);
        }
        OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
        }
        OpCode::Select(Select {
            lhs,
            cond,
            true_value,
            false_value,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, cond);
            f(Sense::Read, true_value);
            f(Sense::Read, false_value);
        }
        OpCode::Index(Index { lhs, arg, path }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
            for dyn_slot in path.dynamic_slots_mut() {
                f(Sense::Read, dyn_slot);
            }
        }
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, rhs);
        }
        OpCode::Splice(Splice {
            lhs,
            orig,
            path,
            subst,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, orig);
            f(Sense::Read, subst);
            for dyn_slot in path.dynamic_slots_mut() {
                f(Sense::Read, dyn_slot);
            }
        }
        OpCode::Repeat(Repeat { lhs, value, len: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, value);
        }
        OpCode::Struct(Struct {
            lhs,
            fields,
            rest,
            template: _,
        }) => {
            f(Sense::Write, lhs);
            for val in fields {
                f(Sense::Read, &mut val.value);
            }
            rest.as_mut().map(|x| f(Sense::Read, x));
        }
        OpCode::Tuple(Tuple { lhs, fields }) => {
            f(Sense::Write, lhs);
            for field in fields {
                f(Sense::Read, field);
            }
        }
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, discriminant);
            for (arg, val) in table {
                f(Sense::Read, val);
                if let CaseArgument::Slot(slot) = arg {
                    f(Sense::Read, slot);
                }
            }
        }
        OpCode::Exec(Exec { lhs, id: _, args }) => {
            f(Sense::Write, lhs);
            for arg in args {
                f(Sense::Read, arg);
            }
        }
        OpCode::Array(Array { lhs, elements }) => {
            f(Sense::Write, lhs);
            for arg in elements {
                f(Sense::Read, arg);
            }
        }
        OpCode::Enum(Enum {
            lhs,
            fields,
            template: _,
        }) => {
            f(Sense::Write, lhs);
            for field in fields {
                f(Sense::Read, &mut field.value);
            }
        }
        OpCode::AsBits(Cast { lhs, arg, len: _ })
        | OpCode::AsSigned(Cast { lhs, arg, len: _ })
        | OpCode::Resize(Cast { lhs, arg, len: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Retime(Retime { lhs, arg, color: _ }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Wrap(Wrap {
            op: __,
            lhs,
            arg,
            kind: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Comment(_) => {}
    }
}

pub fn visit_object_slots<F: FnMut(Sense, &Slot)>(object: &Object, mut f: F) {
    for arg in &object.arguments {
        f(Sense::Write, &Slot::Register(*arg));
    }
    for lop in &object.ops {
        visit_slots(&lop.op, &mut f);
    }
    f(Sense::Read, &object.return_slot);
}

pub fn visit_object_slots_mut<F: FnMut(Sense, &mut Slot)>(object: &mut Object, mut f: F) {
    for arg in object.arguments.iter_mut() {
        let mut slot = Slot::Register(*arg);
        f(Sense::Write, &mut slot);
        *arg = slot
            .reg()
            .expect("Argument slots must remain register.  Do not mutate them into literals!");
    }
    for lop in object.ops.iter_mut() {
        visit_slots_mut(&mut lop.op, &mut f);
    }
    f(Sense::Read, &mut object.return_slot);
}
