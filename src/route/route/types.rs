use crate::bytes::{deserialize_i16, deserialize_i32, deserialize_i8, deserialize_ip_addr};
use crate::{Error, Result};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::net::IpAddr;

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
pub type RouteAttrHeader = AttrHeader<RouteAttrType>;
// pub struct RouteAttrHeader {
//     pub len: u16,
//     pub typ: RouteAttrType,
// }

#[repr(C)]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct AttrHeader<T> {
    pub len: u16,
    pub typ: T,
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

/// Strongly-typed [`RouteAttr`].
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
    NewDest(IpAddr),
    Pref(i8),
    EncapType(i16),
    Encap(Vec<u8>),
    Expires(i32),
}

/// Statistics about a link.
///
/// See [rtnl_link_stats](https://github.com/torvalds/linux/blob/master/tools/include/uapi/linux/if_link.h).
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
    pub(crate) fn deserialize(typ: RouteAttrType, payload: &[u8]) -> Result<Self> {
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
                deserialize_ip_addr(payload).map(Self::NewDest)
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
