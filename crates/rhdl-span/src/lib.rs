use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct SpanLoc {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Path(pub Vec<String>);

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct NameValue {
    pub name: Path,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub enum Attribute {
    Path(Path),
    NameValue(NameValue),
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Meta {
    pub span: SpanLoc,
    pub attributes: Vec<Attribute>,
}

pub type MetaDB = std::collections::BTreeMap<u32, Meta>;
