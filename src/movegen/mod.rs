//! Move generation module - Generate legal and pseudo-legal moves
//!
//! This module handles all aspects of move generation, including:
//! - Pseudo-legal move generation
//! - Legal move validation
//! - Move ordering for search efficiency

pub mod generator;
pub mod legal;
pub mod ordering;

pub use self::prelude::*;

pub mod prelude {
    pub use super::generator::*;
    pub use super::legal::*;
    pub use super::ordering::*;
}

pub mod lib {
    pub use super::*;
}
