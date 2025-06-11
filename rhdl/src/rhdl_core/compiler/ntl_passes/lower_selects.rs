use super::pass::Pass;

use crate::core::ntl::object::*;
use crate::core::ntl::spec::*;
use crate::prelude::RHDLError;

pub struct LowerSelects;

fn rewrite_select_with_hardwired_selector(op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Some(bitx) = select.selector.bitx() else {
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

fn rewrite_select_with_not(op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Operand::One = select.false_case else {
        return;
    };
    let Operand::Zero = select.true_case else {
        return;
    };
    *op = OpCode::Not(Not {
        lhs: select.lhs,
        arg: select.selector,
    })
}

fn rewrite_select_with_assign(op: &mut OpCode) {
    let OpCode::Select(select) = &op else {
        return;
    };
    let Operand::Zero = select.false_case else {
        return;
    };
    let Operand::One = select.true_case else {
        return;
    };
    *op = OpCode::Assign(Assign {
        lhs: select.lhs,
        rhs: select.selector,
    })
}

fn lower_select(op: &mut OpCode) {
    rewrite_select_with_equal_branches(op);
    rewrite_select_with_hardwired_selector(op);
    rewrite_select_with_not(op);
    rewrite_select_with_assign(op);
}

impl Pass for LowerSelects {
    fn description() -> &'static str {
        "Lower Select opcodes where possible"
    }

    fn run(mut input: Object) -> Result<Object, RHDLError> {
        input.ops.iter_mut().for_each(|lop| {
            lower_select(&mut lop.op);
        });
        Ok(input)
    }
}
