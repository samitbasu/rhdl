use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::{
    Kind, ScopedName,
    ast::SourcePool,
    flow_graph::{ChildOutputKernelQ, EdgeKind, KernelDToChildInput, NodeKind},
    rhif::spec::Slot,
    types::path::Path,
};

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphICE {
    #[error("Missing input port for path: {path:?}")]
    MissingInputPortForPath { path: Path },
    #[error("Missing input port for index {index} and path: {path:?}")]
    MissingInputPortForIndexAndPath { index: usize, path: Path },
    #[error("Missing output port for path: {path:?} - available ports: {available:?}")]
    MissingOutputPortForPath { path: Path, available: Vec<Path> },
    #[error("Missing child flow graph for: {0}")]
    MissingChildFlowGraph(ScopedName),
}

#[derive(Error, Debug, Diagnostic)]
pub enum FlowGraphErrorKind {
    #[error("Mismatched kinds in assignment: from {from:?} to {to:?}")]
    MismatchedKindsInAssignment { from: Kind, to: Kind },
    #[error("Cannot copy atoms from {from:?} to {to:?}[{to_base_path:?}]")]
    CannotCopyAtomsToPath {
        from: Kind,
        to: Kind,
        to_base_path: Path,
    },
    #[error(
        "Cannot copy atoms from {from:?}[{from_base_path:?}] ({sub_kind:?}) to {to:?}[{to_base_path:?}] ({to_sub_kind:?})"
    )]
    CannotCopyAtomsFromPathToPath {
        from: Kind,
        to: Kind,
        from_base_path: Path,
        to_base_path: Path,
        sub_kind: Kind,
        to_sub_kind: Kind,
    },
    #[error(
        "Mismatched kinds in arguments to external function: expected {expected:?} got {actual:?}"
    )]
    MismatchedKindsInExternalFunctionCall {
        expected: Vec<Kind>,
        actual: Vec<Kind>,
    },
    #[error("Node is missing in flow graph: {slot:?} with path {path:?}")]
    MissingNode { slot: Slot, path: Path },
    #[error(
        "Mismatched return kinds in external function call: expected {expected:?} got {actual:?}"
    )]
    MismatchedReturnKindsInExternalFunctionCall { expected: Kind, actual: Kind },
}

#[derive(Debug, Error)]
#[error("Flow Graph Error")]
pub struct FlowGraphError {
    pub kind: FlowGraphErrorKind,
    pub src: SourcePool,
    pub err_span: SourceSpan,
}

impl Diagnostic for FlowGraphError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.kind.help()
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_primary_with_span(Some(self.kind.to_string()), self.err_span),
        )))
    }
}

#[derive(Error, Debug)]
#[error("Design contains a logic loop")]
pub struct LogicLoopError {
    pub errors: Vec<LoopStepError>,
}

impl Diagnostic for LogicLoopError {
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(
            "The design includes a loop of logic elements which is not allowed.  The loop includes the identified instructions.",
        ))
    }
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(self.errors.iter().map(|e| e as &dyn Diagnostic)))
    }
}

#[derive(Error, Debug)]
#[error("Loop step")]
pub struct LoopStepError {
    pub from: NodeKind,
    pub edge: EdgeKind,
    pub to: NodeKind,
    pub pool: Option<SourcePool>,
}

impl LoopStepError {
    fn node_label(&self) -> Option<miette::LabeledSpan> {
        match &self.edge {
            EdgeKind::OpCode(opcode) => {
                let span: miette::SourceSpan = opcode.object.symbols.span(opcode.opcode.loc).into();
                Some(miette::LabeledSpan::new_primary_with_span(
                    Some(format!("{:?}", opcode.opcode.op)),
                    span,
                ))
            }
            _ => None,
        }
    }
    fn node_summary(&self, node: &NodeKind) -> String {
        match node {
            NodeKind::Buffer(buffer_ref) => {
                format!("buffer {}{:?}", buffer_ref.name, buffer_ref.path)
            }
            NodeKind::Constant(constant_ref) => format!(
                "constant {}{:?}{:?}",
                constant_ref.name, constant_ref.value, constant_ref.path
            ),
            NodeKind::Slot(opcode_ref) => format!("{}{:?}", opcode_ref.slot, opcode_ref.path),
        }
    }
    fn kernel_name(&self, node: &NodeKind) -> String {
        match node {
            NodeKind::Slot(opcode_ref) => format!("({})", opcode_ref.object.name),
            _ => "".to_string(),
        }
    }
    fn edge_summary(&self, edge: &EdgeKind) -> String {
        match edge {
            EdgeKind::OpCode(_) => "".into(),
            EdgeKind::InputToKernel => "input to kernel".to_string(),
            EdgeKind::ChildOutputKernelQ(ChildOutputKernelQ {
                parent,
                child,
                path,
            }) => {
                format!("child {child} output{path:?} -> kernel {parent} Q",)
            }
            EdgeKind::KernelToOutput => "kernel to output".to_string(),
            EdgeKind::KernelDToChildInput(KernelDToChildInput {
                parent,
                child,
                path,
            }) => {
                format!("kernel {parent} to {child} input{path:?}")
            }
            EdgeKind::ClockResetToKernel => "clock/reset to kernel".to_string(),
            EdgeKind::InputForwardToChild => "input forward to child".to_string(),
            EdgeKind::OutputForwardFromChild => "output forward from child".to_string(),
            EdgeKind::ChildToChild => "child to child".to_string(),
        }
    }
    fn step_summary(&self) -> String {
        format!(
            "change propogates from {} {} -> {} {}\n{}",
            self.node_summary(&self.from),
            self.kernel_name(&self.from),
            self.node_summary(&self.to),
            self.kernel_name(&self.to),
            self.edge_summary(&self.edge)
        )
    }
}

impl Diagnostic for LoopStepError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.pool
            .as_ref()
            .map(|pool| pool as &dyn miette::SourceCode)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(self.step_summary()))
    }
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + 'a>> {
        self.node_label().map(|label| {
            Box::new(std::iter::once(label)) as Box<dyn Iterator<Item = miette::LabeledSpan>>
        })
    }
}
