//! Utilities module - Common helper functions and data structures
//!
//! This module contains various utility functions used throughout the engine,
//! including bit manipulation helpers and Zobrist hashing.

pub mod bits;
pub mod zobrist;

pub use self::prelude::*;

pub mod prelude {
    pub use super::bits::*;
    pub use super::zobrist::*;
}

pub mod lib {
    pub use super::*;
}
