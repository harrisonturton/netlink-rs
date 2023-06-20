use crate::{MULTI, aligned_size_of, Error, NetlinkHeader, NetlinkMessage, Result, MessageType};
use bincode::deserialize;
use nix::errno::Errno;
use nix::sys::socket::{
    bind, recvmsg, send, socket, AddressFamily, MsgFlags, NetlinkAddr, SockFlag, SockProtocol,
    SockType, RecvMsg,
};
use nix::unistd::getpid;
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, IoSliceMut, Read, Write};
use std::os::fd::RawFd;

pub struct NetlinkSocket {
    fd: RawFd,
    pid: u32,
    seq: u32,
}

impl NetlinkSocket {
    pub fn connect() -> Result<Self> {
        let fd = socket(
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
        bind(fd, &sock_addr).map_err(Error::ErrBindSocket)?;

        Ok(NetlinkSocket { fd, pid, seq: 0 })
    }

    pub fn send<T: Serialize>(&mut self, nlmsg: NetlinkMessage<T>) -> Result<()> {
        let bytes = bincode::serialize(&nlmsg).map_err(Error::ErrSerialize)?;
        self.write_all(&bytes).map_err(Error::ErrWriteSocket)
    }

    pub fn recv(&mut self) -> Result<NetlinkMessageStream> {
        let bytes = self.recv_until_block()?;
        Ok(NetlinkMessageStream::from_bytes(bytes))
    }

    fn recv_until_block(&self) -> Result<Vec<u8>> {
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
            recvmsg(self.fd, iov, None, MsgFlags::empty()).map_err(Error::ErrRecvSocket)?;

        let buf = msghdr.iovs().next().ok_or(Error::ErrRecvSocketNoBuf)?;
        Ok(buf.to_vec())
    }
}

impl std::io::Write for NetlinkSocket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        send(self.fd, &buf, MsgFlags::empty())
            .map(|sent| sent as usize)
            .map_err(|errno| std::io::Error::from_raw_os_error(errno as i32))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Data is sent immediately when write() is called; flushing is not
        // needed. Noop implementation because std::io::Write requires it.
        Ok(())
    }
}

pub struct NetlinkMessageStream {
    bytes: Vec<u8>,
}

impl NetlinkMessageStream {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn messages(&mut self) -> Messages {
        Messages::new(&self.bytes)
    }
}

pub struct Messages<'a> {
    bytes: &'a [u8],
    cursor: usize,
    ended: bool,
}

impl<'a> Messages<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            ended: false,
        }
    }

    pub fn next_chunk(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.cursor + len > self.bytes.len() {
            return Err(Error::ErrUnexpectedEof);
        }
        let bytes = &self.bytes[self.cursor..self.cursor + len];
        self.cursor += len;
        Ok(bytes)
    }

    pub fn next_val<T: Deserialize<'a>>(&mut self) -> Result<T> {
        let len = aligned_size_of::<T>();
        let bytes = self.next_chunk(len)?;
        deserialize(&bytes).map_err(Error::ErrDeserialize)
    }

    pub fn next_message(&mut self) -> Result<NetlinkMessage<&'a [u8]>> {
        let header = self.next_val::<NetlinkHeader>()?;
        let payload_len = header.len as usize - aligned_size_of::<NetlinkHeader>();
        let payload = self.next_chunk(payload_len)?;
        Ok(NetlinkMessage::new(header, payload))
    }
}

impl<'a> Iterator for Messages<'a> {
    type Item = Result<NetlinkMessage<&'a [u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ended {
            return None;
        }

        let msg = match self.next_message() {
            Ok(msg) => msg,
            Err(err) => return Some(Err(err)),
        };

        if msg.header.has_type(MessageType::Done) {
            self.ended = true;
        }

        if !msg.header.has_flags(MULTI) {
            self.ended = true;
        }

        Some(Ok(msg))
    }
}
