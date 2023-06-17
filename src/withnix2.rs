use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let nlsock = netlink::connect()?;
    netlink::dump_routes(nlsock)?;
    Ok(())
}

mod netlink {
    use bincode::{deserialize, serialize};
    use libc::{AF_INET, NLM_F_DUMP, NLM_F_MULTI, NLM_F_REQUEST, RTM_GETROUTE, NLMSG_DONE};
    use nix::errno::Errno;
    use nix::sys::socket::{
        bind, recvmsg, send, socket, AddressFamily, MsgFlags, NetlinkAddr, RecvMsg, SockFlag,
        SockProtocol, SockType,
    };
    use nix::unistd::getpid;
    use serde::Serialize;
    use std::io::IoSliceMut;
    use std::mem::size_of;
    use std::os::fd::RawFd;

    use crate::withnix::NetlinkRequest;
    use crate::withnix2::netlink::messages::RouteMessage;

    pub const NLMSG_ALIGNTO: usize = 4;

    pub struct NetlinkSocket(RawFd);

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("failed to create socket with errno {0}")]
        ErrCreateSocket(Errno),
        #[error("failed to bind socket with errno {0}")]
        ErrBindSocket(Errno),
        #[error("failed to send to socket with errno {0}")]
        ErrSendSocket(Errno),
        #[error("failed to recv from socket with errno {0}")]
        ErrRecvSocket(Errno),
        #[error("failed to serialize with error {0}")]
        ErrSerialize(bincode::Error),
        #[error("failed to deserialize with error {0}")]
        ErrDeserialize(bincode::Error),
    }

    /// Create a Netlink connection  
    pub fn connect() -> Result<NetlinkSocket, Error> {
        let nfd = socket(
            AddressFamily::Netlink,
            SockType::Raw,
            SockFlag::SOCK_CLOEXEC,
            SockProtocol::NetlinkRoute,
        )
        .map_err(Error::ErrCreateSocket)?;

        let pid = getpid().as_raw() as u32;
        let sock_addr = NetlinkAddr::new(pid, 0);

        // Binding is not required. However, it provides metadata to strace that
        // enables it to render the netlink messages. Without binding, it just
        // prints binary data, which makes it very hard to debug/observe.
        // https://john-millikin.com/creating-tun-tap-interfaces-in-linux#fn:1
        bind(nfd, &sock_addr).map_err(Error::ErrBindSocket)?;

        Ok(NetlinkSocket(nfd))
    }

    pub fn send_sock_val<T: Serialize>(nlsock: &NetlinkSocket, val: &T) -> Result<(), Error> {
        let bytes = serialize(&val).map_err(Error::ErrSerialize)?;
        send_sock(nlsock, &bytes)?;
        todo!()
    }

    pub fn dump_routes_req() -> messages::NetlinkRequest<messages::RouteMessage> {
        messages::NetlinkRequest {
            header: messages::NetlinkHeader {
                nlmsg_len: nlmsg_length::<messages::RouteMessage>() as u32,
                nlmsg_type: RTM_GETROUTE,
                nlmsg_flags: (NLM_F_REQUEST | NLM_F_DUMP) as u16,
                ..Default::default()
            },
            message: messages::RouteMessage {
                rtm_family: AF_INET as u8,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Print a list of all routes
    pub fn dump_routes(nlsock: NetlinkSocket) -> Result<(), Error> {
        // log::trace!("Sending request");
        let rtreq = dump_routes_req();
        send_sock_val(&nlsock, &rtreq)?;
        // log::trace!("Sent request");

        // log::trace!("Reading");
        let mut bytes = recv_sock(&nlsock)?;
        // log::trace!("Read {} bytes", bytes.len());

        let mut messages = vec![];

        let mut base = 0;
        loop {
            let mut cursor = base;

            if cursor + nlmsg_hdrlen() > bytes.len() {
                // log::warn!("READING MORE BYTES");
                let mut more_bytes = recv_sock(&nlsock)?;
                bytes.append(&mut more_bytes);
            }

            let header = {
                // log::trace!("Reading NetlinkHeader");
                let header_len = nlmsg_hdrlen();
                let header_bytes = &bytes[cursor..cursor + header_len];
                cursor += header_len;

                deserialize::<messages::NetlinkHeader>(&header_bytes)
                    .map_err(Error::ErrDeserialize)?
            };

            messages.push(NetlinkMessage::MessageHeader(header.clone()));
            // log::info!("{header:?}");
            
            if header.nlmsg_type == NLMSG_DONE as u16 {
                // log::warn!("REACHED END OF MULTIPART MESSAGE");
                break; 
            }

            let rtm = {
                // log::trace!("Reading RouteMessage");
                let rtm_len = nlmsg_align::<RouteMessage>();
                let rtm_bytes = &bytes[cursor..cursor + rtm_len];
                cursor += rtm_len;

                deserialize::<messages::RouteMessage>(&rtm_bytes).map_err(Error::ErrDeserialize)?
            };

            messages.push(NetlinkMessage::RouteMessage(rtm.clone()));
            // log::info!("{rtm:?}");

            loop {
                if cursor as u32 == (base as u32 + header.nlmsg_len) {
                    // Reached end of message
                    base = cursor;
                    break;
                }

                // log::trace!("Reading RouteAttr");
                let rattr_len = nlmsg_align::<messages::RouteAttrHeader>();
                let rattr_bytes = &bytes[cursor..cursor + rattr_len];
                let rattr = deserialize::<messages::RouteAttrHeader>(&rattr_bytes)
                    .map_err(Error::ErrDeserialize)?;
                cursor += rattr_len;
                // log::info!("{rattr:?}");
                messages.push(NetlinkMessage::RouteAttrHeader(rattr.clone()));

                // log::trace!("reading table value");
                let attr_val_len = (rattr.rta_len - (rattr_len as u16)) as usize;
                let attr_val_bytes = &bytes[cursor..cursor + attr_val_len];
                messages.push(NetlinkMessage::RouteAttrValue(attr_val_bytes.to_vec()));
                cursor += attr_val_len;
            }

            // log::debug!("Processed");
        }

        log::info!("Received {} messages", messages.len());
        for message in &messages {
            log::trace!("  {message:?}");
        }

        log::info!("Done");

        Ok(())
    }

    // Inefficient loops to send(). Should replace with scatter/gather.
    fn send_sock(nlsock: &NetlinkSocket, data: &[u8]) -> Result<(), Error> {
        let &NetlinkSocket(fd) = nlsock;
        let mut remaining = data;

        loop {
            let sent = send(fd, &remaining, MsgFlags::empty()).map_err(Error::ErrSendSocket)?;
            if sent == remaining.len() {
                break;
            }
            remaining = &data[sent..];
        }

        Ok(())
    }

    // Inefficient loops to recv(). Should replace with scatter/gather.
    fn recv_sock(nlsock: &NetlinkSocket) -> Result<Vec<u8>, Error> {
        let &NetlinkSocket(fd) = nlsock;

        let mut buf_a = vec![0u8; 2048];
        let iov_a = IoSliceMut::new(&mut buf_a);

        let iov = &mut [iov_a];
        let msghdr: RecvMsg<NetlinkAddr> =
            recvmsg(fd, iov, None, MsgFlags::empty()).map_err(Error::ErrRecvSocket)?;

        let buf = msghdr.iovs().next().unwrap().to_vec();
        Ok(buf)
    }

    fn nlmsg_align<T>() -> usize {
        (size_of::<T>() + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
    }

    fn nlmsg_hdrlen() -> usize {
        nlmsg_align::<messages::NetlinkHeader>()
    }

    fn nlmsg_length<T>() -> usize {
        size_of::<T>() + nlmsg_hdrlen()
    }

    #[derive(PartialEq, Clone, Debug)]
    pub enum NetlinkMessage {
        MessageHeader(messages::NetlinkHeader),
        RouteMessage(messages::RouteMessage),
        RouteAttrHeader(messages::RouteAttrHeader),
        RouteAttrValue(Vec<u8>),
    }

    pub mod messages {
        use serde::{Deserialize, Serialize};

        // --------------------------------------------------
        // Netlink core messages
        // --------------------------------------------------

        #[repr(C)]
        #[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
        pub struct NetlinkHeader {
            pub nlmsg_len: u32,
            pub nlmsg_type: u16,
            pub nlmsg_flags: u16,
            pub nlmsg_seq: u32,
            pub nlmsg_pid: u32,
        }

        #[repr(C)]
        #[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
        pub struct NetlinkRequest<T: Serialize> {
            pub header: NetlinkHeader,
            pub message: T,
        }

        #[repr(C)]
        #[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
        pub struct NetlinkAttr {
            pub nla_len: u16,
            pub nla_type: u16,
        }

        // --------------------------------------------------
        // Netlink route messages
        // --------------------------------------------------

        #[repr(C)]
        #[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
        pub struct RouteMessage {
            pub rtm_family: u8,
            pub rtm_dst_len: u8,
            pub rtm_src_len: u8,
            pub rtm_tos: u8,
            pub rtm_table: u8,
            pub rtm_protocol: u8,
            pub rtm_scope: u8,
            pub rtm_type: u8,
            pub rtm_flags: u8,
        }

        #[repr(C)]
        #[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
        pub struct RouteAttrHeader {
            pub rta_len: u16,
            pub rta_type: u16,
        }
    }
}
