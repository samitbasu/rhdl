// Every node in the netlist is one of 3 things:
// A register, which has a width and a sign flag
// A literal, which has a binary string and a sign flag
// An operation, which is an RTL op code which may include additional information
//
// For example, the add operation r2 <- r1 + r0 would be
//
// r0 -------+
//           |
//           v
// r1 ----> add ----> r2
//
// While the index operation would be r2 <- r1[3:0]
// r1 ----> index ----> r2
//          [3:0]
//
// The netlist is a directed graph of these nodes, where the edges are the operands of the operation.
// Note that the opcode contains information about

use std::{collections::HashMap, ops::Range};

use crate::{
    ast::ast_impl::{FunctionId, NodeId},
    rhif::spec::{AluBinary, AluUnary},
    rtl::{
        object::{BitString, LocatedOpCode, RegisterKind},
        spec::{LiteralId, OpCode, Operand, RegisterId},
        Object,
    },
};

use crate::rtl::spec as tl;

type NodeIx = petgraph::graph::NodeIndex<u32>;

#[derive(Clone, Debug)]
pub enum Node {
    Register(Register),
    Literal(Literal),
    Operation(Operation),
}

#[derive(Clone, Debug)]
pub struct Register {
    pub fn_id: FunctionId,
    pub id: RegisterId,
    pub kind: RegisterKind,
}

#[derive(Clone, Debug)]
pub struct Literal {
    pub fn_id: FunctionId,
    pub id: LiteralId,
    pub value: BitString,
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub fn_id: FunctionId,
    pub id: NodeId,
    pub kind: OperationKind,
}

#[derive(Clone, Debug)]
pub enum OperationKind {
    AsBits(Cast),
    Assign(Assign),
    AsSigned(Cast),
    Binary(Binary),
    Case(Case),
    Concat(Concat),
    DynamicIndex(DynamicIndex),
    DynamicSplice(DynamicSplice),
    Index(Index),
    Select(Select),
    Splice(Splice),
    Unary(Unary),
}

#[derive(Clone, Debug)]
pub struct Cast {
    pub lhs: NodeIx,
    pub arg: NodeIx,
    pub len: usize,
}

#[derive(Clone, Debug)]
pub struct Assign {
    pub lhs: NodeIx,
    pub rhs: NodeIx,
}

#[derive(Clone, Debug)]
pub struct Binary {
    pub op: AluBinary,
    pub lhs: NodeIx,
    pub arg1: NodeIx,
    pub arg2: NodeIx,
}

#[derive(Clone, Debug)]
pub enum CaseArgument {
    Literal(NodeIx),
    Wild,
}

#[derive(Clone, Debug)]
pub struct Case {
    pub lhs: NodeIx,
    pub discriminant: NodeIx,
    pub table: Vec<(CaseArgument, NodeIx)>,
}

#[derive(Clone, Debug)]
pub struct Concat {
    pub lhs: NodeIx,
    pub args: Vec<NodeIx>,
}

#[derive(Clone, Debug)]
pub struct DynamicIndex {
    pub lhs: NodeIx,
    pub arg: NodeIx,
    pub offset: NodeIx,
    pub len: usize,
}

#[derive(Clone, Debug)]
pub struct DynamicSplice {
    pub lhs: NodeIx,
    pub arg: NodeIx,
    pub offset: NodeIx,
    pub len: usize,
    pub value: NodeIx,
}

#[derive(Clone, Debug)]
pub struct Index {
    pub lhs: NodeIx,
    pub arg: NodeIx,
    pub bit_range: Range<usize>,
}

#[derive(Clone, Debug)]
pub struct Splice {
    pub lhs: NodeIx,
    pub orig: NodeIx,
    pub bit_range: Range<usize>,
    pub value: NodeIx,
}

#[derive(Clone, Debug)]
pub struct Select {
    pub lhs: NodeIx,
    pub cond: NodeIx,
    pub true_value: NodeIx,
    pub false_value: NodeIx,
}

#[derive(Clone, Debug)]
pub struct Unary {
    pub op: AluUnary,
    pub lhs: NodeIx,
    pub arg1: NodeIx,
}

struct NetListBuilder<'a> {
    object: &'a Object,
    operand_to_node: HashMap<Operand, NodeIx>,
    graph: petgraph::graph::DiGraph<Node, ()>,
}

