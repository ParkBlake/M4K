//! UCI (Universal Chess Interface) module - Communication protocol
//!
//! This module implements the UCI protocol for communication with chess GUIs
//! and other engines, handling commands and responses.

pub mod commands;
pub mod protocol;

pub use self::prelude::*;

pub mod prelude {
    pub use super::commands::*;
    pub use super::protocol::*;
}

pub mod lib {
    pub use super::*;
}
