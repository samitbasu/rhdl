pub mod builder;
pub mod component;
pub mod dot;
pub mod edge_kind;
pub mod flow_graph_impl;
mod passes;
pub use builder::build_rtl_flow_graph;
