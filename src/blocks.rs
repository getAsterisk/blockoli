use serde::Serialize;

/// Represents a code block with its associated vector embedding.
#[derive(Debug, Clone)]
pub struct EmbeddedBlock {
    /// The code block itself.
    pub block: asterisk::block::Block,

    /// The vector embedding of the code block.
    pub vectors: Vec<f32>,
}

/// Represents a set of related code blocks.
#[derive(Debug, Serialize, Clone)]
pub struct BlockSet {
    /// The source file path where the code blocks originated from.
    pub source_file: String,

    /// The name of the function associated with the code blocks, if applicable.
    pub function_name: Option<String>,

    /// The actual code content of the blocks.
    pub code: String,

    /// A string representation of the incoming function calls to the code blocks.
    pub incoming_calls: String,

    /// A string representation of the outgoing function calls from the code blocks.
    pub outgoing_calls: String,
}
