use super::spec::*;

fn vec_f<F: FnMut(Operand) -> Operand>(f: &mut F, v: Vec<Operand>) -> Vec<Operand> {
    v.into_iter().map(|x| f(x)).collect::<Vec<_>>()
}

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
        OpCode::Vector(Vector {
            op,
            lhs,
            arg1,
            arg2,
            signed,
        }) => OpCode::Vector(Vector {
            op,
            lhs: vec_f(&mut f, lhs),
            arg1: vec_f(&mut f, arg1),
            arg2: vec_f(&mut f, arg2),
            signed,
        }),
        OpCode::Case(Case {
            lhs,
            discriminant,
            entries,
        }) => OpCode::Case(Case {
            lhs: f(lhs),
            discriminant: vec_f(&mut f, discriminant),
            entries: entries
                .into_iter()
                .map(|(case_entry, op)| (case_entry, f(op)))
                .collect(),
        }),
        OpCode::Comment(comment) => OpCode::Comment(comment),
        OpCode::DynamicIndex(DynamicIndex { lhs, arg, offset }) => {
            OpCode::DynamicIndex(DynamicIndex {
                lhs: vec_f(&mut f, lhs),
                arg: vec_f(&mut f, arg),
                offset: vec_f(&mut f, offset),
            })
        }
        OpCode::DynamicSplice(DynamicSplice {
            lhs,
            arg,
            offset,
            value,
        }) => OpCode::DynamicSplice(DynamicSplice {
            lhs: vec_f(&mut f, lhs),
            arg: vec_f(&mut f, arg),
            offset: vec_f(&mut f, offset),
            value: vec_f(&mut f, value),
        }),
        OpCode::Select(Select {
            lhs,
            selector,
            true_case,
            false_case,
        }) => OpCode::Select(Select {
            lhs: f(lhs),
            selector: f(selector),
            true_case: f(true_case),
            false_case: f(false_case),
        }),
        OpCode::Not(Not { lhs, arg }) => OpCode::Not(Not {
            lhs: f(lhs),
            arg: f(arg),
        }),
        OpCode::Dff(Dff {
            lhs,
            arg,
            clock,
            reset,
            reset_value,
        }) => OpCode::Dff(Dff {
            lhs: f(lhs),
            arg: f(arg),
            clock: f(clock),
            reset: f(reset),
            reset_value,
        }),
        OpCode::BlackBox(BlackBox { lhs, arg, code }) => OpCode::BlackBox(BlackBox {
            lhs: vec_f(&mut f, lhs),
            arg: vec_f(&mut f, arg),
            code,
        }),
        OpCode::Unary(Unary { op, lhs, arg }) => OpCode::Unary(Unary {
            op,
            lhs: vec_f(&mut f, lhs),
            arg: vec_f(&mut f, arg),
        }),
    }
}
