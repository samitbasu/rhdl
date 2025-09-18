use crate::{common::sense::Sense, ntl::Object};

use super::spec::*;

fn vec_v<F: FnMut(Sense, &Wire)>(f: &mut F, sense: Sense, v: &[Wire]) {
    v.iter().for_each(|v| f(sense, v));
}

pub fn visit_wires<F: FnMut(Sense, &Wire)>(op: &OpCode, mut f: F) {
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
        OpCode::BlackBox(BlackBox { lhs, arg, code: _ }) => {
            vec_v(&mut f, Sense::Write, lhs);
            for a in arg {
                vec_v(&mut f, Sense::Read, a);
            }
        }
        OpCode::Unary(Unary { op: _, lhs, arg }) => {
            vec_v(&mut f, Sense::Write, lhs);
            vec_v(&mut f, Sense::Read, arg);
        }
    }
}

fn vec_m<F: FnMut(Sense, &mut Wire)>(f: &mut F, sense: Sense, v: &mut [Wire]) {
    for op in v {
        f(sense, op)
    }
}

pub fn visit_wires_mut<F: FnMut(Sense, &mut Wire)>(op: &mut OpCode, mut f: F) {
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
            vec_m(&mut f, Sense::Write, lhs);
            vec_m(&mut f, Sense::Read, arg1);
            vec_m(&mut f, Sense::Read, arg2);
        }
        OpCode::Case(Case {
            lhs,
            discriminant,
            entries,
        }) => {
            f(Sense::Write, lhs);
            vec_m(&mut f, Sense::Read, discriminant);
            for (_, entry) in entries {
                f(Sense::Read, entry);
            }
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
        OpCode::BlackBox(BlackBox { lhs, arg, code: _ }) => {
            vec_m(&mut f, Sense::Write, lhs);
            for a in arg {
                vec_m(&mut f, Sense::Read, a);
            }
        }
        OpCode::Unary(Unary { op: _, lhs, arg }) => {
            vec_m(&mut f, Sense::Write, lhs);
            vec_m(&mut f, Sense::Read, arg);
        }
    }
}

pub fn visit_object_wires<F: FnMut(Sense, &Wire)>(object: &Object, mut f: F) {
    for arg in object.inputs.iter().flatten() {
        f(Sense::Write, &Wire::Register(*arg))
    }
    for lop in &object.ops {
        visit_wires(&lop.op, &mut f);
    }
    for out in &object.outputs {
        f(Sense::Read, out)
    }
}

pub fn visit_object_wires_mut<F: FnMut(Sense, &mut Wire)>(object: &mut Object, mut f: F) {
    for arg in object.inputs.iter_mut().flatten() {
        let mut wire = Wire::Register(*arg);
        f(Sense::Write, &mut wire);
        *arg = wire
            .reg()
            .expect("Argument operands must remain registers.  Do not mutate them into literals");
    }
    for lop in object.ops.iter_mut() {
        visit_wires_mut(&mut lop.op, &mut f);
    }
    for out in object.outputs.iter_mut() {
        f(Sense::Read, out)
    }
}
