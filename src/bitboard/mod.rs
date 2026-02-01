//! Bitboard module - Core board representation using 64-bit integers
//!
//! This module provides the foundation for all board state and piece manipulation.

pub mod types;
pub mod attacks;
pub mod magic;

pub use self::prelude::*;

pub mod prelude {
    pub use super::types::*;
    pub use super::attacks::*;
    pub use super::magic::*;
}

pub mod lib {
    pub use super::*;
}
