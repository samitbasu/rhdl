use super::spec::{
    Assign, Binary, Case, CaseArgument, Cast, Concat, Index, OpCode, Operand, Select, Splice, Unary,
};

pub fn remap_operands<F: FnMut(Operand) -> Operand>(op: OpCode, mut f: F) -> OpCode {
    match op {
        OpCode::Noop => op,
        OpCode::Assign(Assign { lhs, rhs }) => OpCode::Assign(Assign {
            lhs: f(lhs),
            rhs: f(rhs),
        }),
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
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => OpCode::Case(Case {
            lhs: f(lhs),
            discriminant: f(discriminant),
            table: table
                .into_iter()
                .map(|(arg, result)| {
                    (
                        match arg {
                            CaseArgument::Literal(lit) => {
                                let fn_id = f(Operand::Literal(lit));
                                let Operand::Literal(fn_lit) = fn_id else {
                                    panic!("Expected literal, got {fn_id:?}");
                                };
                                CaseArgument::Literal(fn_lit)
                            }
                            _ => arg,
                        },
                        f(result),
                    )
                })
                .collect(),
        }),
        OpCode::Cast(Cast {
            lhs,
            arg,
            len,
            kind: signed,
        }) => OpCode::Cast(Cast {
            lhs: f(lhs),
            arg: f(arg),
            len,
            kind: signed,
        }),
        OpCode::Comment(_) => op,
        OpCode::Concat(Concat { lhs, args }) => OpCode::Concat(Concat {
            lhs: f(lhs),
            args: args.into_iter().map(f).collect(),
        }),
        OpCode::Index(Index {
            lhs,
            arg,
            bit_range,
        }) => OpCode::Index(Index {
            lhs: f(lhs),
            arg: f(arg),
            bit_range,
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
        OpCode::Splice(Splice {
            lhs,
            orig,
            bit_range,
            value,
        }) => OpCode::Splice(Splice {
            lhs: f(lhs),
            orig: f(orig),
            bit_range,
            value: f(value),
        }),
        OpCode::Unary(Unary { op, lhs, arg1 }) => OpCode::Unary(Unary {
            op,
            lhs: f(lhs),
            arg1: f(arg1),
        }),
    }
}

pub fn rename_read_operands(op: OpCode, old: Operand, new: Operand) -> OpCode {
    remap_operands(op, |op| if op == old { new } else { op })
}
