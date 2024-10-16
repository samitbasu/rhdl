use std::collections::HashMap;

use crate::{
    ast::source_location::SourceLocation,
    hdl::ast::{unsigned_width, SignedWidth},
    rhif::spec::{AluBinary, AluUnary},
    rtl::{
        object::LocatedOpCode,
        spec::{OpCode, Operand},
        Object,
    },
};

use super::{
    component::{Binary, Case, CaseEntry, ComponentKind, DynamicIndex, DynamicSplice, Unary},
    edge_kind::EdgeKind,
    flow_graph_impl::{FlowGraph, FlowIx},
};

use crate::rtl::spec as tl;

struct FlowGraphBuilder<'a> {
    object: &'a Object,
    fg: FlowGraph,
    operand_map: HashMap<Operand, Vec<FlowIx>>,
}

pub fn build_rtl_flow_graph(object: &Object) -> FlowGraph {
    let mut bob = FlowGraphBuilder::new(object);
    object.ops.iter().for_each(|lop| bob.op(lop));
    // Link the arguments
    let location = object.symbols.fallback(object.fn_id);
    object
        .arguments
        .iter()
        .zip(bob.fg.inputs.clone())
        .for_each(|(o, f)| {
            if let (Some(reg), node) = (o, f) {
                let reg_operand = bob.operand(location, Operand::Register(*reg));
                for (reg, node) in reg_operand.iter().zip(node.iter()) {
                    bob.fg.edge(*node, *reg, EdgeKind::ArgBit(0, 0));
                }
            }
        });
    let ret_operand = bob.operand(location, object.return_register);
    for (reg, output) in ret_operand.iter().zip(bob.fg.output.clone().iter()) {
        bob.fg.edge(*reg, *output, EdgeKind::ArgBit(0, 0));
    }
    bob.fg
}

