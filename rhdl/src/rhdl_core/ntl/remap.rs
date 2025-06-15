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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Read,
    Write,
}

fn vec_v<F: FnMut(Sense, &Operand)>(f: &mut F, sense: Sense, v: &[Operand]) {
    v.iter().for_each(|v| f(sense, v));
}

pub fn visit_operands<F: FnMut(Sense, &Operand)>(op: &OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, rhs);
        }
        OpCode::Binary(Binary {
            op: _,
            lhs,
            arg1,
            arg2,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
            f(Sense::Read, arg2);
        }
        OpCode::Vector(Vector {
            op: _,
            lhs,
            arg1,
            arg2,
            signed: _,
        }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg1);
            vec_v(&mut f, Sense::Read, arg2);
        }
        OpCode::Case(Case {
            lhs,
            discriminant,
            entries,
        }) => {
            f(Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, discriminant);
            for (_, entry) in entries {
                f(Sense::Read, entry);
            }
        }
        OpCode::Comment(_) => {}
        OpCode::DynamicIndex(DynamicIndex { lhs, arg, offset }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg);
            vec_v(&mut f, Sense::Read, offset);
        }
        OpCode::DynamicSplice(DynamicSplice {
            lhs,
            arg,
            offset,
            value,
        }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg);
            vec_v(&mut f, Sense::Read, offset);
            vec_v(&mut f, Sense::Read, value);
        }
        OpCode::Select(Select {
            lhs,
            selector,
            true_case,
            false_case,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, selector);
            f(Sense::Read, true_case);
            f(Sense::Read, false_case);
        }
        OpCode::Not(Not { lhs, arg }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Dff(Dff {
            lhs,
            arg,
            clock,
            reset,
            reset_value: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
            f(Sense::Read, clock);
            f(Sense::Read, reset);
        }
        OpCode::BlackBox(BlackBox { lhs, arg, code }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg);
        }
        OpCode::Unary(Unary { op, lhs, arg }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg);
        }
    }
}

fn vec_m<F: FnMut(&mut Operand)>(f: &mut F, v: &mut [Operand]) {
    for op in v {
        f(op)
    }
}

pub fn visit_operands_mut<F: FnMut(&mut Operand)>(op: &mut OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(lhs);
            f(rhs);
        }
        OpCode::Binary(Binary {
            op,
            lhs,
            arg1,
            arg2,
        }) => {
            f(lhs);
            f(arg1);
            f(arg2);
        }
        OpCode::Vector(Vector {
            op,
            lhs,
            arg1,
            arg2,
            signed,
        }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg1);
            vec_m(&mut f, arg2);
        }
        OpCode::Case(Case {
            lhs,
            discriminant,
            entries,
        }) => {
            f(lhs);
            vec_m(&mut f, discriminant);
            for (_, entry) in entries {
                f(entry);
            }
        }
        OpCode::Comment(comment) => {}
        OpCode::DynamicIndex(DynamicIndex { lhs, arg, offset }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg);
            vec_m(&mut f, offset);
        }
        OpCode::DynamicSplice(DynamicSplice {
            lhs,
            arg,
            offset,
            value,
        }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg);
            vec_m(&mut f, offset);
            vec_m(&mut f, value);
        }
        OpCode::Select(Select {
            lhs,
            selector,
            true_case,
            false_case,
        }) => {
            f(lhs);
            f(selector);
            f(true_case);
            f(false_case);
        }
        OpCode::Not(Not { lhs, arg }) => {
            f(lhs);
            f(arg);
        }
        OpCode::Dff(Dff {
            lhs,
            arg,
            clock,
            reset,
            reset_value,
        }) => {
            f(lhs);
            f(arg);
            f(clock);
            f(reset);
        }
        OpCode::BlackBox(BlackBox { lhs, arg, code }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg);
        }
        OpCode::Unary(Unary { op, lhs, arg }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg);
        }
    }
}
