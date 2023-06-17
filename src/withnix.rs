use nix::errno::Errno;
use nix::libc::{AF_INET, NLM_F_DUMP, NLM_F_REQUEST, RTM_GETROUTE};
use nix::sys::socket::{
    bind, recv, send, socket, AddressFamily, MsgFlags, NetlinkAddr, SockFlag, SockProtocol,
    SockType,
};
use nix::unistd::getpid;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use std::os::fd::RawFd;

pub fn main() {
    println!("Hello, world!");

    println!("{}", size_of::<NetlinkRequest<RouteMessage>>());

    let netlink_fd = netlink_connect().expect("could not create netlink fd");
    get_routes(netlink_fd).expect("could not get routes");

    println!("Done");
}

pub struct NetlinkSocket {
    seq: u32,
    fd: RawFd,
}

pub fn netlink_connect() -> Result<RawFd, Errno> {
    let netlink_fd = socket(
        AddressFamily::Netlink,
        SockType::Raw,
        SockFlag::SOCK_CLOEXEC,
        SockProtocol::NetlinkRoute,
    )?;
    println!("got netlink fd");

    // Binding is not strictly necessary, but provides metadata that is useful
    // to strace that makes it's output much easier to understand.
    // https://john-millikin.com/creating-tun-tap-interfaces-in-linux#fn:1
    // let sock_addr = getsockname(netlink_fd)?;
    let pid = getpid().as_raw() as u32;
    let sock_addr = NetlinkAddr::new(pid, 0);
    bind(netlink_fd, &sock_addr)?;

    println!("Got sockaddr");

    Ok(netlink_fd)
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct NetlinkRequest<T: Serialize> {
    header: NetlinkHeader,
    content: T,
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub struct NetlinkHeader {
    pub nlmsg_len: u32,
    pub nlmsg_type: u16,
    pub nlmsg_flags: u16,
    pub nlmsg_seq: u32,
    pub nlmsg_pid: u32,
}

#[repr(C)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RouteMessage {
    rtm_family: u8,
    rtm_dst_len: u8,
    rtm_src_len: u8,
    rtm_tos: u8,
    rtm_table: u8,
    rtm_protocol: u8,
    rtm_scope: u8,
    rtm_type: u8,
    rtm_flags: u8,
}

#[repr(C)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct NetlinkAttr {
    nla_len: u16,
    nla_type: u16,
}

#[repr(C)]
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RouteAttr {
    rta_len: u16,
    rta_type: u16,
}

pub fn get_routes(netlink_fd: RawFd) -> Result<(), Errno> {
    let req = NetlinkRequest {
        header: NetlinkHeader {
            nlmsg_len: nlmsg_length::<RouteMessage>() as u32,
            nlmsg_type: RTM_GETROUTE as u16,
            nlmsg_flags: (NLM_F_REQUEST | NLM_F_DUMP) as u16,
            nlmsg_seq: 0,
            nlmsg_pid: 0,
        },
        content: RouteMessage {
            rtm_family: AF_INET as u8,
            ..Default::default()
        },
    };

    println!("Route message len: {}", nlmsg_length::<RouteMessage>());

    let encoded = bincode::serialize(&req).unwrap();
    println!("len encoded: {}", encoded.len());
    let ret = send(netlink_fd, &encoded, MsgFlags::empty())?;
    // let slice = IoSlice::new(&en                                                    rtm_table is ignoredcoded);
    // let ret = sendmsg::<()>(netlink_fd, &[slice], &[], MsgFlags::empty(), None)?;
    println!("send() returned {ret}");

    let mut leading_buf = [0u8; 2048];
    recv(netlink_fd, &mut leading_buf, MsgFlags::empty())?;
    println!("Received leading buffer of length: {}", leading_buf.len());

    let mut trailing_buf = [0u8; 2048];
    recv(netlink_fd, &mut trailing_buf, MsgFlags::empty())?;
    println!("Received trailing buffer of length: {}", trailing_buf.len());

    let header_slice = &leading_buf[0..nlmsg_hdrlen()];
    let header =
        bincode::deserialize::<NetlinkHeader>(&header_slice).expect("could not deserialize header");
    println!("header: {header:?}");

    let mut cursor = nlmsg_hdrlen();
    for i in 0..header.nlmsg_len {
        let rtmsg_slice = &leading_buf[cursor..cursor + nlmsg_length::<RouteMessage>()];
        let rtmsg = bincode::deserialize::<RouteMessage>(&rtmsg_slice)
            .expect("could not deserialize RouteMessage");
        cursor += rtmsg_slice.len();
        println!("rtmsg: {rtmsg:?}");

        let nl_attr_slice = &leading_buf[cursor..cursor + nlmsg_length::<NetlinkAttr>()];
        let nl_attr = bincode::deserialize::<RouteAttr>(&nl_attr_slice)
            .expect("could not deserialize nl_attr_slice");
        println!("attr: {nl_attr:?}");

        break;
    }

    // let mut recv_iov = [IoSliceMut::new(&mut recv_buf)];
    // let res = recvmsg::<()>(netlink_fd, &mut recv_iov, None, MsgFlags::empty())?;

    // for iov in res.iovs() {
    //     let data: NetlinkRequest<Vec<u8>> = bincode::deserialize(&iov).expect("could not deserialize");
    //     println!("MSG\n{data:?}");
    // }

    Ok(())
}

pub const NLMSG_ALIGNTO: usize = 4;

fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}

fn nlmsg_hdrlen() -> usize {
    let hdrlen = std::mem::size_of::<NetlinkHeader>();
    nlmsg_align(hdrlen)
}

fn nlmsg_length<T>() -> usize {
    size_of::<T>() + nlmsg_hdrlen()
}
