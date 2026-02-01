//! Main evaluator - Combine all evaluation components
//!
//! This module provides the main Evaluator struct that combines all
//! evaluation components into a complete position evaluation.

use super::material::*;
use crate::bitboard::{Bitboard, Color};

/// Main position evaluator
pub struct Evaluator {
    // Evaluation parameters could be stored here
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Evaluator {}
    }

    /// Evaluate a position from the perspective of the side to move
    ///
    /// Returns a score in centipawns where positive scores favor the side to move.
    pub fn evaluate(&self, position: &crate::bitboard::position::Position) -> i32 {
        // Placeholder evaluation - just material for now
        // In a real implementation, this would combine:
        // - Material balance
        // - Piece-square tables
        // - Pawn structure
        // - King safety
        // - Mobility
        // - etc.

        // For now, return a neutral score
        0
    }

    /// Evaluate material balance only
    pub fn evaluate_material_only(
        &self,
        white_pawns: Bitboard,
        white_knights: Bitboard,
        white_bishops: Bitboard,
        white_rooks: Bitboard,
        white_queens: Bitboard,
        white_king: Bitboard,
        black_pawns: Bitboard,
        black_knights: Bitboard,
        black_bishops: Bitboard,
        black_rooks: Bitboard,
        black_queens: Bitboard,
        black_king: Bitboard,
    ) -> i32 {
        evaluate_material(
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::new();
        // Test that evaluation returns a reasonable score
        let score = evaluator.evaluate();
        assert!(score >= -20000 && score <= 20000); // Within reasonable bounds
    }
}
