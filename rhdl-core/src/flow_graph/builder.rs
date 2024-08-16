use std::collections::HashMap;

use crate::{
    rhif::object::SourceLocation,
    rtl::{
        object::LocatedOpCode,
        spec::{OpCode, Operand},
        Object,
    },
};

use super::{
    component::{
        Binary, Buffer, Cast, ComponentKind, Constant, DynamicIndex, DynamicSplice, Index, Splice,
        Unary,
    },
    EdgeKind, FlowGraph, FlowIx,
};

use crate::rtl::spec as tl;

pub struct FlowGraphBuilder<'a> {
    object: &'a Object,
    fg: FlowGraph,
    operand_map: HashMap<Operand, FlowIx>,
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
                bob.fg.edge(reg_operand, node, EdgeKind::Arg(0));
            }
        });
    let ret_operand = bob.operand(location, object.return_register);
    bob.fg.edge(bob.fg.output, ret_operand, EdgeKind::Arg(0));
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
    fn operand(&mut self, loc: SourceLocation, operand: Operand) -> FlowIx {
        if let Some(port) = self.operand_map.get(&operand) {
            return *port;
        }
        match operand {
            Operand::Literal(literal_id) => {
                let bs = &self.object.literals[&literal_id];
                let comp = self
                    .fg
                    .new_component(ComponentKind::Constant(Constant { bs: bs.clone() }), loc);
                self.operand_map.insert(operand, comp);
                comp
            }
            Operand::Register(register_id) => {
                let reg = self.object.register_kind[&register_id];
                let comp = self.fg.new_component(
                    ComponentKind::Buffer(Buffer {
                        kind: reg,
                        name: format!("{:?}", register_id),
                    }),
                    loc,
                );
                self.operand_map.insert(operand, comp);
                comp
            }
        }
    }
    fn build_assign(&mut self, loc: SourceLocation, assign: &tl::Assign) {
        let rhs = self.operand(loc, assign.rhs);
        let lhs = self.operand(loc, assign.lhs);
        let comp = self.fg.new_component(ComponentKind::Assign, loc);
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, rhs, 0);
    }
    fn build_binary(&mut self, loc: SourceLocation, binary: &tl::Binary) {
        let arg1 = self.operand(loc, binary.arg1);
        let arg2 = self.operand(loc, binary.arg2);
        let lhs = self.operand(loc, binary.lhs);
        let len = self.object.kind(binary.arg1).len();
        let comp = self.fg.new_component(
            ComponentKind::Binary(Binary {
                op: binary.op,
                width: len,
            }),
            loc,
        );
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg1, 0);
        self.fg.arg(comp, arg2, 1);
    }
    fn build_case(&mut self, loc: SourceLocation, case: &tl::Case) {
        let lhs = self.operand(loc, case.lhs);
        let discriminant = self.operand(loc, case.discriminant);
        let comp = self.fg.new_component(ComponentKind::Case, loc);
        self.fg.lhs(comp, lhs);
        self.fg.selector(comp, discriminant);
        for (test, value) in &case.table {
            let value = self.operand(loc, *value);
            match test {
                tl::CaseArgument::Literal(lit_id) => {
                    let bs = &self.object.literals[lit_id];
                    self.fg.case_literal(comp, value, bs.clone());
                }
                tl::CaseArgument::Wild => self.fg.case_wild(comp, value),
            }
        }
    }
    fn build_cast(&mut self, loc: SourceLocation, cast: &tl::Cast) {
        let lhs = self.operand(loc, cast.lhs);
        let arg = self.operand(loc, cast.arg);
        let comp = self.fg.new_component(
            ComponentKind::Cast(Cast {
                len: cast.len,
                signed: cast.signed,
            }),
            loc,
        );
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg, 0);
    }
    fn build_concat(&mut self, loc: SourceLocation, concat: &tl::Concat) {
        let lhs = self.operand(loc, concat.lhs);
        let args = concat
            .args
            .iter()
            .map(|x| self.operand(loc, *x))
            .collect::<Vec<_>>();
        let comp = self.fg.new_component(ComponentKind::Concat, loc);
        for (ndx, arg) in args.iter().enumerate() {
            self.fg.arg(comp, *arg, ndx);
        }
        self.fg.lhs(comp, lhs);
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
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg, 0);
        self.fg.offset(comp, offset);
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
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg, 0);
        self.fg.offset(comp, offset);
        self.fg.value(comp, value);
    }
    fn build_index(&mut self, loc: SourceLocation, index: &tl::Index) {
        let lhs = self.operand(loc, index.lhs);
        let arg = self.operand(loc, index.arg);
        let comp = self.fg.new_component(
            ComponentKind::Index(Index {
                bit_range: index.bit_range.clone(),
            }),
            loc,
        );
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg, 0);
    }
    fn build_select(&mut self, loc: SourceLocation, select: &tl::Select) {
        let lhs = self.operand(loc, select.lhs);
        let cond = self.operand(loc, select.cond);
        let true_value = self.operand(loc, select.true_value);
        let false_value = self.operand(loc, select.false_value);
        let comp = self.fg.new_component(ComponentKind::Select, loc);
        self.fg.lhs(comp, lhs);
        self.fg.edge(comp, cond, EdgeKind::Selector);
        self.fg.edge(comp, true_value, EdgeKind::True);
        self.fg.edge(comp, false_value, EdgeKind::False);
    }
    fn build_splice(&mut self, loc: SourceLocation, splice: &tl::Splice) {
        let lhs = self.operand(loc, splice.lhs);
        let orig = self.operand(loc, splice.orig);
        let value = self.operand(loc, splice.value);
        let comp = self.fg.new_component(
            ComponentKind::Splice(Splice {
                bit_range: splice.bit_range.clone(),
            }),
            loc,
        );
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, orig, 0);
        self.fg.value(comp, value);
    }
    fn build_unary(&mut self, loc: SourceLocation, unary: &tl::Unary) {
        let lhs = self.operand(loc, unary.lhs);
        let arg1 = self.operand(loc, unary.arg1);
        let comp = self
            .fg
            .new_component(ComponentKind::Unary(Unary { op: unary.op }), loc);
        self.fg.lhs(comp, lhs);
        self.fg.arg(comp, arg1, 0);
    }
}
