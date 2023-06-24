use std::{
    mem::size_of,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use crate::{Error, Result};
use bincode::deserialize;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Types of route messages.
#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
pub enum RouteMessageType {
    // Link
    NewLink = 16,
    DelLink = 17,
    GetLink = 18,
    SetLink = 19,
    // Addr
    NewAddr = 20,
    DelAddr = 21,
    GetAddr = 22,
    // Route
    NewRoute = 24,
    DelRoute = 25,
    GetRoute = 26,
}

impl From<RouteMessageType> for u16 {
    fn from(value: RouteMessageType) -> Self {
        value as u16
    }
}

/// Header of messages to create, remove or get information about specific
/// network interface. Includes real and virtual interfaces.
///
/// See [`ifaddrmsg`.](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(default, build_fn(error = "Error"))]
pub struct InterfaceInfoMessage {
    /// AF_UNSPEC
    pub family: u8,
    /// Device type
    pub typ: u16,
    /// Interface index
    pub index: i32,
    /// Device flags.
    //let index = /
    /// See
    /// [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html)
    pub flags: u32,
    // Change mask. This value is constant. See
    // https://man7.org/linux/man-pages/man7/rtnetlink.7.html
    #[builder(setter(skip))]
    #[builder(default = "0xFFFFFFFF")]
    pub change: u32,
}

impl InterfaceInfoMessage {
    #[must_use]
    pub fn builder() -> InterfaceInfoMessageBuilder {
        InterfaceInfoMessageBuilder::default()
    }
}

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

/// Header for messages that create, delete or receive information about a
/// network route.
///
/// See [`rtmsg`](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(default, build_fn(error = "Error"))]
pub struct RouteMessage {
    pub family: u8,
    pub dst_len: u8,
    pub src_len: u8,
    pub tos: u8,
    pub table: u8,
    pub protocol: u8,
    pub scope: u8,
    pub typ: u8,
    pub flags: u8,
}

impl RouteMessage {
    #[must_use]
    pub fn builder() -> RouteMessageBuilder {
        RouteMessageBuilder::default()
    }
}

/// Attribute of a request or response. See [`RouteAttrValue`] to understand how
/// to interpret the data pointed at by this header.
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RouteAttrHeader {
    pub len: u16,
    pub typ: RouteAttrType,
}

/// A MAC or Ethernet address.
pub type HardwareAddr = [u8; 6];

/// Type of the route attribute. This determines the type of the [`RouteAttr`].
#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
pub enum RouteAttrType {
    Unspec = 0,
    Dest = 1,
    Source = 2,
    InputInterfaceIndex = 3,
    OutputInterfaceIndex = 4,
    Gateway = 5,
    Priority = 6,
    PreferredSourceAddr = 7,
    Metrics = 8,
    Multipath = 9,
    ProtoInfo = 10,
    Flow = 11,
    CacheInfo = 12,
    Session = 13,
    MpAlgo = 14,
    Table = 15,
    Mark = 16,
    MfcStats = 17,
    Via = 18,
    NewDest = 19,
    Pref = 20,
    EncapType = 21,
    Encap = 22,
    Expires = 23,
}

/// Strongly-typed [`RouteAttr`]. This is parsed
#[derive(PartialEq, Clone, Debug)]
pub enum RouteAttrValue {
    Unspec,
    Dest(IpAddr),
    Source(IpAddr),
    InputInterfaceIndex(i32),
    OutputInterfaceIndex(i32),
    Gateway(IpAddr),
    Priority(i32),
    PreferredSourceAddr(IpAddr),
    Metrics(i32),
    // Unknown
    Multipath(Vec<u8>), // unknown
    // No longer use
    ProtoInfo(Vec<u8>), // No longer used
    Flow(i32),
    // rta_info
    CacheInfo(Vec<u8>),
    // No longer used
    Session(Vec<u8>),
    // No longer used
    MpAlgo(Vec<u8>),
    Table(i32),
    Mark(i32),
    // mfc_stats
    MfcStats(Vec<u8>),
    // rt_via
    Via(Vec<u8>),
    NewDest([u8; 4]),
    Pref(i8),
    EncapType(i16),
    Encap(Vec<u8>),
    Expires(i32),
}

/// Statistics about a link.
///
/// See
/// [rtnl_link_stats.](https://github.com/torvalds/linux/blob/master/tools/include/uapi/linux/if_link.h)
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(default, build_fn(error = "Error"))]
pub struct LinkStats {
    pub rx_packets: u32,
    pub tx_packets: u32,
    pub rx_bytes: u32,
    pub tx_bytes: u32,
    pub rx_errors: u32,
    pub tx_errors: u32,
    pub rx_dropped: u32,
    pub tx_dropped: u32,
    pub multicast: u32,
    pub collisions: u32,
    pub rx_length_errors: u32,
    pub rx_over_errors: u32,
    pub rx_crc_errors: u32,
    pub rx_frame_errors: u32,
    pub rx_fifo_errors: u32,
    pub rx_missed_errors: u32,
    pub tx_aborted_errors: u32,
    pub tx_carrier_errors: u32,
    pub tx_fifo_errors: u32,
    pub tx_heartbeat_errors: u32,
    pub tx_window_errors: u32,
    pub rx_compressed: u32,
    pub tx_compressed: u32,
}

