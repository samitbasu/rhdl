use std::collections::HashMap;

use crate::{
    rhif::{
        object::SourceLocation,
        spec::{AluBinary, AluUnary},
    },
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
    let location = (object.fn_id, object.symbols[&object.fn_id].source.fallback).into();
    object
        .arguments
        .iter()
        .zip(bob.fg.inputs.clone())
        .for_each(|(o, f)| {
            if let (Some(reg), Some(node)) = (o, f) {
                let reg_operand = bob.operand(location, Operand::Register(*reg));
                for (ndx, reg) in reg_operand.iter().enumerate() {
                    bob.fg.edge(*reg, node, EdgeKind::ArgBit(0, ndx));
                }
            }
        });
    let ret_operand = bob.operand(location, object.return_register);
    for (ndx, reg) in ret_operand.iter().enumerate() {
        bob.fg.edge(bob.fg.output, *reg, EdgeKind::ArgBit(0, ndx));
    }
    bob.fg
}

impl<'a> FlowGraphBuilder<'a> {
    fn new(object: &'a Object) -> Self {
        let mut fg = FlowGraph::default();
        // TODO - in the future, maybe tag the arguments and return with source locations?
        let location = None;
        // Allocate input and output ports.
        let inputs = object
            .arguments
            .iter()
            .enumerate()
            .map(|(ndx, x)| {
                x.map(|reg| fg.buffer(object.register_kind[&reg], &format!("a{ndx}"), location))
            })
            .collect();
        let output_kind = object.kind(object.return_register);
        let output = fg.buffer(output_kind, "y", location);
        fg.inputs = inputs;
        fg.output = output;
        Self {
            object,
            fg,
            operand_map: Default::default(),
        }
    }
    fn op(&mut self, lop: &LocatedOpCode) {
        let loc = SourceLocation {
            func: lop.func,
            node: lop.id,
        };
        match &lop.op {
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
                    .map(|b| self.fg.new_component(ComponentKind::Constant(*b), loc))
                    .collect::<Vec<_>>();
                self.operand_map.insert(operand, ndx.clone());
                ndx
            }
            Operand::Register(register_id) => {
                let reg = self.object.register_kind[&register_id];
                let ndx = (0..reg.len())
                    .map(|_| self.fg.new_component(ComponentKind::Buffer, loc))
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
            self.fg.graph.add_edge(*rhs, *lhs, EdgeKind::Arg(0));
        }
    }
    fn build_binary(&mut self, loc: SourceLocation, binary: &tl::Binary) {
        let arg1 = self.operand(loc, binary.arg1);
        let arg2 = self.operand(loc, binary.arg2);
        let lhs = self.operand(loc, binary.lhs);
        let len = self.object.kind(binary.arg1).len();
        if is_bitwise_binary(binary.op) {
            for (lhs, (arg1, arg2)) in lhs.iter().zip(arg1.iter().zip(arg2.iter())) {
                let comp = self
                    .fg
                    .new_component(ComponentKind::Binary(Binary { op: binary.op }), loc);
                self.fg.graph.add_edge(comp, *arg1, EdgeKind::Arg(0));
                self.fg.graph.add_edge(comp, *arg2, EdgeKind::Arg(1));
                self.fg.graph.add_edge(*lhs, comp, EdgeKind::Arg(0));
            }
        } else {
            let comp = self
                .fg
                .new_component(ComponentKind::Binary(Binary { op: binary.op }), loc);
            for (ndx, (lhs, (arg1, arg2))) in
                lhs.iter().zip(arg1.iter().zip(arg2.iter())).enumerate()
            {
                self.fg
                    .graph
                    .add_edge(comp, *arg1, EdgeKind::ArgBit(0, ndx));
                self.fg
                    .graph
                    .add_edge(comp, *arg2, EdgeKind::ArgBit(1, ndx));
                self.fg.graph.add_edge(*lhs, comp, EdgeKind::ArgBit(0, ndx));
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
                }),
                loc,
            );
            for (ndx, disc) in discriminant.iter().enumerate() {
                self.fg.graph.add_edge(comp, *disc, EdgeKind::Selector(ndx));
            }
            for (ndx, arg) in arguments.iter().enumerate() {
                self.fg
                    .graph
                    .add_edge(comp, arg[bit], EdgeKind::ArgBit(ndx, bit));
            }
            self.fg.graph.add_edge(lhs[bit], comp, EdgeKind::Arg(0));
        }
    }
    fn build_cast(&mut self, loc: SourceLocation, cast: &tl::Cast) {
        let lhs = self.operand(loc, cast.lhs);
        let arg = self.operand(loc, cast.arg);
        for (lhs, rhs) in lhs.iter().zip(arg.iter()) {
            self.fg.graph.add_edge(*rhs, *lhs, EdgeKind::Arg(0));
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
            self.fg.graph.add_edge(*rhs, *lhs, EdgeKind::Arg(0));
        }
    }
    fn build_dynamic_index(&mut self, loc: SourceLocation, dynamic_index: &tl::DynamicIndex) {
        let lhs = self.operand(loc, dynamic_index.lhs);
        let arg = self.operand(loc, dynamic_index.arg);
        let offset = self.operand(loc, dynamic_index.offset);
        let comp = self.fg.new_component(
            ComponentKind::DynamicIndex(DynamicIndex {
                len: dynamic_index.len,
            }),
            loc,
        );
        /*         self.fg.lhs(comp, lhs);
               self.fg.arg(comp, arg, 0);
               self.fg.offset(comp, offset);
        */
        todo!();
    }
    fn build_dynamic_splice(&mut self, loc: SourceLocation, dynamic_splice: &tl::DynamicSplice) {
        let lhs = self.operand(loc, dynamic_splice.lhs);
        let arg = self.operand(loc, dynamic_splice.arg);
        let offset = self.operand(loc, dynamic_splice.offset);
        let value = self.operand(loc, dynamic_splice.value);
        let comp = self.fg.new_component(
            ComponentKind::DynamicSplice(DynamicSplice {
                len: dynamic_splice.len,
            }),
            loc,
        );
        /*
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg, 0);
        self.fg.offset(comp, offset);
        self.fg.value(comp, value);
        */
        todo!();
    }
    fn build_index(&mut self, loc: SourceLocation, index: &tl::Index) {
        let lhs = self.operand(loc, index.lhs);
        let arg = self.operand(loc, index.arg);
        for (lhs, rhs) in lhs.iter().zip(arg.iter().skip(index.bit_range.start)) {
            self.fg.graph.add_edge(*rhs, *lhs, EdgeKind::Arg(0));
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
            let comp = self.fg.new_component(ComponentKind::Select, loc);
            self.fg.edge(comp, cond[0], EdgeKind::Selector(0));
            self.fg.edge(comp, *true_val, EdgeKind::True);
            self.fg.edge(comp, *false_val, EdgeKind::False);
            self.fg.lhs(comp, *lhs);
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
            self.fg.graph.add_edge(*rhs, *lhs, EdgeKind::Arg(0));
        }
    }
    fn build_unary(&mut self, loc: SourceLocation, unary: &tl::Unary) {
        let lhs = self.operand(loc, unary.lhs);
        let arg1 = self.operand(loc, unary.arg1);
        if is_bitwise_unary(unary.op) {
            for (lhs, rhs) in lhs.iter().zip(arg1.iter()) {
                let comp = self
                    .fg
                    .new_component(ComponentKind::Unary(Unary { op: unary.op }), loc);
                self.fg.graph.add_edge(comp, *rhs, EdgeKind::Arg(0));
                self.fg.graph.add_edge(*lhs, comp, EdgeKind::Arg(0));
            }
        } else {
            let comp = self
                .fg
                .new_component(ComponentKind::Unary(Unary { op: unary.op }), loc);
            for (ndx, (lhs, rhs)) in lhs.iter().zip(arg1.iter()).enumerate() {
                self.fg.graph.add_edge(comp, *rhs, EdgeKind::ArgBit(0, ndx));
                self.fg.graph.add_edge(*lhs, comp, EdgeKind::ArgBit(0, ndx));
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
