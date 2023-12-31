use crate::{Error, Result};
use serde::Serialize;
use std::mem::size_of;

// Netlink requires that objects are serialized into buffers aligned to 4
// bytes. This trait adds support for this behaviour to every type that
// implements [`serde::Serialize`].
pub(crate) fn serialize_aligned<T: Serialize>(val: T) -> Result<Vec<u8>> {
    let mut bytes = bincode::serialize(&val).map_err(Error::ErrSerialize)?;
    let padding_len = aligned_size_of::<T>() - bytes.len();
    let mut padding = vec![0u8; padding_len];

    bytes.append(&mut padding);
    Ok(bytes)
}

// Netlink pads messages to 4 bytes
const ALIGN_TO: usize = 4;

#[must_use]
pub(crate) const fn aligned_size(val: usize) -> usize {
    (val + ALIGN_TO - 1) & !(ALIGN_TO - 1)
}

#[must_use]
pub(crate) const fn aligned_size_of<T>() -> usize {
    (size_of::<T>() + ALIGN_TO - 1) & !(ALIGN_TO - 1)
}
