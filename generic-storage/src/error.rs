use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("no data stored")]
    NoData,

    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("deserialization error: {0}")]
    Deserialize(String),
}
