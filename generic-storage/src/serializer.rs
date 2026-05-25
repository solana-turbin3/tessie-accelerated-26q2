use borsh::{BorshDeserialize, BorshSerialize};
use serde::{de::DeserializeOwned, Serialize};
use wincode::{config::DefaultConfig, SchemaRead, SchemaWrite};

use crate::StorageError;

pub trait Serializer {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, StorageError>
    where
        T: StorageData;

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, StorageError>
    where
        T: StorageData;
}

pub trait StorageData:
    BorshSerialize
    + BorshDeserialize
    + Serialize
    + DeserializeOwned
    + SchemaWrite<DefaultConfig, Src = Self>
    + for<'de> SchemaRead<'de, DefaultConfig, Dst = Self>
{
}

impl<T> StorageData for T where
    T: BorshSerialize
        + BorshDeserialize
        + Serialize
        + DeserializeOwned
        + SchemaWrite<DefaultConfig, Src = Self>
        + for<'de> SchemaRead<'de, DefaultConfig, Dst = Self>
{
}

#[derive(Debug, Clone, Copy)]
pub struct Borsh;

#[derive(Debug, Clone, Copy)]
pub struct Json;

#[derive(Debug, Clone, Copy)]
pub struct Wincode;

impl Serializer for Borsh {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, StorageError>
    where
        T: StorageData,
    {
        borsh::to_vec(value).map_err(|e| StorageError::Serialize(e.to_string()))
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, StorageError>
    where
        T: StorageData,
    {
        T::try_from_slice(bytes).map_err(|e| StorageError::Deserialize(e.to_string()))
    }
}

impl Serializer for Json {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, StorageError>
    where
        T: StorageData,
    {
        serde_json::to_vec(value).map_err(|e| StorageError::Serialize(e.to_string()))
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, StorageError>
    where
        T: StorageData,
    {
        serde_json::from_slice(bytes).map_err(|e| StorageError::Deserialize(e.to_string()))
    }
}

impl Serializer for Wincode {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, StorageError>
    where
        T: StorageData,
    {
        wincode::serialize(value).map_err(|e| StorageError::Serialize(e.to_string()))
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, StorageError>
    where
        T: StorageData,
    {
        wincode::deserialize(bytes).map_err(|e| StorageError::Deserialize(e.to_string()))
    }
}
