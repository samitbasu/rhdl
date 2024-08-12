use component::{Buffer, Cast, ComponentKind};
use petgraph::{Directed, Graph};

use crate::{
    rhif::object::SourceLocation,
    rtl::object::{BitString, RegisterKind},
};

pub mod builder;
pub mod component;
pub mod dot;

pub use builder::build_rtl_flow_graph;

#[derive(Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub location: SourceLocation,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ComponentKind::Assign => write!(f, "="),
            ComponentKind::Buffer(buffer) => write!(f, "{}", buffer.name),
            ComponentKind::Binary(binary) => write!(f, "{:?}", binary.op),
            ComponentKind::BlackBox => write!(f, "BB"),
            ComponentKind::Case => write!(f, "Case"),
            ComponentKind::Cast(cast) => {
                if cast.signed {
                    write!(f, "as s{}", cast.len)
                } else {
                    write!(f, "as b{}", cast.len)
                }
            }
            ComponentKind::Concat => write!(f, "{{}}"),
            ComponentKind::Constant(constant) => write!(f, "{:?}", constant.bs),
            ComponentKind::DynamicIndex(dynamic_index) => write!(f, "[[{}]]", dynamic_index.len),
            ComponentKind::DynamicSplice(dynamic_splice) => write!(f, "//{}//", dynamic_splice.len),
            ComponentKind::Index(index) => {
                write!(f, "{}..{}", index.bit_range.start, index.bit_range.end)
            }
            ComponentKind::Select => write!(f, "?"),
            ComponentKind::Splice(splice) => {
                write!(f, "/{}..{}/", splice.bit_range.start, splice.bit_range.end)
            }
            ComponentKind::Unary(unary) => write!(f, "{:?}", unary.op),
        }
    }
}

type FlowIx = petgraph::graph::NodeIndex;

#[derive(Debug, Clone, Default)]
pub struct FlowGraph {
    graph: Graph<Component, EdgeKind, Directed>,
    inputs: Vec<FlowIx>,
    output: FlowIx,
}

impl FlowGraph {
    fn buffer(&mut self, kind: RegisterKind, name: &str, location: SourceLocation) -> FlowIx {
        self.graph.add_node(Component {
            kind: ComponentKind::Buffer(Buffer {
                kind,
                name: name.into(),
            }),
            location,
        })
    }
    fn new_component(&mut self, kind: ComponentKind, location: SourceLocation) -> FlowIx {
        self.graph.add_node(Component { kind, location })
    }
    fn lhs(&mut self, component: FlowIx, lhs: FlowIx) {
        self.graph.add_edge(component, lhs, EdgeKind::Output);
    }
    fn arg(&mut self, component: FlowIx, arg: FlowIx, index: usize) {
        self.graph.add_edge(arg, component, EdgeKind::Arg(index));
    }
    fn offset(&mut self, component: FlowIx, offset: FlowIx) {
        self.graph
            .add_edge(offset, component, EdgeKind::DynamicOffset);
    }
    fn value(&mut self, component: FlowIx, value: FlowIx) {
        self.graph.add_edge(value, component, EdgeKind::Splice);
    }
    fn edge(&mut self, component: FlowIx, source: FlowIx, kind: EdgeKind) {
        self.graph.add_edge(source, component, kind);
    }
    fn selector(&mut self, component: FlowIx, selector: FlowIx) {
        self.graph.add_edge(selector, component, EdgeKind::Selector);
    }
    fn case_literal(&mut self, component: FlowIx, case: FlowIx, literal: BitString) {
        self.graph
            .add_edge(case, component, EdgeKind::CaseLiteral(literal));
    }
    fn case_wild(&mut self, component: FlowIx, case: FlowIx) {
        self.graph.add_edge(case, component, EdgeKind::CaseWild);
    }
}

#[derive(Clone)]
pub enum EdgeKind {
    Arg(usize),
    Selector,
    True,
    False,
    DynamicOffset,
    Splice,
    CaseLiteral(BitString),
    CaseWild,
    Output,
}

impl std::fmt::Debug for EdgeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arg(arg0) => write!(f, "a{}", arg0),
            Self::Selector => write!(f, "sel"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::DynamicOffset => write!(f, "dyn_offset"),
            Self::Splice => write!(f, "splice"),
            Self::CaseLiteral(arg0) => write!(f, "{:?}", arg0),
            Self::CaseWild => write!(f, "_"),
            Self::Output => Ok(()),
        }
    }
}
