use crate::bytes::{aligned_size_of, serialize_aligned};
use crate::types::{Flag, MessageType, NetlinkHeader, NetlinkMessage};
use crate::{Error, Result};
use bincode::deserialize;
use nix::sys::socket::{
    bind, recv, send, socket, AddressFamily, MsgFlags, NetlinkAddr, SockFlag, SockProtocol,
    SockType,
};
use nix::unistd::getpid;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::fd::RawFd;

/// Wraps a socket [`RawFd`] descriptor to provide safe read and write methods
/// for sending and receiving Netlink messages. It will buffer reads and writes
/// to the socket.
///
/// This also keeps track of the PID and sequence numbers required to create a
/// properly-formatted Netlink messages. See [`NetlinkMessage::build`].
#[derive(PartialEq, Clone, Debug)]
pub struct NetlinkSocket {
    fd: RawFd,
    pid: u32,
}

impl NetlinkSocket {
    /// Initialize a new Netlink socket and connect to it.
    fn connect() -> Result<Self> {
        let fd = socket(
            AddressFamily::Netlink,
            SockType::Raw,
            SockFlag::SOCK_CLOEXEC | SockFlag::SOCK_NONBLOCK,
            SockProtocol::NetlinkRoute,
        )
        .map_err(Error::ErrCreateSocket)?;

        let pid: u32 = getpid()
            .as_raw()
            .try_into()
            .map_err(|_| Error::ErrValueConversion)?;
        let sock_addr = NetlinkAddr::new(pid, 0);

        // Binding is not required. However, it provides metadata to strace that
        // enables it to render the netlink messages. Without binding, it just
        // prints binary data, which makes it very hard to debug/observe.
        // https://john-millikin.com/creating-tun-tap-interfaces-in-linux#fn:1
        bind(fd, &sock_addr).map_err(Error::ErrBindSocket)?;

        Ok(NetlinkSocket { fd, pid })
    }
}

impl std::io::Read for NetlinkSocket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        recv(self.fd, buf, MsgFlags::empty())
            .map_err(|err| std::io::Error::from_raw_os_error(err as i32))
    }
}

impl std::io::Write for NetlinkSocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        send(self.fd, buf, MsgFlags::empty())
            .map_err(|errno| std::io::Error::from_raw_os_error(errno as i32))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Data is sent immediately when write() is called; flushing is not
        // needed. No-op implementation because [std::io::Write] requires it.
        Ok(())
    }
}

/// This is the primary way to interact with a Netlink interface. It provides
/// methods to read and write messages, and buffers all the underlying byte
/// reads.
///
/// For example:
///
/// ```rust
/// use std::error::Error;
/// use netlink::route::{RouteHeader, RouteMessageType, AF_INET};
/// use netlink::{NetlinkStream, Flag, NetlinkMessage};
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let mut conn = NetlinkStream::connect()?;
///
///     let rthdr = RouteHeader::builder()
///         .family(AF_INET)
///         .build();
///
///     let msg = NetlinkMessage::builder()
///         .typ(RouteMessageType::GetRoute)
///         .flags(Flag::Request | Flag::Dump)
///         .append(rthdr)?
///         .build();
///
///     conn.send(msg)?;
///
///     for msg in conn.into_iter(){
///         println!("{msg:?}");
///     }
///
///     Ok(())
/// }
/// ```
pub struct NetlinkStream {
    sock: NetlinkSocket,
    reader: BufReader<NetlinkSocket>,
    writer: BufWriter<NetlinkSocket>,
    seq: u32,
    has_remaining_reads: bool,
}

impl NetlinkStream {
    /// Returns a bidirectional stream of Netlink messages.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] when a Netlink socket cannot be successfully
    /// created. This might happen for a variety of reasons.
    pub fn connect() -> Result<Self> {
        let sock = NetlinkSocket::connect()?;
        let writer = BufWriter::new(sock.clone());
        let reader = BufReader::new(sock.clone());
        Ok(Self {
            sock,
            reader,
            writer,
            seq: 0,
            has_remaining_reads: true,
        })
    }

