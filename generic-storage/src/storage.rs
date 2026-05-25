use std::marker::PhantomData;

use crate::{Serializer, StorageData, StorageError};

pub struct Storage<T, S> {
    serializer: S,
    data: Option<Vec<u8>>,
    _marker: PhantomData<T>,
}

impl<T, S> Storage<T, S>
where
    T: StorageData,
    S: Serializer,
{
    pub fn new(serializer: S) -> Self {
        Self {
            serializer,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn save(&mut self, value: &T) -> Result<(), StorageError> {
        let bytes = self.serializer.to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }

    pub fn load(&self) -> Result<T, StorageError> {
        let bytes = self.data.as_ref().ok_or(StorageError::NoData)?;
        self.serializer.from_bytes(bytes)
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}
