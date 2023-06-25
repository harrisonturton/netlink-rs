use crate::{Error, Result};
use bincode::deserialize;
use std::mem::size_of;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub(crate) fn deserialize_i8(payload: &[u8]) -> Result<i8> {
    let bytes: [u8; 1] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i8::from_le_bytes(bytes))
}

pub(crate) fn deserialize_i16(payload: &[u8]) -> Result<i16> {
    let bytes: [u8; 2] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i16::from_le_bytes(bytes))
}

pub(crate) fn deserialize_i32(payload: &[u8]) -> Result<i32> {
    let bytes: [u8; 4] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i32::from_le_bytes(bytes))
}

pub(crate) fn deserialize_u32(payload: &[u8]) -> Result<u32> {
    let bytes: [u8; 4] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(u32::from_le_bytes(bytes))
}

pub(crate) fn deserialize_ascii(payload: &[u8]) -> String {
    String::from_utf8_lossy(payload)
        .trim_matches(char::from(0))
        .to_owned()
}

pub(crate) fn deserialize_ip_addr(payload: &[u8]) -> Result<IpAddr> {
    if payload.len() == size_of::<Ipv6Addr>() {
        deserialize::<Ipv6Addr>(payload)
            .map(IpAddr::V6)
            .map_err(Error::ErrDeserialize)
    } else {
        deserialize::<Ipv4Addr>(payload)
            .map(IpAddr::V4)
            .map_err(Error::ErrDeserialize)
    }
}
