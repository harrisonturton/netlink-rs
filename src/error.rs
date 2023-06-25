
/// Convenience return type for fallible Netlink methods.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that might go wrong when trying to pack Netlink packets and send
/// them to the kernel.
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
    #[error("failed to if_nametoindex with errno {0}")]
    ErrNameToIndex(nix::errno::Errno),
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
    #[error("failed to build with error {0}")]
    ErrBuild(derive_builder::UninitializedFieldError),
    #[error("failed to case to enum")]
    ErrCastEnum(u16),
    #[error("failed to deserialize route attribute {0:?}")]
    ErrDeserializeRouteAttr(crate::route::route::RouteAttrType),
    #[error("failued to convert value")]
    ErrValueConversion,
}

impl From<derive_builder::UninitializedFieldError> for Error {
    fn from(err: derive_builder::UninitializedFieldError) -> Self {
        Self::ErrBuild(err)
    }
}