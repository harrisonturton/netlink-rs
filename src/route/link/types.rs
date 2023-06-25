use std::net::IpAddr;

use crate::{
    bytes::{deserialize_ascii, deserialize_ip_addr, deserialize_u32},
    Error, Result,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Header of messages to create, remove or get information about specific
/// network interface. Includes real and virtual interfaces.
///
/// See [`ifaddrmsg`.](https://man7.org/linux/man-pages/man7/rtnetlink.7.html)
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug, Default, Builder, Serialize, Deserialize)]
#[builder(default, build_fn(error = "Error"))]
pub struct InterfaceInfoMessage {
    /// AF_UNSPEC
    pub family: u8,
    /// Device type
    pub typ: u16,
    /// Interface index
    pub index: i32,
    /// Device flags.
    /// See [`netdevice(7)`](https://man7.org/linux/man-pages/man7/netdevice.7.html)
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

/// Attribute of a request or response. See [`LinkAttrValue`] to understand how
/// to interpret the data pointed at by this header.
#[repr(C)]
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LinkAttrHeader {
    pub len: u16,
    pub typ: LinkAttrType,
}

/// Type of the link attribute. This determines the type of the [`RouteAttr`].
#[repr(u16)]
#[derive(Debug, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[serde(try_from = "u16")]
pub enum LinkAttrType {
    Unspec,
    Address,
    Broadcast,
    InterfaceName,
    MaxTransmissionUnit,
    Link,
    QueueingDiscipline,
    Stats,
    Cost,
    Priority,
    Master,
    Wireless,
    Protinfo,
    TransmissionQueueLen,
    Map,
    Weight,
    Operstate,
    Linkmode,
    Linkinfo,
    NetNsPid,
    InterfaceAlias,
    NumVf,
    VfinfoList,
    Stats64,
    VfPorts,
    PortSelf,
    AfSpec,
    Group,
    NetNsFd,
    ExtMask,
    Promiscuity,
    NumTxQueues,
    NumRxQueues,
    Carrier,
    PhysPortId,
    CarrierChanges,
    PhysSwitchId,
    LinkNetnsid,
    PhysPortName,
    ProtoDown,
    GsoMaxSegs,
    GsoMaxSize,
    Pad,
    Xdp,
    Event,
    NewNetnsid,
    IfNetnsid,
    TargetNetnsid,
    CarrierUpCount,
    CarrierDownCount,
    NewInterfaceIndex,
    MinMtu,
    MaxMtu,
    PropList,
    AltInterfaceName,
    PermAddress,
    ProtoDownReason,
    ParentDevName,
    ParentDevBusName,
    GroMaxSize,
    TsoMaxSize,
    TsoMaxSegs,
    AllMulti,
}

impl From<LinkAttrType> for u16 {
    fn from(typ: LinkAttrType) -> Self {
        match typ {
            LinkAttrType::Unspec => 0,
            LinkAttrType::Address => 1,
            LinkAttrType::Broadcast => 2,
            LinkAttrType::InterfaceName => 3,
            LinkAttrType::MaxTransmissionUnit => 4,
            LinkAttrType::Link => 5,
            LinkAttrType::QueueingDiscipline => 6,
            LinkAttrType::Stats => 7,
            LinkAttrType::Cost => 8,
            LinkAttrType::Priority => 9,
            LinkAttrType::Master => 10,
            LinkAttrType::Wireless => 11,
            LinkAttrType::Protinfo => 12,
            LinkAttrType::TransmissionQueueLen => 13,
            LinkAttrType::Map => 14,
            LinkAttrType::Weight => 15,
            LinkAttrType::Operstate => 16,
            LinkAttrType::Linkmode => 17,
            LinkAttrType::Linkinfo => 18,
            LinkAttrType::NetNsPid => 19,
            LinkAttrType::InterfaceAlias => 20,
            LinkAttrType::NumVf => 21,
            LinkAttrType::VfinfoList => 22,
            LinkAttrType::Stats64 => 23,
            LinkAttrType::VfPorts => 24,
            LinkAttrType::PortSelf => 25,
            LinkAttrType::AfSpec => 26,
            LinkAttrType::Group => 27,
            LinkAttrType::NetNsFd => 28,
            LinkAttrType::ExtMask => 29,
            LinkAttrType::Promiscuity => 30,
            LinkAttrType::NumTxQueues => 31,
            LinkAttrType::NumRxQueues => 32,
            LinkAttrType::Carrier => 33,
            LinkAttrType::PhysPortId => 34,
            LinkAttrType::CarrierChanges => 35,
            LinkAttrType::PhysSwitchId => 36,
            LinkAttrType::LinkNetnsid => 37,
            LinkAttrType::PhysPortName => 38,
            LinkAttrType::ProtoDown => 39,
            LinkAttrType::GsoMaxSegs => 40,
            LinkAttrType::GsoMaxSize => 41,
            LinkAttrType::Pad => 42,
            LinkAttrType::Xdp => 43,
            LinkAttrType::Event => 44,
            LinkAttrType::NewNetnsid => 45,
            LinkAttrType::IfNetnsid => 46,
            LinkAttrType::TargetNetnsid => 46,
            LinkAttrType::CarrierUpCount => 47,
            LinkAttrType::CarrierDownCount => 48,
            LinkAttrType::NewInterfaceIndex => 49,
            LinkAttrType::MinMtu => 50,
            LinkAttrType::MaxMtu => 51,
            LinkAttrType::PropList => 52,
            LinkAttrType::AltInterfaceName => 53,
            LinkAttrType::PermAddress => 54,
            LinkAttrType::ProtoDownReason => 55,
            LinkAttrType::ParentDevName => 56,
            LinkAttrType::ParentDevBusName => 57,
            LinkAttrType::GroMaxSize => 58,
            LinkAttrType::TsoMaxSize => 59,
            LinkAttrType::TsoMaxSegs => 60,
            LinkAttrType::AllMulti => 61,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum LinkAttrValue {
    Unspec,
    Address(IpAddr),
    Broadcast(Vec<u8>),
    InterfaceName(String),
    MaxTransmissionUnit(Vec<u8>),
    Link(Vec<u8>),
    QueueingDiscipline(Vec<u8>),
    Stats(Vec<u8>),
    Cost(Vec<u8>),
    Priority(Vec<u8>),
    Master(Vec<u8>),
    Wireless(Vec<u8>),
    Protinfo(Vec<u8>),
    TransmissionQueueLen(Vec<u8>),
    Map(Vec<u8>),
    Weight(Vec<u8>),
    Operstate(Vec<u8>),
    Linkmode(Vec<u8>),
    Linkinfo(Vec<u8>),
    NetNsPid(Vec<u8>),
    InterfaceAlias(Vec<u8>),
    NumVf(Vec<u8>),
    VfinfoList(Vec<u8>),
    Stats64(Vec<u8>),
    VfPorts(Vec<u8>),
    PortSelf(Vec<u8>),
    AfSpec(Vec<u8>),
    Group(Vec<u8>),
    NetNsFd(Vec<u8>),
    ExtMask(Vec<u8>),
    Promiscuity(u32),
    NumTxQueues(Vec<u8>),
    NumRxQueues(Vec<u8>),
    Carrier(Vec<u8>),
    PhysPortId(Vec<u8>),
    CarrierChanges(Vec<u8>),
    PhysSwitchId(Vec<u8>),
    LinkNetnsid(Vec<u8>),
    PhysPortName(String),
    ProtoDown(Vec<u8>),
    GsoMaxSegs(Vec<u8>),
    GsoMaxSize(Vec<u8>),
    Pad(Vec<u8>),
    Xdp(Vec<u8>),
    Event(Vec<u8>),
    NewNetnsid(Vec<u8>),
    IfNetnsid(Vec<u8>),
    TargetNetnsid(Vec<u8>),
    CarrierUpCount(Vec<u8>),
    CarrierDownCount(Vec<u8>),
    NewInterfaceIndex(Vec<u8>),
    MinMtu(Vec<u8>),
    MaxMtu(Vec<u8>),
    PropList(Vec<u8>),
    AltInterfaceName(String),
    PermAddress(IpAddr),
    ProtoDownReason(Vec<u8>),
    ParentDevName(String),
    ParentDevBusName(String),
    GroMaxSize(Vec<u8>),
    TsoMaxSize(Vec<u8>),
    TsoMaxSegs(Vec<u8>),
    AllMulti(Vec<u8>),
}

#[rustfmt::skip]
impl LinkAttrValue {
    pub(crate) fn deserialize(typ: LinkAttrType, payload: &[u8]) -> Result<Self> {
        match typ {
            LinkAttrType::Unspec => {
                Ok(Self::Unspec)
            },
            LinkAttrType::Address => {
                deserialize_ip_addr(payload).map(Self::Address)
            },
            LinkAttrType::Broadcast => {
                Ok(Self::Broadcast(payload.to_vec()))
            },
            LinkAttrType::InterfaceName => {
                Ok(Self::InterfaceName(deserialize_ascii(payload)))
            },
            LinkAttrType::MaxTransmissionUnit => {
                Ok(Self::MaxTransmissionUnit(payload.to_vec()))
            },
            LinkAttrType::Link => {
                Ok(Self::Link(payload.to_vec()))
            },
            LinkAttrType::QueueingDiscipline => {
                Ok(Self::QueueingDiscipline(payload.to_vec()))
            },
            LinkAttrType::Stats => {
                Ok(Self::Stats(payload.to_vec()))
            },
            LinkAttrType::Cost => {
                Ok(Self::Cost(payload.to_vec()))
            },
            LinkAttrType::Priority => {
                Ok(Self::Priority(payload.to_vec()))
            },
            LinkAttrType::Master => {
                Ok(Self::Master(payload.to_vec()))
            },
            LinkAttrType::Wireless => {
                Ok(Self::Wireless(payload.to_vec()))
            },
            LinkAttrType::Protinfo => {
                Ok(Self::Protinfo(payload.to_vec()))
            },
            LinkAttrType::TransmissionQueueLen => {
                Ok(Self::TransmissionQueueLen(payload.to_vec()))
            },
            LinkAttrType::Map => {
                Ok(Self::Map(payload.to_vec()))
            },
            LinkAttrType::Weight => {
                Ok(Self::Weight(payload.to_vec()))
            },
            LinkAttrType::Operstate => {
                Ok(Self::Operstate(payload.to_vec()))
            },
            LinkAttrType::Linkmode => {
                Ok(Self::Linkmode(payload.to_vec()))
            },
            LinkAttrType::Linkinfo => {
                Ok(Self::Linkinfo(payload.to_vec()))
            },
            LinkAttrType::NetNsPid => {
                Ok(Self::NetNsPid(payload.to_vec()))
            },
            LinkAttrType::InterfaceAlias => {
                Ok(Self::InterfaceAlias(payload.to_vec()))
            },
            LinkAttrType::NumVf => {
                Ok(Self::NumVf(payload.to_vec()))
            },
            LinkAttrType::VfinfoList => {
                Ok(Self::VfinfoList(payload.to_vec()))
            },
            LinkAttrType::Stats64 => {
                Ok(Self::Stats64(payload.to_vec()))
            },
            LinkAttrType::VfPorts => {
                Ok(Self::VfPorts(payload.to_vec()))
            },
            LinkAttrType::PortSelf => {
                Ok(Self::PortSelf(payload.to_vec()))
            },
            LinkAttrType::AfSpec => {
                Ok(Self::AfSpec(payload.to_vec()))
            },
            LinkAttrType::Group => {
                Ok(Self::Group(payload.to_vec()))
            },
            LinkAttrType::NetNsFd => {
                Ok(Self::NetNsFd(payload.to_vec()))
            },
            LinkAttrType::ExtMask => {
                Ok(Self::ExtMask(payload.to_vec()))
            },
            LinkAttrType::Promiscuity => {
                deserialize_u32(payload).map(Self::Promiscuity)
            },
            LinkAttrType::NumTxQueues => {
                Ok(Self::NumTxQueues(payload.to_vec()))
            },
            LinkAttrType::NumRxQueues => {
                Ok(Self::NumRxQueues(payload.to_vec()))
            },
            LinkAttrType::Carrier => {
                Ok(Self::Carrier(payload.to_vec()))
            },
            LinkAttrType::PhysPortId => {
                Ok(Self::PhysPortId(payload.to_vec()))
            },
            LinkAttrType::CarrierChanges => {
                Ok(Self::CarrierChanges(payload.to_vec()))
            },
            LinkAttrType::PhysSwitchId => {
                Ok(Self::PhysSwitchId(payload.to_vec()))
            },
            LinkAttrType::LinkNetnsid => {
                Ok(Self::LinkNetnsid(payload.to_vec()))
            },
            LinkAttrType::PhysPortName => {
                Ok(Self::PhysPortName(deserialize_ascii(payload)))
            },
            LinkAttrType::ProtoDown => {
                Ok(Self::ProtoDown(payload.to_vec()))
            },
            LinkAttrType::GsoMaxSegs => {
                Ok(Self::GsoMaxSegs(payload.to_vec()))
            },
            LinkAttrType::GsoMaxSize => {
                Ok(Self::GsoMaxSize(payload.to_vec()))
            },
            LinkAttrType::Pad => {
                Ok(Self::Pad(payload.to_vec()))
            },
            LinkAttrType::Xdp => {
                Ok(Self::Xdp(payload.to_vec()))
            },
            LinkAttrType::Event => {
                Ok(Self::Event(payload.to_vec()))
            },
            LinkAttrType::NewNetnsid => {
                Ok(Self::NewNetnsid(payload.to_vec()))
            },
            LinkAttrType::IfNetnsid => {
                Ok(Self::IfNetnsid(payload.to_vec()))
            },
            LinkAttrType::TargetNetnsid => {
                Ok(Self::TargetNetnsid(payload.to_vec()))
            },
            LinkAttrType::CarrierUpCount => {
                Ok(Self::CarrierUpCount(payload.to_vec()))
            },
            LinkAttrType::CarrierDownCount => {
                Ok(Self::CarrierDownCount(payload.to_vec()))
            },
            LinkAttrType::NewInterfaceIndex => {
                Ok(Self::NewInterfaceIndex(payload.to_vec()))
            },
            LinkAttrType::MinMtu => {
                Ok(Self::MinMtu(payload.to_vec()))
            },
            LinkAttrType::MaxMtu => {
                Ok(Self::MaxMtu(payload.to_vec()))
            },
            LinkAttrType::PropList => {
                Ok(Self::PropList(payload.to_vec()))
            },
            LinkAttrType::AltInterfaceName => {
                Ok(Self::AltInterfaceName(deserialize_ascii(payload)))
            },
            LinkAttrType::PermAddress => {
                deserialize_ip_addr(payload).map(Self::PermAddress)
            },
            LinkAttrType::ProtoDownReason => {
                Ok(Self::ProtoDownReason(payload.to_vec()))
            },
            LinkAttrType::ParentDevName => {
                Ok(Self::ParentDevName(deserialize_ascii(payload)))
            },
            LinkAttrType::ParentDevBusName => {
                Ok(Self::ParentDevBusName(deserialize_ascii(payload)))
            },
            LinkAttrType::GroMaxSize => {
                Ok(Self::GroMaxSize(payload.to_vec()))
            },
            LinkAttrType::TsoMaxSize => {
                Ok(Self::TsoMaxSize(payload.to_vec()))
            },
            LinkAttrType::TsoMaxSegs => {
                Ok(Self::TsoMaxSegs(payload.to_vec()))
            },
            LinkAttrType::AllMulti => {
                Ok(Self::AllMulti(payload.to_vec()))
            },
        }
    }
}
