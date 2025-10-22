// Serialization helpers for bincode operations

#![allow(clippy::result_large_err)]

use crate::errors::CoreError;

/// Serialize a value to bytes using bincode
pub fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
    bincode::serialize(value).map_err(|e| CoreError::Dataflow(e.to_string()))
}

/// Deserialize bytes to a value using bincode
pub fn deserialize<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, CoreError> {
    bincode::deserialize(data).map_err(|e| CoreError::Dataflow(e.to_string()))
}
