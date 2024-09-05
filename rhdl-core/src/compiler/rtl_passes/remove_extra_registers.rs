use crate::{
    rtl::{
        object::LocatedOpCode,
        remap::rename_read_operands,
        spec::{Assign, OpCode},
        Object,
    },
    RHDLError,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct RemoveExtraRegistersPass {}

fn find_assign_op(ops: &[LocatedOpCode]) -> Option<usize> {
    ops.iter()
        .position(|lop| matches!(lop.op, OpCode::Assign(_)))
}

impl Pass for RemoveExtraRegistersPass {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        while let Some(op_ndx) = find_assign_op(&input.ops) {
            let lop = input.ops[op_ndx].clone();
            let op = &lop.op;
            eprintln!("Found assign op {:?}", op);
            if let OpCode::Assign(assign) = op {
                input.ops = input
                    .ops
                    .into_iter()
                    .map(|lop| {
                        let LocatedOpCode { op, loc } = lop;
                        let old = assign.lhs;
                        let new = assign.rhs;
                        match op {
                            OpCode::Assign(Assign { lhs, rhs }) => {
                                let new_rhs = if rhs == old { new } else { rhs };
                                if new_rhs == lhs {
                                    LocatedOpCode {
                                        op: OpCode::Noop,
                                        loc,
                                    }
                                } else {
                                    LocatedOpCode {
                                        op: OpCode::Assign(Assign { lhs, rhs: new_rhs }),
                                        loc,
                                    }
                                }
                            }
                            _ => LocatedOpCode {
                                op: rename_read_operands(op, old, new),
                                loc,
                            },
                        }
                    })
                    .map(|lop| {
                        let LocatedOpCode { op, loc } = lop;
                        match op {
                            OpCode::Assign(Assign { lhs, rhs: _ }) => {
                                if lhs != assign.lhs {
                                    (op, loc).into()
                                } else {
                                    (OpCode::Noop, loc).into()
                                }
                            }
                            _ => (op, loc).into(),
                        }
                    })
                    .collect();
                input
                    .register_kind
                    .remove(&assign.lhs.as_register().unwrap());
                input.return_register = if input.return_register == assign.lhs {
                    assign.rhs
                } else {
                    input.return_register
                };
            }
        }
        Ok(input)
    }
}
