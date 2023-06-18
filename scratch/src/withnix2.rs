use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let nlsock = netlink::connect()?;
    netlink::dump_routes2(nlsock)?;
    Ok(())
}

mod netlink {
    use bincode::{deserialize, serialize};
    use libc::{AF_INET, NLMSG_DONE, NLM_F_DUMP, NLM_F_REQUEST, RTM_GETROUTE};
    use nix::errno::Errno;
    use nix::sys::socket::{
        bind, recvmsg, send, socket, AddressFamily, MsgFlags, NetlinkAddr, RecvMsg, SockFlag,
        SockProtocol, SockType,
    };
    use nix::unistd::getpid;
    use serde::Serialize;
    use std::io::{IoSliceMut, Read};
    use std::mem::size_of;
    use std::os::fd::RawFd;

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
        #[error("expected more bytes but there were not enough")]
        ErrUnexpectedEof,
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
            SockFlag::SOCK_CLOEXEC | SockFlag::SOCK_NONBLOCK,
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
        Ok(())
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

    // Read data from the socket until it blocks. This is detected by an error
    // reading from the socket, so it assumes the socket has been opened with
    // the SOCK_NONBLOCK flag.
    pub fn read_until_block(nlsock: &NetlinkSocket) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];

        loop {
            let chunk = recv_sock(&nlsock);
            match chunk {
                Ok(mut chunk) => bytes.append(&mut chunk),
                Err(Error::ErrRecvSocket(Errno::EAGAIN)) => break,
                Err(err) => return Err(err),
            }
        }

        Ok(bytes)
    }

    pub fn read_all_messages(bytes: &[u8]) -> Result<Vec<messages::NetlinkMessage<&[u8]>>, Error> {
        use messages::{NetlinkHeader, NetlinkMessage};

        let mut messages = vec![];
        let mut cursor = 0usize;

        loop {
            let header_len = nlmsg_hdrlen();
            let header_bytes = &bytes[cursor..cursor + header_len];
            let header: NetlinkHeader =
                deserialize(&header_bytes).map_err(Error::ErrDeserialize)?;
            cursor += header_len;

            if header.nlmsg_type == NLMSG_DONE as u16 {
                break;
            }

            log::debug!("{header:?}");

            // Header length is inclusive of the header itself
            let payload_len = header.nlmsg_len as usize - header_len;

            if cursor + (payload_len as usize) > bytes.len() {
                return Err(Error::ErrUnexpectedEof);
            }

            let payload = &bytes[cursor..cursor + payload_len as usize];
            cursor += payload_len;

            messages.push(NetlinkMessage { header, payload });
        }

        Ok(messages)
    }

    // loop {
    //     let mut chunk = recv_sock(&nlsock)?;

    //     if chunk.len() < nlmsg_hdrlen() {
    //         return Err(Error::ErrUnexpectedEof);
    //     }

    //     let header_bytes = &chunk[cursor..nlmsg_hdrlen()];
    //     let header: NetlinkHeader = deserialize(&header_bytes)
    //         .map_err(Error::ErrDeserialize)?;

    //     // Loop in case the payload of a single message is split over
    //     // multiple recv() attempts
    //     loop {
    //         if chunk.len() >= header.nlmsg_len as usize {
    //             break;
    //         }
    //         let mut next_chunk = recv_sock(&nlsock)?;
    //         chunk.append(&mut next_chunk);
    //     }

    //     let payload = &chunk[cursor + nlmsg_hdrlen()..cursor + header.nlmsg_len as usize];

    // }

    // // let mut cursor = 0;
    // // let mut messages: Vec<messages::NetlinkMessage<Vec<u8>>> = vec![];

    // todo!()

    pub fn dump_routes2(nlsock: NetlinkSocket) -> Result<(), Error> {
        let rtreq = dump_routes_req();
        send_sock_val(&nlsock, &rtreq)?;

        let data = read_until_block(&nlsock)?;
        let messages = read_all_messages(&data)?;

        log::info!("Received {} messages", messages.len());
        for message in &messages {
            log::trace!("  {message:?}");
        }

        log::info!("Done");

        Ok(())
    }

    /// Print a list of all routes
    pub fn dump_routes(nlsock: NetlinkSocket) -> Result<(), Error> {
        let rtreq = dump_routes_req();
        send_sock_val(&nlsock, &rtreq)?;

        log::trace!("recv_sock");
        let mut bytes = recv_sock(&nlsock)?;

        let mut messages = vec![];

        let mut base = 0;
        loop {
            let mut cursor = base;

            if cursor + nlmsg_hdrlen() > bytes.len() {
                log::trace!("recv_sock");
                let mut more_bytes = recv_sock(&nlsock)?;
                bytes.append(&mut more_bytes);
            }

            let header = {
                let header_len = nlmsg_hdrlen();
                let header_bytes = &bytes[cursor..cursor + header_len];
                cursor += header_len;

                deserialize::<messages::NetlinkHeader>(&header_bytes)
                    .map_err(Error::ErrDeserialize)?
            };

            messages.push(NetlinkMessage::MessageHeader(header.clone()));

            if header.nlmsg_type == NLMSG_DONE as u16 {
                break;
            }

            let rtm = {
                let rtm_len = nlmsg_align::<messages::RouteMessage>();
                let rtm_bytes = &bytes[cursor..cursor + rtm_len];
                cursor += rtm_len;

                deserialize::<messages::RouteMessage>(&rtm_bytes).map_err(Error::ErrDeserialize)?
            };

            messages.push(NetlinkMessage::RouteMessage(rtm.clone()));

            loop {
                if cursor as u32 == (base as u32 + header.nlmsg_len) {
                    // Reached end of message
                    base = cursor;
                    break;
                }

                let rattr_len = nlmsg_align::<messages::RouteAttrHeader>();
                let rattr_bytes = &bytes[cursor..cursor + rattr_len];
                let rattr = deserialize::<messages::RouteAttrHeader>(&rattr_bytes)
                    .map_err(Error::ErrDeserialize)?;
                cursor += rattr_len;
                messages.push(NetlinkMessage::RouteAttrHeader(rattr.clone()));

                let attr_val_len = (rattr.rta_len - (rattr_len as u16)) as usize;
                let attr_val_bytes = &bytes[cursor..cursor + attr_val_len];
                messages.push(NetlinkMessage::RouteAttrValue(attr_val_bytes.to_vec()));
                cursor += attr_val_len;
            }
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

        #[derive(PartialEq, Clone, Debug)]
        pub struct NetlinkMessage<T> {
            pub header: NetlinkHeader,
            pub payload: T,
        }

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