#[rustfmt::skip]
impl RouteAttrValue {
    pub(crate) fn from(typ: RouteAttrType, payload: &[u8]) -> Result<Self> {
        match typ {
            RouteAttrType::Unspec => {
                Ok(Self::Unspec)
            }
            RouteAttrType::Dest => {
                deserialize_ip_addr(payload).map(Self::Dest)
            }
            RouteAttrType::Source => {
                deserialize_ip_addr(payload).map(Self::Source)
            }
            RouteAttrType::InputInterfaceIndex => {
                deserialize_i32(payload).map(Self::InputInterfaceIndex)
            }
            RouteAttrType::OutputInterfaceIndex => {
                deserialize_i32(payload).map(Self::OutputInterfaceIndex)
            }
            RouteAttrType::Gateway => {
                deserialize_ip_addr(payload).map(Self::Gateway)
            }
            RouteAttrType::Priority => {
                deserialize_i32(payload).map(Self::Priority)
            }
            RouteAttrType::PreferredSourceAddr => {
                deserialize_ip_addr(payload).map(Self::PreferredSourceAddr)
            }
            RouteAttrType::Metrics => {
                deserialize_i32(payload).map(Self::Metrics)
            }
            RouteAttrType::Multipath => {
                Ok(Self::Multipath(payload.to_vec()))
            }
            RouteAttrType::ProtoInfo => {
                Ok(Self::ProtoInfo(payload.to_vec()))
            }
            RouteAttrType::Flow => {
                deserialize_i32(payload).map(Self::Flow)
            }
            RouteAttrType::CacheInfo => {
                Ok(Self::CacheInfo(payload.to_vec()))
            }
            RouteAttrType::Session => {
                Ok(Self::Session(payload.to_vec()))
            }
            RouteAttrType::MpAlgo => {
                Ok(Self::MpAlgo(payload.to_vec()))
            }
            RouteAttrType::Table => {
                deserialize_i32(payload).map(Self::Table)
            }
            RouteAttrType::Mark => {
                deserialize_i32(payload).map(Self::Mark)
            }
            RouteAttrType::MfcStats => {
                Ok(Self::MfcStats(payload.to_vec()))
            }
            RouteAttrType::Via => {
                Ok(Self::Via(payload.to_vec()))
            }
            RouteAttrType::NewDest => {
                deserialize_quad(payload).map(Self::NewDest)
            }
            RouteAttrType::Pref => {
                deserialize_i8(payload).map(Self::Pref)
            }
            RouteAttrType::EncapType => {
                deserialize_i16(payload).map(Self::EncapType)
            }
            RouteAttrType::Encap => {
                Ok(Self::Encap(payload.to_vec()))
            }
            RouteAttrType::Expires => {
                deserialize_i32(payload).map(Self::Expires)
            }
        }
    }
}

fn deserialize_i8(payload: &[u8]) -> Result<i8> {
    let bytes: [u8; 1] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i8::from_le_bytes(bytes))
}

fn deserialize_i16(payload: &[u8]) -> Result<i16> {
    let bytes: [u8; 2] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i16::from_le_bytes(bytes))
}

fn deserialize_i32(payload: &[u8]) -> Result<i32> {
    let bytes: [u8; 4] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(i32::from_le_bytes(bytes))
}

fn deserialize_quad(payload: &[u8]) -> Result<[u8; 4]> {
    let bytes: [u8; 4] = payload.try_into().map_err(|_| Error::ErrUnexpectedEof)?;
    Ok(bytes)
}

fn deserialize_ip_addr(payload: &[u8]) -> Result<IpAddr> {
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