impl<'a> FlowGraphBuilder<'a> {
    fn new(object: &'a Object) -> Self {
        let mut fg = FlowGraph {
            code: object.symbols.source_set.clone(),
            ..Default::default()
        };
        // TODO - in the future, maybe tag the arguments and return with source locations?
        let name = &object.name;
        // Allocate input and output ports.
        let inputs = object
            .arguments
            .iter()
            .enumerate()
            .map(|(ndx, x)| {
                if let Some(reg) = x {
                    fg.input(object.register_kind[reg], ndx, name)
                } else {
                    vec![]
                }
            })
            .collect();
        let output_kind = object.kind(object.return_register);
        let output = fg.output(output_kind, name);
        fg.inputs = inputs;
        fg.output = output;
        Self {
            object,
            fg,
            operand_map: Default::default(),
        }
    }
    fn op(&mut self, lop: &LocatedOpCode) {
        let loc = lop.loc;
        match &lop.op {
            OpCode::Noop => {}
            OpCode::Assign(assign) => self.build_assign(loc, assign),
            OpCode::Binary(binary) => self.build_binary(loc, binary),
            OpCode::Case(case) => self.build_case(loc, case),
            OpCode::Cast(cast) => self.build_cast(loc, cast),
            OpCode::Comment(_) => {}
            OpCode::Concat(concat) => self.build_concat(loc, concat),
            OpCode::DynamicIndex(dynamic_index) => self.build_dynamic_index(loc, dynamic_index),
            OpCode::DynamicSplice(dynamic_splice) => self.build_dynamic_splice(loc, dynamic_splice),
            OpCode::Index(index) => self.build_index(loc, index),
            OpCode::Select(select) => self.build_select(loc, select),
            OpCode::Splice(splice) => self.build_splice(loc, splice),
            OpCode::Unary(unary) => self.build_unary(loc, unary),
        }
    }
    fn operand(&mut self, loc: SourceLocation, operand: Operand) -> Vec<FlowIx> {
        if let Some(port) = self.operand_map.get(&operand) {
            return port.clone();
        }
        match operand {
            Operand::Literal(literal_id) => {
                let bs = &self.object.literals[&literal_id];
                let ndx = bs
                    .bits()
                    .iter()
                    .map(|b| self.fg.new_component(ComponentKind::Constant(*b), 1, loc))
                    .collect::<Vec<_>>();
                self.operand_map.insert(operand, ndx.clone());
                ndx
            }
            Operand::Register(register_id) => {
                let reg = self.object.register_kind[&register_id];
                let ndx = (0..reg.len())
                    .map(|ndx| {
                        self.fg.new_component(
                            ComponentKind::Buffer(format!(
                                "{}_{:?}[{ndx}]",
                                self.object.name, operand
                            )),
                            1,
                            loc,
                        )
                    })
                    .collect::<Vec<_>>();
                self.operand_map.insert(operand, ndx.clone());
                ndx
            }
        }
    }
    fn build_assign(&mut self, loc: SourceLocation, assign: &tl::Assign) {
        let rhs = self.operand(loc, assign.rhs);
        let lhs = self.operand(loc, assign.lhs);
        for (lhs, rhs) in lhs.iter().zip(rhs.iter()) {
            self.fg.edge(*rhs, *lhs, EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_binary(&mut self, loc: SourceLocation, binary: &tl::Binary) {
        let arg1 = self.operand(loc, binary.arg1);
        let arg1_len: SignedWidth = self.object.kind(binary.arg1).into();
        let arg2 = self.operand(loc, binary.arg2);
        let arg2_len: SignedWidth = self.object.kind(binary.arg2).into();
        let lhs = self.operand(loc, binary.lhs);
        if is_bitwise_binary(binary.op) {
            for (lhs, (arg1, arg2)) in lhs.iter().zip(arg1.iter().zip(arg2.iter())) {
                let comp = self.fg.new_component(
                    ComponentKind::Binary(Binary {
                        op: binary.op,
                        left_len: unsigned_width(1),
                        right_len: unsigned_width(1),
                    }),
                    1,
                    loc,
                );
                self.fg.edge(*arg1, comp, EdgeKind::ArgBit(0, 0));
                self.fg.edge(*arg2, comp, EdgeKind::ArgBit(1, 0));
                self.fg.edge(comp, *lhs, EdgeKind::ArgBit(0, 0));
            }
        } else {
            let comp = self.fg.new_component(
                ComponentKind::Binary(Binary {
                    op: binary.op,
                    left_len: arg1_len,
                    right_len: arg2_len,
                }),
                arg1.len(),
                loc,
            );
            for (ndx, lhs) in lhs.iter().enumerate() {
                let bit = self.fg.bit_select(comp, ndx);
                self.fg.edge(bit, *lhs, EdgeKind::ArgBit(0, 0));
            }
            for (ndx, arg1) in arg1.iter().enumerate() {
                self.fg.edge(*arg1, comp, EdgeKind::ArgBit(0, ndx));
            }
            for (ndx, arg2) in arg2.iter().enumerate() {
                self.fg.edge(*arg2, comp, EdgeKind::ArgBit(1, ndx));
            }
        }
    }
    fn build_case(&mut self, loc: SourceLocation, case: &tl::Case) {
        let lhs = self.operand(loc, case.lhs);
        let discriminant = self.operand(loc, case.discriminant);
        let table = case
            .table
            .iter()
            .map(|(x, _)| {
                if let tl::CaseArgument::Literal(lit_id) = x {
                    CaseEntry::Literal(self.object.literals[lit_id].clone())
                } else {
                    CaseEntry::WildCard
                }
            })
            .collect::<Vec<_>>();
        let arguments = case
            .table
            .iter()
            .map(|(_, x)| self.operand(loc, *x))
            .collect::<Vec<_>>();
        let num_bits = lhs.len();
        for bit in 0..num_bits {
            let comp = self.fg.new_component(
                ComponentKind::Case(Case {
                    entries: table.clone(),
                    discriminant_width: discriminant.len(),
                }),
                1,
                loc,
            );
            for (ndx, disc) in discriminant.iter().enumerate() {
                self.fg.edge(*disc, comp, EdgeKind::Selector(ndx));
            }
            for (ndx, arg) in arguments.iter().enumerate() {
                self.fg.edge(arg[bit], comp, EdgeKind::ArgBit(ndx, bit));
            }
            self.fg.edge(comp, lhs[bit], EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_cast(&mut self, loc: SourceLocation, cast: &tl::Cast) {
        let lhs = self.operand(loc, cast.lhs);
        let arg = self.operand(loc, cast.arg);
        for (lhs, rhs) in lhs.iter().zip(arg.iter()) {
            self.fg.edge(*rhs, *lhs, EdgeKind::ArgBit(0, 0));
        }
        let lhs_signed = self.object.kind(cast.lhs).is_signed();
        let use_unsigned = matches!(cast.kind, tl::CastKind::Unsigned)
            || (matches!(cast.kind, tl::CastKind::Resize) && !lhs_signed);
        if use_unsigned {
            let zero = self
                .fg
                .new_component(ComponentKind::Constant(false), lhs.len(), loc);
            for lhs in lhs.iter().skip(arg.len()) {
                self.fg.edge(zero, *lhs, EdgeKind::ArgBit(0, 0));
            }
        } else {
            let msb = arg.last().unwrap();
            for lhs in lhs.iter().skip(arg.len()) {
                self.fg.edge(*msb, *lhs, EdgeKind::ArgBit(0, 0));
            }
        }
    }
    fn build_concat(&mut self, loc: SourceLocation, concat: &tl::Concat) {
        let lhs = self.operand(loc, concat.lhs);
        let args = concat
            .args
            .iter()
            .map(|x| self.operand(loc, *x))
            .collect::<Vec<_>>();
        for (lhs, rhs) in lhs.iter().zip(args.iter().flatten()) {
            self.fg.edge(*rhs, *lhs, EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_dynamic_index(&mut self, loc: SourceLocation, dynamic_index: &tl::DynamicIndex) {
        let lhs = self.operand(loc, dynamic_index.lhs);
        let arg = self.operand(loc, dynamic_index.arg);
        let offset = self.operand(loc, dynamic_index.offset);
        let comp = self.fg.new_component(
            ComponentKind::DynamicIndex(DynamicIndex {
                offset_len: offset.len(),
                arg_len: arg.len(),
            }),
            lhs.len(),
            loc,
        );
        for (ndx, lhs) in lhs.iter().enumerate() {
            let bit = self.fg.bit_select(comp, ndx);
            self.fg.edge(bit, *lhs, EdgeKind::ArgBit(0, 0));
        }
        for (ndx, offset) in offset.iter().enumerate() {
            self.fg.edge(*offset, comp, EdgeKind::DynamicOffset(ndx));
        }
        for (ndx, arg) in arg.iter().enumerate() {
            self.fg.edge(*arg, comp, EdgeKind::ArgBit(0, ndx));
        }
    }
    fn build_dynamic_splice(&mut self, loc: SourceLocation, dynamic_splice: &tl::DynamicSplice) {
        let lhs = self.operand(loc, dynamic_splice.lhs);
        let arg = self.operand(loc, dynamic_splice.arg);
        let offset = self.operand(loc, dynamic_splice.offset);
        let value = self.operand(loc, dynamic_splice.value);
        let comp = self.fg.new_component(
            ComponentKind::DynamicSplice(DynamicSplice {
                splice_len: dynamic_splice.len,
                offset_len: offset.len(),
            }),
            lhs.len(),
            loc,
        );
        for (ndx, lhs) in lhs.iter().enumerate() {
            let bit = self.fg.bit_select(comp, ndx);
            self.fg.edge(bit, *lhs, EdgeKind::ArgBit(0, 0));
        }
        for (ndx, offset) in offset.iter().enumerate() {
            self.fg.edge(*offset, comp, EdgeKind::DynamicOffset(ndx));
        }
        for (ndx, arg) in arg.iter().enumerate() {
            self.fg.edge(*arg, comp, EdgeKind::ArgBit(0, ndx));
        }
        for (ndx, value) in value.iter().enumerate() {
            self.fg.edge(*value, comp, EdgeKind::Splice(ndx));
        }
    }
    fn build_index(&mut self, loc: SourceLocation, index: &tl::Index) {
        let lhs = self.operand(loc, index.lhs);
        let arg = self.operand(loc, index.arg);
        for (lhs, rhs) in lhs.iter().zip(arg.iter().skip(index.bit_range.start)) {
            self.fg.edge(*rhs, *lhs, EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_select(&mut self, loc: SourceLocation, select: &tl::Select) {
        let lhs = self.operand(loc, select.lhs);
        let cond = self.operand(loc, select.cond);
        let true_value = self.operand(loc, select.true_value);
        let false_value = self.operand(loc, select.false_value);
        for (lhs, (true_val, false_val)) in
            lhs.iter().zip(true_value.iter().zip(false_value.iter()))
        {
            let comp = self.fg.new_component(ComponentKind::Select, 1, loc);
            self.fg.edge(cond[0], comp, EdgeKind::Selector(0));
            self.fg.edge(*true_val, comp, EdgeKind::True);
            self.fg.edge(*false_val, comp, EdgeKind::False);
            self.fg.edge(comp, *lhs, EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_splice(&mut self, loc: SourceLocation, splice: &tl::Splice) {
        let lhs = self.operand(loc, splice.lhs);
        let orig = self.operand(loc, splice.orig);
        let value = self.operand(loc, splice.value);
        let lsb_iter = orig.iter().take(splice.bit_range.start);
        let center_iter = value.iter();
        let msb_iter = orig.iter().skip(splice.bit_range.end);
        let rhs = lsb_iter.chain(center_iter).chain(msb_iter);
        for (lhs, rhs) in lhs.iter().zip(rhs) {
            self.fg.edge(*rhs, *lhs, EdgeKind::ArgBit(0, 0));
        }
    }
    fn build_unary(&mut self, loc: SourceLocation, unary: &tl::Unary) {
        let lhs = self.operand(loc, unary.lhs);
        let arg1 = self.operand(loc, unary.arg1);
        if is_bitwise_unary(unary.op) {
            for (lhs, rhs) in lhs.iter().zip(arg1.iter()) {
                let comp = self.fg.new_component(
                    ComponentKind::Unary(Unary {
                        op: unary.op,
                        arg_len: unsigned_width(1),
                    }),
                    1,
                    loc,
                );
                self.fg.edge(*rhs, comp, EdgeKind::ArgBit(0, 0));
                self.fg.edge(comp, *lhs, EdgeKind::ArgBit(0, 0));
            }
        } else {
            let comp = self.fg.new_component(
                ComponentKind::Unary(Unary {
                    op: unary.op,
                    arg_len: self.object.kind(unary.arg1).into(),
                }),
                lhs.len(),
                loc,
            );
            for (ndx, rhs) in arg1.iter().enumerate() {
                self.fg.edge(*rhs, comp, EdgeKind::ArgBit(0, ndx));
            }
            for (ndx, lhs) in lhs.iter().enumerate() {
                let bit = self.fg.bit_select(comp, ndx);
                self.fg.edge(bit, *lhs, EdgeKind::ArgBit(0, 0));
            }
        }
    }
}

fn is_bitwise_unary(op: AluUnary) -> bool {
    matches!(op, AluUnary::Not | AluUnary::Signed | AluUnary::Unsigned)
}

fn is_bitwise_binary(op: AluBinary) -> bool {
    matches!(op, AluBinary::BitAnd | AluBinary::BitOr | AluBinary::BitXor)
}
