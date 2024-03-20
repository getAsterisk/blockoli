use serde::Serialize;

#[derive(Debug, Clone)]
pub struct EmbeddedBlock {
    pub block: asterisk::block::Block,
    pub vectors: Vec<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BlockSet {
    pub source_file: String,
    pub function_name: Option<String>,
    pub code: String,
    pub incoming_calls: String,
    pub outgoing_calls: String,
}
