//! PI5 Chess Engine - High-performance chess engine for Raspberry Pi 5
//!
//! This engine uses ARM Neon SIMD instructions extensively for maximum performance.

#![cfg_attr(target_arch = "aarch64", feature(stdarch_aarch64_neon_intrinsics))]
#![allow(dead_code)]
#![warn(missing_docs)]

// Public modules
pub mod asm;
pub mod bitboard;
pub mod eval;
pub mod movegen;
pub mod search;
pub mod uci;
pub mod utils;

// Re-export commonly used types
pub use bitboard::{Bitboard, Square, Piece, Color, CastleRights};
pub use movegen::{Move, MoveList};
pub use search::SearchEngine;
pub use uci::UciEngine;

/// Project-wide prelude for internal use
pub mod prelude {
    pub use crate::asm::prelude::*;
    pub use crate::bitboard::prelude::*;
    pub use crate::eval::prelude::*;
    pub use crate::movegen::prelude::*;
    pub use crate::search::prelude::*;
    pub use crate::utils::prelude::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        // Ensure basic types are accessible
        let _bb: Bitboard = Bitboard::EMPTY;
        let _sq: Square = Square::A1;
    }
}
