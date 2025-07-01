use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

use crate::{
    prelude::{Path, RHDLError, bit_range},
    rhdl_core::{
        compiler::mir::error::ICE,
        error::rhdl_error,
        ntl::{
            Object,
            error::NetLoopError,
            spec::{OpCode, Operand, RegisterId},
            visit::{Sense, visit_operands},
        },
        types::path::leaf_paths,
    },
};

use super::pass::Pass;

#[derive(Default, Debug, Clone)]
pub struct ReorderInstructions {}

fn raise_cycle_error(
    input: &Object,
    elements: Vec<(Option<String>, miette::SourceSpan)>,
) -> RHDLError {
    rhdl_error(NetLoopError {
        src: input.code.source(),
        elements,
    })
}

impl Pass for ReorderInstructions {
    fn run(mut input: Object) -> Result<Object, RHDLError> {
        // An implementation of Kahn's algorithm
        // The set N contains the set of register values that are
        // required for the reordering to be successful
        let mut needed = BTreeSet::<RegisterId>::new();
        needed.extend(input.outputs.iter().flat_map(Operand::reg));
        // The set S contains the working set of defined register values.
        let mut satisfied = VecDeque::<RegisterId>::default();
        // The vector P contains the ordering of op codes
        let mut scheduled = Vec::<usize>::default();
        // The set L contains the completed set of register values
        let mut finished = BTreeSet::<RegisterId>::default();
        // We start by pre-populating the satisfied set with all of the inputs
        satisfied.extend(input.inputs.iter().flatten());
        // Next we scan through all op-codes and pre-emit those that correspond
        // to black box invokations.  Since those are write-before read, we need
        // to treat them twice.
        input
            .ops
            .iter()
            .filter_map(|lop| match &lop.op {
                OpCode::BlackBox(blackbox) => Some(blackbox),
                _ => None,
            })
            .for_each(|black_box| {
                satisfied.extend(black_box.lhs.iter().filter_map(Operand::reg));
                needed.extend(black_box.arg.iter().flatten().flat_map(Operand::reg));
            });
        // Now, we create a pair of maps.  The first, maps each register to the set of
        // opcodes that depend on it.  The second maps each opcode to the set of registers
        // that it depends on.
        let mut reg_to_op = BTreeMap::<RegisterId, BTreeSet<usize>>::default();
        let mut op_to_read_regs = BTreeMap::<usize, BTreeSet<RegisterId>>::default();
        let mut write_regs_to_op = BTreeMap::<RegisterId, usize>::default();
        for (ndx, lop) in input.ops.iter().enumerate() {
            visit_operands(&lop.op, |sense, opnd| {
                if sense == Sense::Read {
                    if let Some(reg) = opnd.reg() {
                        reg_to_op.entry(reg).or_default().insert(ndx);
                        op_to_read_regs.entry(ndx).or_default().insert(reg);
                    }
                } else if let Some(reg) = opnd.reg() {
                    write_regs_to_op.insert(reg, ndx);
                }
            });
        }
        // Schedule any ops that do not depend on any inputs.  These are
        // op codes like comments, Noops, and op codes that take constants
        // as inputs (and probably should have been eliminated already).
        for (ndx, lop) in input.ops.iter().enumerate() {
            if !matches!(lop.op, OpCode::BlackBox(_)) && !op_to_read_regs.contains_key(&ndx) {
                scheduled.push(ndx);
                let op_code = &input.ops[ndx].op;
                visit_operands(op_code, |sense, operand| {
                    if sense == Sense::Write {
                        if let Some(reg) = operand.reg() {
                            satisfied.push_back(reg);
                        }
                    }
                });
            }
        }
        // Run the Kahn algorithm
        while let Some(n) = satisfied.pop_front() {
            finished.insert(n);
            let Some(dep_ops) = reg_to_op.remove(&n) else {
                // It is possible that no ops depend on a register
                continue;
            };
            for op in dep_ops {
                // The given opcode has a dependency on this register.
                // Remove the dependency
                let can_schedule = if let Some(deps) = op_to_read_regs.get_mut(&op) {
                    deps.remove(&n);
                    deps.is_empty()
                } else {
                    true
                };
                // If we can schedule this opcode, then add it to the scheduled list
                if can_schedule {
                    scheduled.push(op);
                    // Mark the outputs of this op code as satisfied, unless we are a black box
                    let op_code = &input.ops[op].op;
                    if !matches!(op_code, OpCode::BlackBox(_)) {
                        visit_operands(op_code, |sense, operand| {
                            if sense == Sense::Write {
                                if let Some(reg) = operand.reg() {
                                    satisfied.push_back(reg);
                                }
                            }
                        });
                    }
                }
            }
        }
        // Hope springs eternal...
        if let Some(failed) = needed.iter().find(|r| !finished.contains(r)).copied() {
            // Isolate a loop
            let mut regs = VecDeque::new();
            let mut visited = HashSet::new();
            loop {
                regs.push_back(failed);
                visited.insert(failed);
                // This is the opcode that writes the missing reg
                let opc = write_regs_to_op[&failed];
                // That opcode must be missing an argument (or it would have been scheduled already)
                let Some(&next) = op_to_read_regs[&opc].iter().next() else {
                    // This is an error, since if the op had no unsatisfied inputs
                    // it should have been scheduled.
                    return Err(Self::raise_ice(
                        &input,
                        ICE::LoopIsolationAlgorithmFailed,
                        None,
                    ));
                };
                if visited.contains(&next) {
                    // This reg is in the loop.  Discard regs from the
                    // list that come before this one
                    while !regs.is_empty() && regs.front() != Some(&next) {
                        regs.pop_front();
                    }
                    break;
                }
            }
            if regs.is_empty() {
                return Err(Self::raise_ice(
                    &input,
                    ICE::LoopIsolationAlgorithmFailed,
                    None,
                ));
            }

            // Construct a diagnostic.
            /*             let mut diag = vec![];
            for reg in regs {
                let opc = write_regs_to_op[&reg];
                let lop = &input.ops[opc];
                if let Some(src_op) = lop.loc {
                    let rtl_bit = src_op.bit.unwrap_or_default();
                    // Figure out where this source op code belongs
                    let rtl_obj = &input.rtl[&src_op.rtl.rhif.func];
                    let rtl_op = &rtl_obj.ops[src_op.op];
                    if let Some(src) = rtl_op.loc.op {
                        let rhif_obj = &rtl_obj.rhifs[&rtl_op.loc.rhif.func];
                        let rhif_lop = &rhif_obj.ops[src];
                        let rhif_op = &rhif_lop.op;
                        if let Some(lhs) = rhif_op.lhs() {
                            let ty = rhif_obj.kind(lhs);
                            let paths = leaf_paths(&ty, Path::default());
                            if let Some(path) = paths.iter().find(|p| {
                                let Ok((bits1, _)) = bit_range(ty, p) else {
                                    return false;
                                };
                                bits1.contains(&rtl_bit)
                            }) {
                                let value_description = if !path.is_empty() {
                                    Some(format!("{path:?}"))
                                } else {
                                    None
                                };
                                let span: miette::SourceSpan =
                                    input.code.span(src_op.rtl.rhif).into();
                                diag.push((value_description, span));
                            }
                        }
                    }
                }
            }
             return Err(raise_cycle_error(&input, diag));
             */
            todo!();
        }
        // Reorder and select
        let reordered = scheduled
            .into_iter()
            .map(|ndx| input.ops[ndx].clone())
            .collect();
        input.ops = reordered;
        Ok(input)
    }

    fn description() -> &'static str {
        "Reorder instructions to create legal dataflow"
    }
}
