use std::sync::Arc;

use miette::Diagnostic;
use thiserror::Error;

use crate::{
    circuit::schematic::{CanonicalPath, Port, PortId, Schematic, SchematicId},
    types::path::Path,
};

#[derive(Error, Debug, Diagnostic)]
pub enum SchematicICE {
    #[error("Schematic not available for circuit {name}")]
    SchematicNotAvailable { name: String },
    #[error("Missing Port in Schematic: {path:?}, available ports: {avail:#?}")]
    MissingPortInSchematic {
        path: CanonicalPath,
        avail: Vec<Port>,
    },
    #[error("Unable to canonicalize path {path:?} against kind {kind:?}")]
    UnableToCanonicalizePath { path: Path, kind: crate::Kind },
    #[error("Schematic block {block_name} has an unconnected input port {port:?}")]
    UnconnectedInputPort { block_name: String, port: Port },
    #[error("Schematic has a logic loop")]
    LogicLoop {
        schematic: Arc<Schematic>,
        logic_loop: Vec<PortId>,
    },
    #[error("Duplicate port id: {id:?} in schematic {sid:?}")]
    DuplicatePortId { id: PortId, sid: SchematicId },
    #[error("Duplicate schematic id: {id:?}")]
    DuplicateSchematicId { id: SchematicId },
}
