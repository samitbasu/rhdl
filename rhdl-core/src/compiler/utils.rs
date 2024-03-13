use crate::rhif::spec::{
    Array, Assign, Binary, Case, Cast, Discriminant, Enum, Exec, FieldValue, Index, OpCode, Repeat,
    Select, Slot, Splice, Struct, Tuple, Unary,
};

pub fn remap_slots<F: FnMut(Slot) -> Slot>(op: OpCode, mut f: F) -> OpCode {
    match op {
        OpCode::Binary(Binary {
            op,
            lhs,
            arg1,
            arg2,
        }) => OpCode::Binary(Binary {
            op,
            lhs: f(lhs),
            arg1: f(arg1),
            arg2: f(arg2),
        }),
        OpCode::Unary(Unary { op, lhs, arg1 }) => OpCode::Unary(Unary {
            op,
            lhs: f(lhs),
            arg1: f(arg1),
        }),
        OpCode::Select(Select {
            lhs,
            cond,
            true_value,
            false_value,
        }) => OpCode::Select(Select {
            lhs: f(lhs),
            cond: f(cond),
            true_value: f(true_value),
            false_value: f(false_value),
        }),
        OpCode::Index(Index { lhs, arg, path }) => OpCode::Index(Index {
            lhs: f(lhs),
            arg: f(arg),
            path: path.remap_slots(f),
        }),
        OpCode::Assign(Assign { lhs, rhs }) => OpCode::Assign(Assign {
            lhs: f(lhs),
            rhs: f(rhs),
        }),
        OpCode::Splice(Splice {
            lhs,
            orig,
            path,
            subst,
        }) => OpCode::Splice(Splice {
            lhs: f(lhs),
            orig: f(orig),
            subst: f(subst),
            path: path.remap_slots(f),
        }),
        OpCode::Repeat(Repeat { lhs, value, len }) => OpCode::Repeat(Repeat {
            lhs: f(lhs),
            value: f(value),
            len,
        }),
        OpCode::Struct(Struct {
            lhs,
            fields,
            rest,
            template,
        }) => OpCode::Struct(Struct {
            lhs: f(lhs),
            fields: fields
                .into_iter()
                .map(|v| FieldValue {
                    member: v.member,
                    value: f(v.value),
                })
                .collect(),
            rest: rest.map(f),
            template,
        }),
        OpCode::Tuple(Tuple { lhs, fields }) => OpCode::Tuple(Tuple {
            lhs: f(lhs),
            fields: fields.into_iter().map(f).collect(),
        }),
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => OpCode::Case(Case {
            lhs: f(lhs),
            discriminant: f(discriminant),
            table: table
                .into_iter()
                .map(|(arg, slot)| (arg, f(slot)))
                .collect(),
        }),
        OpCode::Exec(Exec { lhs, id, args }) => OpCode::Exec(Exec {
            lhs: f(lhs),
            id,
            args: args.into_iter().map(f).collect(),
        }),
        OpCode::Array(Array { lhs, elements }) => OpCode::Array(Array {
            lhs: f(lhs),
            elements: elements.into_iter().map(f).collect(),
        }),
        OpCode::Discriminant(Discriminant { lhs, arg }) => OpCode::Discriminant(Discriminant {
            lhs: f(lhs),
            arg: f(arg),
        }),
        OpCode::Enum(Enum {
            lhs,
            fields,
            template,
        }) => OpCode::Enum(Enum {
            lhs: f(lhs),
            fields: fields
                .into_iter()
                .map(|v| FieldValue {
                    member: v.member,
                    value: f(v.value),
                })
                .collect(),
            template,
        }),
        OpCode::AsBits(Cast { lhs, arg, len }) => OpCode::AsBits(Cast {
            lhs: f(lhs),
            arg: f(arg),
            len,
        }),
        OpCode::AsSigned(Cast { lhs, arg, len }) => OpCode::AsSigned(Cast {
            lhs: f(lhs),
            arg: f(arg),
            len,
        }),
        _ => op,
    }
}

