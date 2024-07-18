use crate::{
    ast::ast_impl::NodeId,
    compiler::mir::error::{RHDLSyntaxError, Syntax},
    error::RHDLError,
    rhif::{
        spec::{AluBinary, AluUnary, Binary, OpCode, Slot, Unary},
        Object,
    },
    Kind,
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct CheckForRolledTypesPass {}

fn check_register_like(
    obj: &Object,
    vals: &[(Slot, Kind)],
    cause: Syntax,
    expression: NodeId,
) -> Result<(), RHDLError> {
    for (slot, kind) in vals {
        let kind = kind.signal_data();
        if !matches!(kind, Kind::Bits(_) | Kind::Signed(_)) {
            return Err(Box::new(RHDLSyntaxError {
                src: obj.symbols.source.source.clone(),
                cause,
                err_span: obj
                    .symbols
                    .best_span_for_slot_in_expression(*slot, expression)
                    .into(),
            })
            .into());
        }
    }
    Ok(())
}

impl Pass for CheckForRolledTypesPass {
    fn name() -> &'static str {
        "check_for_rolled_types"
    }
    fn run(obj: Object) -> Result<Object, RHDLError> {
        let slot_type = |slot: &Slot| -> Kind { obj.kind(*slot) };
        let roll_error = |cause: Syntax, slot: Slot, id: NodeId| -> RHDLError {
            Box::new(RHDLSyntaxError {
                src: obj.symbols.source.source.clone(),
                cause,
                err_span: obj
                    .symbols
                    .best_span_for_slot_in_expression(slot, id)
                    .into(),
            })
            .into()
        };
        for lop in &obj.ops {
            let id = lop.id;
            let op = &lop.op;
            match op {
                OpCode::Binary(Binary {
                    op:
                        AluBinary::Add
                        | AluBinary::Sub
                        | AluBinary::Mul
                        | AluBinary::BitAnd
                        | AluBinary::BitOr
                        | AluBinary::BitXor,
                    lhs,
                    arg1,
                    arg2,
                }) => {
                    check_register_like(
                        &obj,
                        &[
                            (*lhs, slot_type(lhs)),
                            (*arg1, slot_type(arg1)),
                            (*arg2, slot_type(arg2)),
                        ],
                        Syntax::RollYourOwnBinary,
                        id,
                    )?;
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Val,
                    lhs: _,
                    arg1,
                }) => {
                    let kind = slot_type(arg1);
                    if !kind.is_signal() {
                        return Err(roll_error(
                            Syntax::RollYourOwnUnary { op: AluUnary::Val },
                            *arg1,
                            id,
                        ));
                    }
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Signed,
                    lhs: _,
                    arg1,
                }) => {
                    let kind = slot_type(arg1).signal_data();
                    if !matches!(kind, Kind::Bits(_) | Kind::Empty) {
                        return Err(roll_error(
                            Syntax::RollYourOwnUnary {
                                op: AluUnary::Signed,
                            },
                            *arg1,
                            id,
                        ));
                    }
                }
                OpCode::Unary(Unary {
                    op: AluUnary::Unsigned,
                    lhs: _,
                    arg1,
                }) => {
                    let kind = slot_type(arg1).signal_data();
                    if !matches!(kind, Kind::Signed(_) | Kind::Empty) {
                        return Err(roll_error(
                            Syntax::RollYourOwnUnary {
                                op: AluUnary::Unsigned,
                            },
                            *arg1,
                            id,
                        ));
                    }
                }
                OpCode::Unary(Unary { op, lhs, arg1 }) => {
                    check_register_like(
                        &obj,
                        &[(*lhs, slot_type(lhs)), (*arg1, slot_type(arg1))],
                        Syntax::RollYourOwnUnary { op: *op },
                        id,
                    )?;
                }
                _ => {}
            }
        }
        Ok(obj)
    }
}
