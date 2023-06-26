#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::module_inception)]
#![allow(clippy::multiple_inherent_impl)]
// This also checks our dependencies, which we have no control over. Silence
// because there's nothing we can do about it.
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::too_many_lines)]

// netlink(7) implementation
pub mod core;
pub use crate::core::*;

// rnetlink(7) implementation
pub mod route;

pub mod error;
pub use error::*;

pub(crate) mod bytes;
