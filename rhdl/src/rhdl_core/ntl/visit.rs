use super::spec::*;

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
        OpCode::BlackBox(BlackBox { lhs, arg, code }) => {
            vec_v(&mut f, Sense::Write, lhs);
            for a in arg {
                vec_v(&mut f, Sense::Read, a);
            }
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
        OpCode::BlackBox(BlackBox { lhs, arg, code: _ }) => {
            vec_m(&mut f, lhs);
            for mut a in arg {
                vec_m(&mut f, a);
            }
        }
        OpCode::Unary(Unary { op, lhs, arg }) => {
            vec_m(&mut f, lhs);
            vec_m(&mut f, arg);
        }
    }
}
