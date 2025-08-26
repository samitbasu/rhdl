use super::pass::Pass;

use crate::core::ntl::object::*;
use crate::core::ntl::spec::*;
use crate::prelude::BitX;
use crate::prelude::RHDLError;

pub struct LowerSelects;

fn rewrite_select_with_hardwired_selector(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(bitx) = object.bitx(select.selector) else {
        return;
    };
    let Some(sel) = bitx.to_bool() else {
        return;
    };
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: if sel {
            select.true_case
        } else {
            select.false_case
        },
    });
}

fn rewrite_select_with_equal_branches(op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    if select.false_case != select.true_case {
        return;
    }
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: select.true_case,
    })
}

fn rewrite_select_with_not(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(BitX::One) = object.bitx(select.false_case) else {
        return;
    };
    let Some(BitX::Zero) = object.bitx(select.true_case) else {
        return;
    };
    *op = OpCode::Not(Not {
        lhs: select.lhs,
        arg: select.selector,
    })
}

// a <- s ? b : X
// a <- b
fn rewrite_select_with_dont_care_in_false(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(BitX::X) = object.bitx(select.false_case) else {
        return;
    };
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: select.true_case,
    })
}

// a <- s ? X : b
// a <- b
fn rewrite_select_with_dont_care_in_true(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(BitX::X) = object.bitx(select.true_case) else {
        return;
    };
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: select.false_case,
    })
}

fn rewrite_select_with_assign(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(BitX::Zero) = object.bitx(select.false_case) else {
        return;
    };
    let Some(BitX::One) = object.bitx(select.true_case) else {
        return;
    };
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: select.selector,
    })
}

fn rewrite_select_with_zero_false(object: &Object, op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(BitX::Zero) = object.bitx(select.false_case) else {
        return;
    };
    *op = OpCode::Binary(Binary {
        op: BinaryOp::And,
        lhs: select.lhs,
        arg1: select.selector,
        arg2: select.true_case,
    })
}

fn lower_select(input: &Object, op: &mut OpCode) {
    rewrite_select_with_equal_branches(op);
    rewrite_select_with_hardwired_selector(input, op);
    rewrite_select_with_not(input, op);
    rewrite_select_with_assign(input, op);
    rewrite_select_with_dont_care_in_false(input, op);
    rewrite_select_with_dont_care_in_true(input, op);
    rewrite_select_with_zero_false(input, op);
}

impl Pass for LowerSelects {
    fn description() -> &'static str {
        "Lower Select opcodes where possible"
    }

    fn run(mut input: Object) -> Result<Object, RHDLError> {
        let mut ops = std::mem::take(&mut input.ops);
        ops.iter_mut().for_each(|lop| {
            lower_select(&input, &mut lop.op);
        });
        input.ops = ops;
        Ok(input)
    }
}