impl<'a> NetListBuilder<'a> {
    fn operand(&mut self, op: Operand) -> NodeIx {
        if let Some(nix) = self.operand_to_node.get(&op) {
            return *nix;
        }
        match op {
            Operand::Literal(lit) => {
                let node = Node::Literal(Literal {
                    fn_id: self.object.fn_id,
                    id: lit,
                    value: self.object.literals[&lit].clone(),
                });
                let nix = self.graph.add_node(node);
                self.operand_to_node.insert(op, nix);
                nix
            }
            Operand::Register(reg) => {
                let node = Node::Register(Register {
                    fn_id: self.object.fn_id,
                    id: reg,
                    kind: self.object.register_kind[&reg],
                });
                let nix = self.graph.add_node(node);
                self.operand_to_node.insert(op, nix);
                nix
            }
        }
    }
    fn build_as_bits(&mut self, id: NodeId, cast: &tl::Cast) {
        let lhs = self.operand(cast.lhs);
        let arg = self.operand(cast.arg);
        let node = Node::Operation(Operation {
            fn_id: self.object.fn_id,
            id,
            kind: OperationKind::AsBits(Cast {
                lhs,
                arg,
                len: cast.len,
            }),
        });
        let nix = self.graph.add_node(node);
        self.graph.add_edge(lhs, nix, ());
        self.graph.add_edge(arg, nix, ());
    }
    fn build_assign(&mut self, id: NodeId, assign: &tl::Assign) {
        let lhs = self.operand(assign.lhs);
        let rhs = self.operand(assign.rhs);
        let node = Node::Operation(Operation {
            fn_id: self.object.fn_id,
            id,
            kind: OperationKind::Assign(Assign { lhs, rhs }),
        });
        let nix = self.graph.add_node(node);
        self.graph.add_edge(lhs, nix, ());
        self.graph.add_edge(rhs, nix, ());
    }
    fn build_as_signed(&mut self, id: NodeId, cast: &tl::Cast) {
        let lhs = self.operand(cast.lhs);
        let arg = self.operand(cast.arg);
        let node = Node::Operation(Operation {
            fn_id: self.object.fn_id,
            id,
            kind: OperationKind::AsSigned(Cast {
                lhs,
                arg,
                len: cast.len,
            }),
        });
        let nix = self.graph.add_node(node);
        self.graph.add_edge(lhs, nix, ());
        self.graph.add_edge(arg, nix, ());
    }
    fn build_binary(&mut self, id: NodeId, binary: &tl::Binary) {
        let lhs = self.operand(binary.lhs);
        let arg1 = self.operand(binary.arg1);
        let arg2 = self.operand(binary.arg2);
        let node = Node::Operation(Operation {
            fn_id: self.object.fn_id,
            id,
            kind: OperationKind::Binary(Binary {
                op: binary.op,
                lhs,
                arg1,
                arg2,
            }),
        });
        let nix = self.graph.add_node(node);
        self.graph.add_edge(lhs, nix, ());
        self.graph.add_edge(arg1, nix, ());
        self.graph.add_edge(arg2, nix, ());
    }
    fn build_case(&mut self, id: NodeId, case: &tl::Case) {
        let lhs = self.operand(case.lhs);
        let discriminant = self.operand(case.discriminant);
        let table = case
            .table
            .iter()
            .map(|(arg, node)| {
                (
                    match arg {
                        tl::CaseArgument::Literal(lit) => {
                            CaseArgument::Literal(self.operand(Operand::Literal(*lit)))
                        }
                        tl::CaseArgument::Wild => CaseArgument::Wild,
                    },
                    self.operand(*node),
                )
            })
            .collect();
        let node = Node::Operation(Operation {
            fn_id: self.object.fn_id,
            id,
            kind: OperationKind::Case(Case {
                lhs,
                discriminant,
                table: table.clone(),
            }),
        });
        let nix = self.graph.add_node(node);
        self.graph.add_edge(lhs, nix, ());
        self.graph.add_edge(discriminant, nix, ());
        for (_, node) in &table {
            self.graph.add_edge(*node, nix, ());
        }
    }
    fn opcode(&mut self, lop: &LocatedOpCode) {
        let id = lop.id;
        match &lop.op {
            OpCode::AsBits(cast) => self.build_as_bits(id, cast),
            OpCode::Assign(assign) => self.build_assign(id, assign),
            OpCode::AsSigned(cast) => self.build_as_signed(id, cast),
            OpCode::Binary(binary) => self.build_binary(id, binary),
            OpCode::Case(case) => self.build_case(id, case),
            OpCode::Concat(concat) => self.build_concat(id, concat),
            OpCode::DynamicIndex(dynamic_index) => self.build_dynamic_index(id, dynamic_index),
            OpCode::DynamicSplice(dynamic_splice) => self.build_dynamic_splice(id, dynamic_splice),
            OpCode::Index(index) => self.build_index(id, index),
            OpCode::Select(select) => self.build_select(id, select),
            OpCode::Splice(splice) => self.build_splice(id, splice),
            OpCode::Unary(unary) => self.build_unary(id, unary),
            _ => {}
        }
    }
}

pub struct NetList {
    pub graph: petgraph::graph::DiGraph<Node, ()>,
}

pub fn build_netlist(object: &Object) -> NetList {}
