use nix::libc::{RTM_GETADDR, RTM_GETROUTE};
use serde::{Serialize, Deserialize};

pub const AF_INET: u8 = 0x2;

pub const NEW_LINK: u16 = 16;
pub const DEL_LINK: u16 = 17;
pub const GET_LINK: u16 = 18;
pub const SET_LINK: u16 = 19;

pub const NEW_ADDR: u16 = 20;
pub const DEL_ADDR: u16 = 21;
pub const GET_ADDR: u16 = 22;

pub const NEW_ROUTE: u16 = 24;
pub const DEL_ROUTE: u16 = 25;
pub const GET_ROUTE: u16 = 26;