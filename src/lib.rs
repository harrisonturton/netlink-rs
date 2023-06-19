mod core;
pub use crate::core::*;

#[cfg(feature = "route")]
pub mod route;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create socket with errno {0}")]
    ErrCreateSocket(nix::errno::Errno),
    #[error("failed to bind socket with errno {0}")]
    ErrBindSocket(nix::errno::Errno),
    #[error("failed to send to socket with errno {0}")]
    ErrSendSocket(nix::errno::Errno),
    #[error("failed to write to socket with error {0}")]
    ErrWriteSocket(std::io::Error),
    #[error("failed to read from socket with error {0}")]
    ErrReadSocket(std::io::Error),
    #[error("failed to recv from socket with errno {0}")]
    ErrRecvSocket(nix::errno::Errno),
    #[error("socket gather vector had no segments")]
    ErrRecvSocketNoBuf,
    #[error("expected more bytes but there were not enough")]
    ErrUnexpectedEof,
    #[error("failed to serialize with error {0}")]
    ErrSerialize(bincode::Error),
    #[error("failed to deserialize with error {0}")]
    ErrDeserialize(bincode::Error),
    #[error("failed due to missing field {0}")]
    ErrMissingField(String),
}

pub type Result<T> = std::result::Result<T, Error>;
