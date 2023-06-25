use serde::{Serialize, Deserialize};
use derive_builder::Builder;
use crate::Error;

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