pub fn rename_read_register(op: OpCode, old: Slot, new: Slot) -> OpCode {
    remap_slots(op, |slot| if slot == old { new } else { slot })
}
/*
pub(crate) fn rename_read_register(op: OpCode, old: Slot, new: Slot) -> OpCode {
    match op {
        OpCode::Binary(Binary {
            op,
            lhs,
            arg1,
            arg2,
        }) => OpCode::Binary(Binary {
            op,
            lhs,
            arg1: arg1.rename(old, new),
            arg2: arg2.rename(old, new),
        }),
        OpCode::Unary(Unary { op, lhs, arg1 }) => OpCode::Unary(Unary {
            op,
            lhs,
            arg1: arg1.rename(old, new),
        }),
        OpCode::Select(Select {
            lhs,
            cond,
            true_value,
            false_value,
        }) => OpCode::Select(Select {
            lhs,
            cond: cond.rename(old, new),
            true_value: true_value.rename(old, new),
            false_value: false_value.rename(old, new),
        }),
        OpCode::Index(Index { lhs, arg, path }) => OpCode::Index(Index {
            lhs,
            arg: arg.rename(old, new),
            path: path.rename_dyn_slots(old, new),
        }),
        OpCode::Assign(Assign { lhs, rhs }) => {
            let new_rhs = rhs.rename(old, new);
            if new_rhs == lhs {
                OpCode::Noop
            } else {
                OpCode::Assign(Assign { lhs, rhs: new_rhs })
            }
        }
        OpCode::Splice(Splice {
            lhs,
            orig,
            path,
            subst,
        }) => OpCode::Splice(Splice {
            lhs,
            orig: orig.rename(old, new),
            path: path.rename_dyn_slots(old, new),
            subst: subst.rename(old, new),
        }),
        OpCode::Repeat(Repeat { lhs, value, len }) => OpCode::Repeat(Repeat {
            lhs,
            value: value.rename(old, new),
            len,
        }),
        OpCode::Struct(Struct {
            lhs,
            fields,
            rest,
            template,
        }) => OpCode::Struct(Struct {
            lhs,
            fields: fields
                .into_iter()
                .map(|f| FieldValue {
                    member: f.member,
                    value: f.value.rename(old, new),
                })
                .collect(),
            rest: rest.map(|r| r.rename(old, new)),
            template,
        }),
        OpCode::Tuple(Tuple { lhs, fields }) => OpCode::Tuple(Tuple {
            lhs,
            fields: fields.into_iter().map(|f| f.rename(old, new)).collect(),
        }),
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => OpCode::Case(Case {
            lhs,
            discriminant: discriminant.rename(old, new),
            table: table
                .into_iter()
                .map(|(arg, slot)| (arg, slot.rename(old, new)))
                .collect(),
        }),
        OpCode::Exec(Exec { lhs, id, args }) => OpCode::Exec(Exec {
            lhs,
            id,
            args: args.into_iter().map(|x| x.rename(old, new)).collect(),
        }),
        OpCode::Array(Array { lhs, elements }) => OpCode::Array(Array {
            lhs,
            elements: elements.into_iter().map(|x| x.rename(old, new)).collect(),
        }),
        OpCode::Discriminant(Discriminant { lhs, arg }) => OpCode::Discriminant(Discriminant {
            lhs,
            arg: arg.rename(old, new),
        }),
        OpCode::Enum(Enum {
            lhs,
            fields,
            template,
        }) => OpCode::Enum(Enum {
            lhs,
            fields: fields
                .into_iter()
                .map(|f| FieldValue {
                    member: f.member,
                    value: f.value.rename(old, new),
                })
                .collect(),
            template,
        }),
        OpCode::AsBits(Cast { lhs, arg, len }) => OpCode::AsBits(Cast {
            lhs,
            arg: arg.rename(old, new),
            len,
        }),
        OpCode::AsSigned(Cast { lhs, arg, len }) => OpCode::AsSigned(Cast {
            lhs,
            arg: arg.rename(old, new),
            len,
        }),
        _ => op,
    }
}
*/
