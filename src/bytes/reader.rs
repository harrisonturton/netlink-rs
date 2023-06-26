use super::aligned_size_of;
use crate::{Error, Result};
use bincode::deserialize;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct OwnedSliceReader {
    pub(crate) slice: Vec<u8>,
    pub(crate) cursor: usize,
}

impl OwnedSliceReader {
    pub(crate) fn new(slice: Vec<u8>) -> Self {
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
        let val = deserialize(bytes).map_err(Error::ErrDeserialize)?;
        Ok(val)
    }

    pub(crate) fn read2<'a, T: crate::generic::DeserializeOwned>(&mut self) -> Result<T> {
        let len = aligned_size_of::<T>();
        let bytes = self.take(len)?;
        let val = T::deserialize(bytes.to_vec())?;
        Ok(val)
    }

    pub(crate) fn remaining(&mut self) -> &[u8] {
        &self.slice[self.cursor..]
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cursor >= self.slice.len()
    }
}

#[derive(Debug)]
pub struct SliceReader<'a> {
    pub(crate) slice: &'a [u8],
    pub(crate) cursor: usize,
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
        let val = deserialize(bytes).map_err(Error::ErrDeserialize)?;
        Ok(val)
    }

    pub(crate) fn remaining(&mut self) -> &[u8] {
        &self.slice[self.cursor..]
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cursor >= self.slice.len()
    }
}
