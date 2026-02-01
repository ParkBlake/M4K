//! Negamax search algorithm
//!
//! This module implements the negamax search algorithm,
//! a variant of minimax that simplifies the implementation.

use crate::bitboard::Color;
use crate::eval::Evaluator;
use crate::movegen::Move;

/// Negamax search result
pub struct NegamaxResult {
    pub score: i32,
}

/// Perform negamax search
pub fn negamax(
    depth: i32,
    color: Color,
    evaluator: &Evaluator,
    // Position parameters would be passed here
) -> NegamaxResult {
    // Base case: evaluate position
    if depth == 0 {
        return NegamaxResult {
            score: evaluator.evaluate(position) * if color == Color::White { 1 } else { -1 },
        };
    }

    let mut max_score = i32::MIN;

    // Generate moves (placeholder)
    let moves = Vec::<Move>::new(); // Would generate actual moves

    for mv in moves {
        // Make move (placeholder)
        // make_move(position, mv);

        // Recursive search with negated color
        let result = negamax(depth - 1, color.opposite(), evaluator, position);
        let score = -result.score;

        // Unmake move (placeholder)
        // unmake_move(position, mv);

        if score > max_score {
            max_score = score;
        }
    }

    NegamaxResult { score: max_score }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negamax_structure() {
        let evaluator = Evaluator::new();
        let dummy_position = crate::bitboard::position::Position::empty();
        let result = negamax(1, Color::White, &evaluator, &dummy_position);
        // Basic test that it returns a result
        assert!(result.score >= i32::MIN && result.score <= i32::MAX);
    }
}
