use crate::Error;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Add, remove, or receive information about an IP address associated with an
/// interface.
///
/// In Linux 2.2, an interface can carry multiple IP addresses, this replaces
/// the alias device concept in 2.0. In Linux 2.2, these messages support IPv4
/// and IPv6 addresses.
///
/// See [`ifaddrmsg`.](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(default, build_fn(error = "Error"))]
pub struct InterfaceAddrMessage {
    /// Address type
    pub family: u8,
    /// Prefixlength of the address
    pub prefixlen: u8,
    /// Address flags
    pub flags: u8,
    /// Address scope
    pub scope: u8,
    /// Interface index
    pub index: u32,
}

impl InterfaceAddrMessage {
    #[must_use]
    pub fn builder() -> InterfaceAddrMessageBuilder {
        InterfaceAddrMessageBuilder::default()
    }
}
