//! Move ordering - Order moves for efficient search
//!
//! This module provides move ordering functionality to improve the efficiency
//! of the alpha-beta search by trying the most promising moves first.

use super::generator::{Move, MoveList, MoveType};
use crate::bitboard::{Piece, Square};

/// Move ordering scores for different move types
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MoveScore {
    /// Hash move from transposition table (highest priority)
    Hash = 10000,
    /// Good captures (MVV-LVA)
    GoodCapture = 8000,
    /// Promotions
    Promotion = 6000,
    /// Killer moves (moves that caused cutoffs)
    Killer1 = 5000,
    Killer2 = 4000,
    /// Bad captures (losing material)
    BadCapture = 2000,
    /// Quiet moves with history heuristic
    Quiet = 0,
}

/// Assign a score to a move for ordering purposes
pub fn score_move(
    mv: Move,
    hash_move: Option<Move>,
    killer_moves: &[Move; 2],
    history_table: &[[i32; 64]; 64], // [from][to] history scores
    see_table: &mut SEE, // Static exchange evaluation
) -> i32 {
    // Hash move gets highest priority
    if Some(mv) == hash_move {
        return MoveScore::Hash as i32;
    }

    match mv.move_type() {
        MoveType::Promotion => {
            // Score promotions based on piece value
            let promo_score = match mv.promotion_piece() {
                Piece::Queen => 900,
                Piece::Rook => 500,
                Piece::Bishop => 300,
                Piece::Knight => 300,
                _ => 0,
            };
            MoveScore::Promotion as i32 + promo_score
        }
        MoveType::EnPassant => MoveScore::GoodCapture as i32 + 100, // En passant is usually good
        MoveType::Castling => MoveScore::Quiet as i32 + 50, // Castling is generally good
        MoveType::Normal => {
            // Check if it's a killer move
            if mv == killer_moves[0] {
                return MoveScore::Killer1 as i32;
            }
            if mv == killer_moves[1] {
                return MoveScore::Killer2 as i32;
            }

            // For captures, use MVV-LVA or SEE
            if mv.is_capture(Bitboard::ALL) { // This would need proper occupied board
                // Placeholder: assume good capture for now
                // In real implementation, use SEE to determine if capture is winning/losing
                MoveScore::GoodCapture as i32
            } else {
                // Quiet move: use history heuristic
                let from_idx = mv.from().0 as usize;
                let to_idx = mv.to().0 as usize;
                MoveScore::Quiet as i32 + history_table[from_idx][to_idx]
            }
        }
    }
}

/// Order a list of moves using the given scoring function
pub fn order_moves(
    moves: &mut MoveList,
    hash_move: Option<Move>,
    killer_moves: &[Move; 2],
    history_table: &[[i32; 64]; 64],
    see_table: &mut SEE,
) {
    // Create a vector of (move, score) pairs
    let mut scored_moves: Vec<(Move, i32)> = moves
        .iter()
        .map(|&mv| {
            let score = score_move(mv, hash_move, killer_moves, history_table, see_table);
            (mv, score)
        })
        .collect();

    // Sort by score in descending order (highest scores first)
    scored_moves.sort_by(|a, b| b.1.cmp(&a.1));

    // Update the move list with sorted moves
    for (i, (mv, _)) in scored_moves.into_iter().enumerate() {
        moves[i] = mv;
    }
}

/// Static Exchange Evaluation (SEE) for captures
/// Determines if a capture sequence is winning or losing
pub struct SEE {
    // Placeholder - in a real implementation, this would contain
    // precomputed attack tables and evaluation logic
}

impl SEE {
    pub fn new() -> Self {
        SEE {}
    }

    /// Evaluate if a capture is winning
    pub fn evaluate_capture(&mut self, _mv: Move) -> i32 {
        // Placeholder implementation
        // Real SEE would simulate the capture sequence
        0
    }
}

/// Update history heuristic for a quiet move that caused a cutoff
pub fn update_history(history_table: &mut [[i32; 64]; 64], mv: Move, depth: i32) {
    if mv.move_type() == MoveType::Normal && !mv.is_capture(Bitboard::ALL) {
        let from_idx = mv.from().0 as usize;
        let to_idx = mv.to().0 as usize;
        // Increase history score, with depth-based bonus
        history_table[from_idx][to_idx] += depth * depth;
    }
}

/// Age history table (reduce scores over time)
pub fn age_history(history_table: &mut [[i32; 64]; 64]) {
    for from in 0..64 {
        for to in 0..64 {
            history_table[from][to] /= 2; // Simple aging
        }
    }
}

/// Update killer moves
pub fn update_killers(killer_moves: &mut [Move; 2], mv: Move) {
    if mv.move_type() == MoveType::Normal && !mv.is_capture(Bitboard::ALL) {
        if mv != killer_moves[0] {
            killer_moves[1] = killer_moves[0];
            killer_moves[0] = mv;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::Square;

    #[test]
    fn test_move_scoring() {
        let mut see = SEE::new();
        let mut history = [[0i32; 64]; 64];
        let killers = [Move::new(Square::A1, Square::A2); 2];

        // Test promotion scoring
        let promo_move = Move::promotion(Square::E7, Square::E8, Piece::Queen);
        let score = score_move(promo_move, None, &killers, &history, &mut see);
        assert!(score >= MoveScore::Promotion as i32);

        // Test killer move scoring
        let killer_move = Move::new(Square::A1, Square::A2);
        let score = score_move(killer_move, None, &killers, &history, &mut see);
        assert_eq!(score, MoveScore::Killer1 as i32);
    }

    #[test]
    fn test_history_update() {
        let mut history = [[0i32; 64]; 64];
        let mv = Move::new(Square::E2, Square::E4);

        update_history(&mut history, mv, 3);
        assert!(history[Square::E2.0 as usize][Square::E4.0 as usize] > 0);

        age_history(&mut history);
        assert!(history[Square::E2.0 as usize][Square::E4.0 as usize] >= 0);
    }
}
