use crate::rhif::spec::{
    Array, Assign, Binary, Case, CaseArgument, Cast, Enum, Exec, FieldValue, Index, OpCode, Repeat,
    Retime, Select, Slot, Splice, Struct, Tuple, Unary,
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
                .map(|(arg, slot)| {
                    let arg = match arg {
                        CaseArgument::Slot(slot) => CaseArgument::Slot(f(slot)),
                        _ => arg,
                    };
                    (arg, f(slot))
                })
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
        OpCode::Retime(Retime { lhs, arg, color }) => OpCode::Retime(Retime {
            lhs: f(lhs),
            arg: f(arg),
            color,
        }),
        _ => op,
    }
}

pub fn rename_read_register(op: OpCode, old: Slot, new: Slot) -> OpCode {
    remap_slots(op, |slot| if slot == old { new } else { slot })
}
