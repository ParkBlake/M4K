//! Bitboard module - Core board representation using 64-bit integers
//!
//! This module provides the foundation for all board state and piece manipulation.

pub mod attacks;
pub mod magic;
pub mod position;
pub mod position;
pub mod types;

pub use self::prelude::*;

pub mod prelude {
    pub use super::attacks::*;
    pub use super::magic::*;
    pub use super::position::*;
    pub use super::types::*;
}

pub mod lib {
    pub use super::*;
}
