use crate::bytes::serialize_aligned;
use crate::Result;
use serde::{Deserialize, Serialize};

/// Core message types for Netlink packets.
#[repr(u16)]
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

impl From<MessageType> for u16 {
    fn from(value: MessageType) -> Self {
        value as u16
    }
}

/// Flags for configuring Netlink messages.
pub enum Flag {
    /// Must be set of all request messages
    Request,
    /// This message is part of a multipart message terminated by a message with
    /// type [MessageType::Done]
    Multi,
    /// Acknowledgement of success
    Ack,
    /// Return the complete table instead of a single entry
    Root,
    /// Return all entries matching the criteria passed in the message content
    Match,
    /// Return an atomic snapshot of the table. Required the `CAP_NET_ADMIN`
    /// capability or an effective UID of 0.
    Atomic,
    /// Return all entries. Convenience macro, equivalent to [Root] OR [Match].
    Dump,
    /// Replace an existing object
    Replace,
    /// Don't replace if the object already exists (see [Flag::Replace])
    Excl,
    /// Create object if it doesn't already exist
    Create,
    /// Add to the end of the object list
    Append,
}

impl From<Flag> for u16 {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Request => 0x1,
            Flag::Multi => 0x2,
            Flag::Ack => 0x4,
            Flag::Root => 0x100,
            Flag::Match => 0x200,
            Flag::Atomic => 0x400,
            Flag::Dump => 0x100 | 0x200,
            Flag::Replace => 0x100,
            Flag::Excl => 0x200,
            Flag::Create => 0x400,
            Flag::Append => 0x800,
        }
    }
}

impl std::ops::BitOr for Flag {
    type Output = u16;

    fn bitor(self, rhs: Self) -> Self::Output {
        let lhs: u16 = self.into();
        let rhs: u16 = rhs.into();
        lhs | rhs
    }
}

/// Descriptor of a Netlink header.
///
/// A complete
/// [`nlmsghdr`](https://man7.org/linux/man-pages/man7/netlink.7.html) has
/// additional `len`, `seq` and `pid` fields. These are calculated by
/// [`NetlinkStream`] when the message is sent.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct NetlinkHeaderDescriptor {
    pub(crate) typ: u16,
    pub(crate) flags: u16,
}

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct NetlinkHeader {
    pub(crate) len: u32,
    pub(crate) typ: u16,
    pub(crate) flags: u16,
    pub(crate) seq: u32,
    pub(crate) pid: u32,
}

impl NetlinkHeader {
    pub(crate) fn into_descriptor(self) -> NetlinkHeaderDescriptor {
        NetlinkHeaderDescriptor {
            typ: self.typ,
            flags: self.flags,
        }
    }

    /// Check if the header has a certain type.
    ///
    /// See
    /// [`nlmsg_type`](https://man7.org/linux/man-pages/man7/netlink.7.html).
    pub fn has_type(&self, typ: MessageType) -> bool {
        self.typ == typ.into()
    }

    /// Check if the header has a certain flag set.
    ///
    /// See ///
    /// [`nlmsg_flags`](https://man7.org/linux/man-pages/man7/netlink.7.html).
    pub fn has_flags(&self, flags: Flag) -> bool {
        let flags: u16 = flags.into();
        self.flags & flags == flags
    }
}

/// The message that is sent to the kernel. This should be constructed using
/// [`NetlinkMessageBuilder`] to ensure the proper byte alignment and header
/// fields are set.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct NetlinkMessage {
    pub header: NetlinkHeaderDescriptor,
    pub payload: Vec<u8>,
}

impl NetlinkMessage {
    pub(crate) fn new(header: NetlinkHeaderDescriptor, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Build a [`NetlinkMessage`] using the safe builder. This will make sure
    /// the length and payloads are aligned to the proper byte offsets.
    #[must_use]
    pub fn builder() -> NetlinkMessageBuilder {
        NetlinkMessageBuilder::new()
    }
}

/// Safe builder for [`NetlinkMessage`].
///
/// This allows you to constructs Netlink messages with the proper byte
/// alignment and header values.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct NetlinkMessageBuilder {
    typ: u16,
    flags: u16,
    payload: Vec<u8>,
}

impl NetlinkMessageBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// See `nlmsg_type` in the [netlink(7)
    /// manpage](https://man7.org/linux/man-pages/man7/netlink.7.html).
    #[must_use]
    pub fn typ<I: Into<u16>>(mut self, typ: I) -> Self {
        self.typ = typ.into();
        self
    }

    /// See `nlmsg_flags` in the [netlink(7)
    /// manpage](https://man7.org/linux/man-pages/man7/netlink.7.html).
    #[must_use]
    pub fn flags(mut self, flags: u16) -> Self {
        self.flags = flags;
        self
    }

    /// Append a type to the message payload. This will serialize `T` into a
    /// `Vec<u8>>` padded to a 4 byte alignment.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] when `payload` cannot be serialized.
    pub fn append<T: Serialize>(mut self, payload: T) -> Result<Self> {
        let mut bytes = serialize_aligned(payload)?;
        self.payload.append(&mut bytes);
        Ok(self)
    }

    pub fn append_slice(mut self, slice: &mut Vec<u8>) -> Result<Self> {
        self.payload.append(slice);
        Ok(self)
    }

    /// Consume the builder and  get the [`NetlinkMessage`].
    #[must_use]
    pub fn build(self) -> NetlinkMessage {
        NetlinkMessage {
            header: NetlinkHeaderDescriptor {
                typ: self.typ,
                flags: self.flags,
            },
            payload: self.payload,
        }
    }
}
