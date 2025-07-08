use super::spec::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sense {
    Read,
    Write,
}

pub fn visit_operands<F: FnMut(Sense, &Operand)>(op: &OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, rhs);
        }
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
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, discriminant);
            for (arg, entry) in table {
                f(Sense::Read, entry);
                if let CaseArgument::Literal(lit) = arg {
                    f(Sense::Read, &Operand::Literal(*lit));
                }
            }
        }
        OpCode::Cast(Cast {
            lhs,
            arg,
            len: _,
            kind: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Comment(_) => {}
        OpCode::Concat(Concat { lhs, args }) => {
            f(Sense::Write, lhs);
            for arg in args {
                f(Sense::Read, arg);
            }
        }
        OpCode::Index(Index {
            lhs,
            arg,
            bit_range: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
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
        OpCode::Splice(Splice {
            lhs,
            orig,
            bit_range: _,
            value,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, orig);
            f(Sense::Read, value);
        }
        OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
        }
    }
}

pub fn visit_operands_mut<F: FnMut(Sense, &mut Operand)>(op: &mut OpCode, mut f: F) {
    match op {
        OpCode::Noop => {}
        OpCode::Assign(Assign { lhs, rhs }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, rhs);
        }
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
        OpCode::Case(Case {
            lhs,
            discriminant,
            table,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, discriminant);
            for (arg, entry) in table {
                f(Sense::Read, entry);
                if let CaseArgument::Literal(lit) = arg {
                    let mut operand = Operand::Literal(*lit);
                    f(Sense::Read, &mut operand);
                    let Operand::Literal(lid) = operand else {
                        panic!("Literal expected in case at this point!");
                    };
                    *lit = lid;
                }
            }
        }
        OpCode::Cast(Cast {
            lhs,
            arg,
            len: _,
            kind: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
        }
        OpCode::Comment(_) => {}
        OpCode::Concat(Concat { lhs, args }) => {
            f(Sense::Write, lhs);
            for arg in args {
                f(Sense::Read, arg);
            }
        }
        OpCode::Index(Index {
            lhs,
            arg,
            bit_range: _,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg);
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
        OpCode::Splice(Splice {
            lhs,
            orig,
            bit_range: _,
            value,
        }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, orig);
            f(Sense::Read, value);
        }
        OpCode::Unary(Unary { op: _, lhs, arg1 }) => {
            f(Sense::Write, lhs);
            f(Sense::Read, arg1);
        }
    }
}
