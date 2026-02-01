//! King safety evaluation
//!
//! This module evaluates king safety including pawn shields,
//! open files near the king, and king attacks.

use crate::bitboard::{Bitboard, Color, Square};

/// Evaluate king safety
pub fn evaluate_king_safety(
    white_king: Square,
    black_king: Square,
    white_pawns: Bitboard,
    black_pawns: Bitboard,
) -> i32 {
    let white_safety = evaluate_single_king_safety(white_king, white_pawns, black_pawns, Color::White);
    let black_safety = evaluate_single_king_safety(black_king, black_pawns, white_pawns, Color::Black);

    white_safety - black_safety
}

/// Evaluate king safety for a single king
fn evaluate_single_king_safety(
    king_sq: Square,
    friendly_pawns: Bitboard,
    enemy_pawns: Bitboard,
    color: Color,
) -> i32 {
    let mut score = 0;

    // Pawn shield (3 points per pawn in shield)
    score += evaluate_pawn_shield(king_sq, friendly_pawns, color) * 3;

    // Penalize open files near king
    if king_files_open(king_sq, enemy_pawns) {
        score -= 20;
    }

    // Penalize semi-open files near king
    if king_files_semi_open(king_sq, friendly_pawns, enemy_pawns) {
        score -= 10;
    }

    score
}

/// Evaluate pawn shield around the king
pub fn evaluate_pawn_shield(king_sq: Square, pawns: Bitboard, color: Color) -> i32 {
    let mut shield_score: i32 = 0;
    let king_file = king_sq.file();
    let king_rank = king_sq.rank();

    // Check pawns in front of king (ranks 2-3 for white, 6-7 for black)
    let shield_ranks = if color == Color::White {
        [king_rank + 1, king_rank + 2]
    } else {
        [king_rank - 1, king_rank - 2]
    };

    // Check files: king file and adjacent files
    let shield_files = [
        king_file.saturating_sub(1),
        king_file,
        (king_file + 1).min(7),
    ];

    for &rank in &shield_ranks {
        if rank >= 8 { continue; }
        for &file in &shield_files {
            let sq = Square::new(file, rank);
            if pawns.is_occupied(sq) {
                // Closer pawns are more valuable
                let rank_distance = if color == Color::White {
                    (rank - king_rank) as i32
                } else {
                    (king_rank - rank) as i32
                };
                let file_distance = (file as i32 - king_file as i32).abs();
                shield_score += 2 - rank_distance + (1 - file_distance);
            }
        }
    }

    shield_score
}

/// Check if files near the king are open (no pawns on the file)
pub fn king_files_open(king_sq: Square, enemy_pawns: Bitboard) -> bool {
    let king_file = king_sq.file();

    // Check adjacent files for enemy pawns
    let adjacent_files = [
        king_file.saturating_sub(1),
        king_file,
        (king_file + 1).min(7),
    ];

    for file in adjacent_files {
        let file_bb = Bitboard::file(file);
        if (enemy_pawns & file_bb).is_empty() {
            return true;
        }
    }

    false
}

/// Check if files near the king are semi-open (enemy pawns but no friendly pawns)
fn king_files_semi_open(king_sq: Square, friendly_pawns: Bitboard, enemy_pawns: Bitboard) -> bool {
    let king_file = king_sq.file();

    // Check adjacent files
    let adjacent_files = [
        king_file.saturating_sub(1),
        king_file,
        (king_file + 1).min(7),
    ];

    for file in adjacent_files {
        let file_bb = Bitboard::file(file);
        let friendly_on_file = !(friendly_pawns & file_bb).is_empty();
        let enemy_on_file = !(enemy_pawns & file_bb).is_empty();

        if enemy_on_file && !friendly_on_file {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_safety() {
        let score = evaluate_king_safety(Square::E1, Square::E8, Bitboard::EMPTY, Bitboard::EMPTY);
        assert_eq!(score, 0);
    }
}
