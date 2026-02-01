//! Pawn structure evaluation
//!
//! This module evaluates pawn structure including doubled pawns,
//! isolated pawns, passed pawns, and pawn chains.

use crate::bitboard::{Bitboard, Color, Square};

/// Evaluate pawn structure
pub fn evaluate_pawn_structure(white_pawns: Bitboard, black_pawns: Bitboard) -> i32 {
    let white_score = evaluate_single_color_pawns(white_pawns, black_pawns, Color::White);
    let black_score = evaluate_single_color_pawns(black_pawns, white_pawns, Color::Black);

    white_score - black_score
}

/// Evaluate pawn structure for one color
fn evaluate_single_color_pawns(friendly_pawns: Bitboard, enemy_pawns: Bitboard, color: Color) -> i32 {
    let mut score = 0;

    for pawn_sq in friendly_pawns.iter() {
        // Doubled pawns penalty
        if is_doubled_pawn(pawn_sq, friendly_pawns) {
            score -= 15;
        }

        // Isolated pawns penalty
        if is_isolated_pawn(pawn_sq, friendly_pawns) {
            score -= 10;
        }

        // Passed pawns bonus
        if is_passed_pawn(pawn_sq, color, enemy_pawns) {
            let rank = pawn_sq.rank();
            let advancement = if color == Color::White {
                rank as i32
            } else {
                (7 - rank) as i32
            };
            score += 10 + advancement * 5; // Bonus increases with advancement
        }
    }

    score
}

/// Check if a pawn is passed
pub fn is_passed_pawn(pawn_sq: Square, color: Color, enemy_pawns: Bitboard) -> bool {
    let file = pawn_sq.file();
    let rank = pawn_sq.rank();

    // Define the pawn's path to promotion
    let (start_rank, end_rank, direction) = if color == Color::White {
        (rank + 1, 7, 1)
    } else {
        (0, rank - 1, -1)
    };

    // Check adjacent files for enemy pawns that could block
    let adjacent_files = [
        file.saturating_sub(1),
        file,
        (file + 1).min(7),
    ];

    for check_rank in start_rank..=end_rank {
        for &check_file in &adjacent_files {
            let sq = Square::new(check_file, check_rank);
            if enemy_pawns.is_occupied(sq) {
                return false;
            }
        }
    }

    true
}

/// Check if a pawn is isolated
pub fn is_isolated_pawn(pawn_sq: Square, friendly_pawns: Bitboard) -> bool {
    let file = pawn_sq.file();

    // Check if there are friendly pawns on adjacent files
    let left_file = file.saturating_sub(1);
    let right_file = (file + 1).min(7);

    let left_bb = if left_file < file { Bitboard::file(left_file) } else { Bitboard::EMPTY };
    let right_bb = if right_file > file { Bitboard::file(right_file) } else { Bitboard::EMPTY };

    (friendly_pawns & (left_bb | right_bb)).is_empty()
}

/// Check if a pawn is doubled
pub fn is_doubled_pawn(pawn_sq: Square, friendly_pawns: Bitboard) -> bool {
    let file = pawn_sq.file();
    let file_bb = Bitboard::file(file);

    // Count pawns on this file
    (friendly_pawns & file_bb).count() > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pawn_structure() {
        let score = evaluate_pawn_structure(Bitboard::EMPTY, Bitboard::EMPTY);
        assert_eq!(score, 0);
    }
}
