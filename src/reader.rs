use bincode::deserialize;
use serde::de::DeserializeOwned;

use crate::{aligned_size_of, Error, Result};

pub struct SliceReader<'a> {
    slice: &'a [u8],
    cursor: usize,
}

impl<'a> SliceReader<'a> {
    pub(crate) fn new(slice: &'a [u8]) -> Self {
        Self { slice, cursor: 0 }
    }

    pub(crate) fn take(&mut self, len: usize) -> Result<&[u8]> {
        if self.cursor + len > self.slice.len() {
            return Err(Error::ErrUnexpectedEof);
        }
        let slice = &self.slice[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(slice)
    }

    pub(crate) fn read<T: DeserializeOwned>(&mut self) -> Result<T> {
        let len = aligned_size_of::<T>();
        let bytes = self.take(len)?;
        let val = deserialize(&bytes).map_err(Error::ErrDeserialize)?;
        Ok(val)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cursor >= self.slice.len()
    }
}
