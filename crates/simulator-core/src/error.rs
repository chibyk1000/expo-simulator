use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Node with id {0} not found")]
    NodeNotFound(u64),

    #[error("Layout computation error: {0}")]
    LayoutError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal core error: {0}")]
    Internal(String),
}
