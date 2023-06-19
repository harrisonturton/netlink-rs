use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::mem::{size_of, size_of_val};

#[derive(Copy, Clone)]
pub enum MessageType {
    /// No operation, message must be discarded
    Noop = 1,
    /// Error message or ACK
    Error = 2,
    /// End of a sequence of multipart messages
    Done = 3,
    /// Overrun error
    Overrun = 4,
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct NetlinkHeader {
    pub len: u32,
    pub typ: u16,
    pub flags: u16,
    pub seq: u32,
    pub pid: u32,
}

impl NetlinkHeader {
    pub fn has_type(&self, typ: MessageType) -> bool {
        self.typ == typ as u16
    }

    pub fn has_flags(&self, flags: u16) -> bool {
        self.flags & flags == flags
    }
}

#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct NetlinkMessage<T> {
    pub header: NetlinkHeader,
    pub payload: T,
}

impl<T> NetlinkMessage<T> {
    pub fn new(header: NetlinkHeader, payload: T) -> Self {
        Self { header, payload }
    }
}

impl<T: Serialize> NetlinkMessage<T> {
    pub fn builder<'a>() -> NetlinkMessageBuilder<'a, T> {
        NetlinkMessageBuilder::new()
    }
}

pub struct NetlinkMessageBuilder<'a, T: Serialize> {
    payload: Option<&'a T>,
    flags: Option<u16>,
    typ: Option<u16>,
    seq: Option<u32>,
    pid: Option<u32>,
}

impl<'a, T: Serialize> NetlinkMessageBuilder<'a, T> {
    pub fn new() -> Self {
        NetlinkMessageBuilder {
            payload: None,
            flags: None,
            typ: None,
            seq: None,
            pid: None,
        }
    }

    pub fn payload(mut self, payload: &'a T) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn flags(mut self, flags: u16) -> Self {
        self.flags = Some(flags);
        self
    }

    pub fn typ(mut self, typ: u16) -> Self {
        self.typ = Some(typ);
        self
    }

    pub fn seq(mut self, seq: u32) -> Self {
        self.seq = Some(seq);
        self
    }

    pub fn pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    pub fn build(&self) -> Result<NetlinkMessage<Vec<u8>>> {
        let payload = self
            .payload
            .map(bincode::serialize)
            .unwrap_or(Ok(Vec::new()))
            .map_err(Error::ErrSerialize)?;

        let len = aligned_size_of_val(&payload) + aligned_size_of::<NetlinkHeader>();

        let header = NetlinkHeader {
            len: len as u32,
            typ: self.typ.unwrap_or(0),
            flags: self.flags.unwrap_or(0),
            seq: self.seq.unwrap_or(0),
            pid: self.pid.unwrap_or(0),
        };

        Ok(NetlinkMessage { header, payload })
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attr {
    pub len: u16,
    pub typ: u16,
}

pub mod flags {
    /// Must be set of all request messages
    pub const REQUEST: u16 = 0x1;

    /// This message is part of a multipart message terminated by a message with
    /// type [MessageType::Done]
    pub const MULTI: u16 = 0x2;

    /// Acknowledgement of success
    pub const ACK: u16 = 0x4;

    /// Return the complete table instead of a single entry
    pub const ROOT: u16 = 0x100;

    /// Return all entries matching the criteria passed in the message content
    pub const MATCH: u16 = 0x200;

    /// Return an atomic snapshot of the table. Required the `CAP_NET_ADMIN`
    /// capability or an effective UID of 0.
    pub const ATOMIC: u16 = 0x400;

    /// Return all entries. Convenience macro, equivalent to
    /// [MessageFlags::Root] OR [MessageFlags::Match].
    pub const DUMP: u16 = ROOT | MATCH;

    /// Replace an existing object
    pub const REPLACE: u16 = 0x100;

    /// Don't replace if the object already exists (see [MessageFlags::Replace])
    pub const EXCL: u16 = 0x200;

    /// Create object if it doesn't already exist
    pub const CREATE: u16 = 0x400;

    /// Add to the end of the object list
    pub const APPEND: u16 = 0x800;
}

// Netlink pads messages to 4 bytes
const ALIGN_TO: usize = 4;

pub(crate) fn aligned_size_of<T>() -> usize {
    (size_of::<T>() + ALIGN_TO - 1) & !(ALIGN_TO - 1)
}

pub(crate) fn aligned_size_of_val<T: Sized>(val: &T) -> usize {
    (size_of_val(&val) + ALIGN_TO - 1) & !(ALIGN_TO - 1)
}
