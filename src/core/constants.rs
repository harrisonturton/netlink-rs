/// Must be set of all request messages
pub const REQUEST: u16 = 0x1;

/// This message is part of a multipart message terminated by a message with
/// type [`MessageType::Done`]
pub const MULTI: u16 = 0x2;

/// Acknowledgement of success
pub const ACK: u16 = 0x4;

/// Return the complete table instead of a single entry
pub const ROOT: u16 = 0x100;

/// Return all entries matching the criteria passed in the message content
pub const MATCH: u16 = 0x200;

/// Return an atomic snapshot of the table. Required the `CAP_NET_ADMIN`
/// capability or an effective UID of 0.
pub const ATOMIC: u16 = 0x400;

/// Return all entries. Convenience macro, equivalent to
/// [`MessageFlags::Root`] OR [`MessageFlags::Match`].
pub const DUMP: u16 = ROOT | MATCH;

/// Replace an existing object
pub const REPLACE: u16 = 0x100;

/// Don't replace if the object already exists (see [`MessageFlags::Replace`])
pub const EXCL: u16 = 0x200;

/// Create object if it doesn't already exist
pub const CREATE: u16 = 0x400;

/// Add to the end of the object list
pub const APPEND: u16 = 0x800;
