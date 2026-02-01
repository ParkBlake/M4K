//! Evaluation module - Static position evaluation
//!
//! This module provides static evaluation functions for chess positions,
//! including material balance, piece-square tables, pawn structure, and king safety.

pub mod evaluator;
pub mod king_safety;
pub mod material;
pub mod pawn;
pub mod pst;

pub use self::prelude::*;

pub mod prelude {
    pub use super::evaluator::*;
    pub use super::king_safety::*;
    pub use super::material::*;
    pub use super::pawn::*;
    pub use super::pst::*;
}

pub mod lib {
    pub use super::*;
}
