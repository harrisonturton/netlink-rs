// --------------------------------------------------
// netlink
// https://man7.org/linux/man-pages/man7/netlink.7.html
// --------------------------------------------------

#[repr(C)]
pub struct nlmsghdr {
    pub nlmsg_len: u32,
    pub nlmsg_type: u16,
    pub nlmsg_flags: u16,
    pub nlmsg_seq: u32,
    pub nlmsg_pid: u32,
}

// https://www.infradead.org/~tgr/libnl/doc/core.html#core_netlink_fundamentals
// https://john-millikin.com/creating-tun-tap-interfaces-in-linux
// https://inai.de/documents/Netlink_Protocol.pdf

// https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/netlink.h#L102

// #define NLMSG_ALIGNTO	4U
// #define NLMSG_ALIGN(len) ( ((len)+NLMSG_ALIGNTO-1) & ~(NLMSG_ALIGNTO-1) )
// #define NLMSG_HDRLEN	 ((int) NLMSG_ALIGN(sizeof(struct nlmsghdr)))
// #define NLMSG_LENGTH(len) ((len) + NLMSG_HDRLEN)
// #define NLMSG_SPACE(len) NLMSG_ALIGN(NLMSG_LENGTH(len))
// #define NLMSG_DATA(nlh)  ((void *)(((char *)nlh) + NLMSG_HDRLEN))
// #define NLMSG_NEXT(nlh,len)	 ((len) -= NLMSG_ALIGN((nlh)->nlmsg_len), \
// 				  (struct nlmsghdr *)(((char *)(nlh)) + \
// 				  NLMSG_ALIGN((nlh)->nlmsg_len)))
// #define NLMSG_OK(nlh,len) ((len) >= (int)sizeof(struct nlmsghdr) && \
// 			   (nlh)->nlmsg_len >= sizeof(struct nlmsghdr) && \
// 			   (nlh)->nlmsg_len <= (len))
// #define NLMSG_PAYLOAD(nlh,len) ((nlh)->nlmsg_len - NLMSG_SPACE((len)))

// #define NLMSG_NOOP		0x1	/* Nothing.		*/
// #define NLMSG_ERROR		0x2	/* Error		*/
// #define NLMSG_DONE		0x3	/* End of a dump	*/
// #define NLMSG_OVERRUN		0x4	/* Data lost		*/
// #define NLMSG_MIN_TYPE		0x10	/* < 0x10: reserved control messages */
#[repr(C)]
pub struct nlmsgerr {
    pub error: u32,
    pub hdr: nlmsghdr,
    // Original message header
    pub original: Option<nlmsghdr>,
}

pub const NLMSG_NOOP: u16 = todo!();
pub const NLMSG_ERROR: u16 = todo!();
pub const NLMSG_DONE: u16 = todo!();

// --------------------------------------------------
// rnetlink
// https://man7.org/linux/man-pages/man7/rtnetlink.7.html
// --------------------------------------------------

#[repr(C)]
pub struct ifinfomsg {
    ifi_family: u8,
    ifi_type: u16,
    ifi_index: i32,
    ifi_flags: u32,
    ifi_change: u32,
}

pub const IFLA_UNSPEC: u16 = todo!();
pub const IFLA_ADDRESS: u16 = todo!();
pub const IFLA_BROADCAST: u16 = todo!();
pub const IFLA_IFNAME: u16 = todo!();
pub const IFLA_MTU: u16 = todo!();
pub const IFLA_LINK: u16 = todo!();
pub const IFLA_QDISC: u16 = todo!();
pub const IFLA_STATS: u16 = todo!();

#[repr(C)]
pub struct ifaddrmsg {
    pub ifa_family: u8,
    pub ifa_prefixlen: u8,
    pub ifa_flags: u8,
    pub ifa_scope: u8,
    pub ifa_index: u32,
    // optionally followed by attributes
}
