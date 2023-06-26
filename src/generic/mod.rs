use std::marker::PhantomData;

use crate::{bytes::{SliceReader, OwnedSliceReader, aligned_size_of, aligned_size}, Result, Flag};

pub trait Serialize: Sized {
    fn serialize(byes: &[u8]) -> Result<Self>;
}

pub trait Deserialize<'de>: Sized {
    fn deserialize(bytes: &'de [u8]) -> Result<Self>;
}

pub trait DeserializeOwned: Sized {
    fn deserialize(bytes: Vec<u8>) -> Result<Self>;
}

pub trait DeserializeTyped<T>: Sized {
    fn deserialize(typ: &T, payload: &[u8]) -> Result<Self>;
}

pub trait MessageType: Sized + PartialEq + From<u16> + Into<u16> + Clone {}

impl MessageType for u16 {}

#[derive(Debug)]
pub struct NetlinkMessage<T, P>
where
    T: MessageType,
    P: for<'a> Deserialize<'a>
{
    pub header: NetlinkHeader<T>,
    pub payload: P,
}

#[derive(Debug, serde::Deserialize)]
pub struct NetlinkHeader<T: MessageType>
{
    pub(crate) nlmsg_len: u32,
    pub(crate) nlmsg_type: T,
    pub(crate) nlmsg_flags: u16,
    pub(crate) nlmsg_seq: u32,
    pub(crate) nlmsg_pid: u32,
}

impl<T: MessageType> NetlinkHeader<T>
{
    pub fn clone_with_type<T2: MessageType>(other: NetlinkHeader<T2>, nlmsg_type: T) -> Self {
        NetlinkHeader {
            nlmsg_type,
            nlmsg_len: other.nlmsg_len,
            nlmsg_flags: other.nlmsg_flags,
            nlmsg_seq: other.nlmsg_seq,
            nlmsg_pid: other.nlmsg_pid,
        }
    }

    pub fn has_type(&self, nlmsg_type: T) -> bool {
        self.nlmsg_type == nlmsg_type
    }

    pub fn has_flags(&self, flags: Flag) -> bool {
        let flags: u16 = flags.into();
        self.nlmsg_flags & flags == flags
    }
}

impl<'a, T, P> Deserialize<'a> for NetlinkMessage<T, P>
where
    T: MessageType + serde::de::DeserializeOwned,
    P: for<'de> Deserialize<'de>
{
    fn deserialize(bytes: &'a [u8]) -> Result<Self> {
        let mut reader = SliceReader::new(&bytes);
        let header = reader.read::<NetlinkHeader<T>>()?;
        let bytes = reader.take(header.nlmsg_len as usize)?;
        let payload = P::deserialize(bytes)?;
        Ok(NetlinkMessage { header, payload })
    }
}

#[derive(Debug)]
pub struct BytePayload(Vec<u8>);

impl<'a> Deserialize<'a> for BytePayload {
    fn deserialize(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self(bytes.to_vec()))
    }
}

pub struct RouteMessage<P>
where
    P: AttrReader<RouteAttrType, RouteAttrValue>,
{
    pub rtm_family: u16,
    pub rtm_type: RouteAttrType,
    pub rtm_payload: P,
}

#[derive(Debug)]
pub struct Attr<T, P>
where
    P: DeserializeTyped<T>,
{
    pub header: AttrHeader<T>,
    pub payload: P,
}

#[derive(Debug, serde::Deserialize)]
pub struct AttrHeader<T> {
    pub nla_len: u16,
    pub nla_type: T,
}

impl<'de, T> crate::generic::Deserialize<'de> for AttrHeader<T>
where
    T: serde::de::DeserializeOwned
{
    fn deserialize(bytes: &'de [u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(crate::Error::ErrDeserialize)
    }
}

pub trait AttrReader<T, P>
where
    P: DeserializeTyped<T>,
{
    /// Attempt to read and deserialize the next attribute.
    fn try_next(&mut self) -> Result<Option<Attr<T, P>>>;

    fn read_all(&mut self) -> Result<Vec<Attr<T, P>>> {
        let mut attrs = vec![];
        while let Ok(Some(attr)) = self.try_next() {
            attrs.push(attr);
        }
        Ok(attrs)
    }
}

#[derive(Debug)]
pub struct AttrSliceReader<T, P>
where
    T: DeserializeOwned,
    P: DeserializeTyped<T>,
{
    bytes: OwnedSliceReader,
    markers: PhantomData<(T, P)>,
}

impl<T, P> AttrSliceReader<T, P>
where
    T: DeserializeOwned,
    P: DeserializeTyped<T>,
{
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: OwnedSliceReader::new(bytes),
            markers: PhantomData,
        }
    }
}

impl<T, P> AttrReader<T, P> for AttrSliceReader<T, P>
where
    T: TryFrom<u16> + DeserializeOwned + serde::de::DeserializeOwned,
    P: DeserializeTyped<T>,
{
    fn try_next(&mut self) -> Result<Option<Attr<T, P>>> {
        if self.bytes.is_empty() {
            return Ok(None);
        }

        let header = self.bytes.read2::<AttrHeader<u16>>()?;
        let typ = T::try_from(header.nla_type).map_err(|_| crate::Error::ErrUnexpectedEof)?;

        let len = header.nla_len as usize - aligned_size_of::<AttrHeader<u16>>();
        let len = aligned_size(len);
        let payload_bytes = self.bytes.take(len)?;

        let payload = P::deserialize(&typ, &payload_bytes)?;
        let header = AttrHeader {
            nla_len: header.nla_len,
            nla_type: typ,
        };
        Ok(Some(Attr { header, payload }))
    }
}

pub enum RouteAttrType {
    One,
    Two,
}

pub enum RouteAttrValue {
    One(u32),
    Two(String),
}

impl DeserializeTyped<RouteAttrType> for RouteAttrValue {
    fn deserialize(typ: &RouteAttrType, _payload: &[u8]) -> Result<Self> {
        Ok(match typ {
            RouteAttrType::One => Self::One(1),
            RouteAttrType::Two => Self::Two("test".to_string()),
        })
    }
}