    /// Attempt to send a Netlink message.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] when writes to socket's underlying file
    /// descriptor fails.
    pub fn send(&mut self, mut msg: NetlinkMessage) -> Result<()> {
        let len = msg.payload.len() + aligned_size_of::<NetlinkHeader>();
        let header = NetlinkHeader {
            len: len.try_into().map_err(|_| Error::ErrValueConversion)?,
            typ: msg.header.typ,
            flags: msg.header.flags,
            pid: self.sock.pid,
            seq: self.seq,
        };

        let mut bytes = serialize_aligned(header)?;
        bytes.append(&mut msg.payload);

        self.writer
            .write_all(&bytes)
            .map_err(Error::ErrWriteSocket)?;
        self.writer.flush().map_err(Error::ErrWriteSocket)?;

        self.seq += 1;
        self.has_remaining_reads = true;
        Ok(())
    }

    pub fn recv_gen<T, P>(&mut self) -> Result<Option<crate::generic::NetlinkMessage<T, P>>>
    where
        T: crate::generic::MessageType,
        P: for<'de> crate::generic::Deserialize<'de>,
    {
        use crate::generic::{NetlinkHeader, NetlinkMessage};

        if !self.has_remaining_reads {
            return Ok(None);
        }

        let header = {
            let buf = &mut [0u8; aligned_size_of::<NetlinkHeader<u16>>()];
            self.reader.read(buf).map_err(Error::ErrReadSocket)?;
            let hdr = deserialize::<NetlinkHeader<u16>>(buf).map_err(Error::ErrDeserialize)?;
            let nlmsg_type = T::from(hdr.nlmsg_type);
            NetlinkHeader::clone_with_type(hdr, nlmsg_type)
        };

        if header.has_flags(Flag::Multi) {
            self.has_remaining_reads = true;
        }

        let raw_nlmsg_type: u16 = header.nlmsg_type.clone().into();
        if u16::from(MessageType::Done) == raw_nlmsg_type {
            self.has_remaining_reads = false;
            return Ok(None);
        }

        let payload = {
            let len = header.nlmsg_len as usize - aligned_size_of::<NetlinkHeader<u16>>();
            let buf = &mut vec![0u8; len];
            self.reader.read(buf).map_err(Error::ErrReadSocket)?;
            P::deserialize(buf)?
        };

        Ok(Some(NetlinkMessage { header, payload }))
    }

    /// Attempt to receive a single Netlink message.
    ///
    /// This will return [`None`] if a message header with [`MessageType::Done`]
    /// is received or after a successful read of a message this is not part
    /// part of a multipart message sequence.
    ///
    /// This will be reset when another message is sent, so the same
    /// [`NetlinkStream`] can be used.
    ///
    /// # Errors
    ///
    /// Returns an [`crate::Error`] on failure to read from the underlying
    /// socket file descriptor.
    pub fn recv(&mut self) -> Result<Option<NetlinkMessage>> {
        if !self.has_remaining_reads {
            return Ok(None);
        }

        let buf = &mut [0u8; aligned_size_of::<NetlinkHeader>()];
        self.reader.read(buf).map_err(Error::ErrReadSocket)?;
        let hdr = deserialize::<NetlinkHeader>(buf).map_err(Error::ErrDeserialize)?;

        if hdr.has_flags(Flag::Multi) {
            self.has_remaining_reads = true;
        }

        if hdr.has_type(MessageType::Done) {
            self.has_remaining_reads = false;
            return Ok(None);
        }

        if hdr.len == 0 {
            let descriptor = hdr.into_descriptor();
            return Ok(Some(NetlinkMessage::new(descriptor, Vec::new())));
        }

        let payload_len = hdr.len as usize - aligned_size_of::<NetlinkHeader>();
        let mut payload = vec![0u8; payload_len];
        self.reader
            .read(&mut payload)
            .map_err(Error::ErrReadSocket)?;

        let descriptor = hdr.into_descriptor();
        Ok(Some(NetlinkMessage::new(descriptor, payload)))
    }
}

impl Iterator for NetlinkStream {
    type Item = Result<NetlinkMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv().transpose()
    }
}
