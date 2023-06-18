use nix::errno::Errno;
use nix::sys::socket::{
    bind, recvmsg, send, socket, AddressFamily, MsgFlags, NetlinkAddr, RecvMsg, SockFlag,
    SockProtocol, SockType,
};
use nix::unistd::getpid;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::IoSliceMut;
use std::os::fd::RawFd;
use crate::{Error, Message, Result, aligned_size_of, Header};

/// Create a connection to a Netlink socket
pub fn connect() -> Result<Conn> {
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

    Ok(Conn {
        sock: nfd,
        seq: 0,
        pid,
    })
}

/// A stateful connection to a Netlink socket.
pub struct Conn {
    sock: RawFd,
    seq: u32,
    pid: u32,
}

impl Conn {
    pub fn send<T: Serialize>(&self, message: &Message<T>) -> Result<()> {
        let bytes = bincode::serialize(&message).map_err(Error::ErrSerialize)?;
        self.send_bytes(&bytes)?;
        Ok(())
    }

    pub fn recv(&self) -> Result<MessageReader> {
        let bytes = self.recv_bytes_until_block()?.to_vec();
        Ok(MessageReader::new(bytes))
    }

    fn send_bytes(&self, data: &[u8]) -> Result<()> {
        let mut remaining = data;

        loop {
            let sent = send(self.sock, &remaining, MsgFlags::empty()).map_err(Error::ErrSendSocket)?;
            if sent == remaining.len() {
                break;
            }
            remaining = &data[sent..];
        }

        Ok(())
    }

    // Read data from the socket until it blocks. This is detected by an error
    // reading from the socket, so it assumes the socket has been opened with
    // the SOCK_NONBLOCK flag.
    fn recv_bytes_until_block(&self) -> Result<Vec<u8>> {
        let mut bytes = vec![];

        loop {
            let chunk = self.recv_bytes();
            match chunk {
                Ok(mut chunk) => bytes.append(&mut chunk),
                Err(Error::ErrRecvSocket(Errno::EAGAIN)) => break,
                Err(err) => return Err(err),
            }
        }

        Ok(bytes)
    }

    fn recv_bytes(&self) -> Result<Vec<u8>> {
        let mut buf_a = vec![0u8; 2048];
        let iov_a = IoSliceMut::new(&mut buf_a);

        let iov = &mut [iov_a];
        let msghdr: RecvMsg<NetlinkAddr> =
            recvmsg(self.sock, iov, None, MsgFlags::empty()).map_err(Error::ErrRecvSocket)?;

        let buf = msghdr.iovs().next().ok_or(Error::ErrRecvSocketNoBuf)?;
        Ok(buf.to_vec())
    }
}

pub struct MessageReader {
    bytes: Vec<u8>,
    cursor: usize,
}

impl MessageReader {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub fn try_next(&mut self) -> Result<Message<&[u8]>> {
        let header = self.try_read::<Header>()?;
        let payload_len = header.len as usize - aligned_size_of::<Header>();
        let payload = self.try_read_bytes(payload_len)?;
        Ok(Message::new(header, payload))
    }

    fn try_read<D: DeserializeOwned>(&mut self) -> Result<D> {
        let len = aligned_size_of::<D>();
        let bytes = self.try_read_bytes(len)?;
        bincode::deserialize(bytes).map_err(Error::ErrDeserialize)
    }

    fn try_read_bytes(&mut self, len: usize) -> Result<&[u8]> {
        if self.cursor + len > self.bytes.len() {
            return Err(Error::ErrUnexpectedEof);
        }
        let bytes = &self.bytes[self.cursor .. self.cursor + len];
        self.cursor += len;
        Ok(bytes)
    }
}