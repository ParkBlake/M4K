//! Search module - Core search algorithms for chess
//!
//! This module implements various search algorithms including:
//! - Negamax search
//! - Alpha-beta pruning
//! - Principal variation search (PVS)
//! - Quiescence search
//! - Transposition table

pub mod alphabeta;
pub mod negamax;
pub mod pvs;
pub mod quiescence;
pub mod transposition;

pub use self::prelude::*;

pub mod prelude {
    pub use super::alphabeta::*;
    pub use super::negamax::*;
    pub use super::pvs::*;
    pub use super::quiescence::*;
    pub use super::transposition::*;
}

pub mod lib {
    pub use super::*;
}
