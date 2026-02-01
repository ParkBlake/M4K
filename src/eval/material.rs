//! Material evaluation - Evaluate material balance
//!
//! This module provides functions to evaluate the material balance
//! in a chess position, assigning values to different pieces.

use crate::bitboard::{Bitboard, Piece};

/// Piece values in centipawns (hundredths of a pawn)
pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000; // Very high value for king

/// Array of piece values indexed by Piece enum
pub const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE,   // Pawn
    KNIGHT_VALUE, // Knight
    BISHOP_VALUE, // Bishop
    ROOK_VALUE,   // Rook
    QUEEN_VALUE,  // Queen
    KING_VALUE,   // King
];

/// Evaluate material balance for a position
///
/// Returns the material score from the perspective of the side to move.
/// Positive scores favor white, negative scores favor black.
pub fn evaluate_material(
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
    let white_material = count_pieces(white_pawns) * PAWN_VALUE
        + count_pieces(white_knights) * KNIGHT_VALUE
        + count_pieces(white_bishops) * BISHOP_VALUE
        + count_pieces(white_rooks) * ROOK_VALUE
        + count_pieces(white_queens) * QUEEN_VALUE
        + count_pieces(white_king) * KING_VALUE;

    let black_material = count_pieces(black_pawns) * PAWN_VALUE
        + count_pieces(black_knights) * KNIGHT_VALUE
        + count_pieces(black_bishops) * BISHOP_VALUE
        + count_pieces(black_rooks) * ROOK_VALUE
        + count_pieces(black_queens) * QUEEN_VALUE
        + count_pieces(black_king) * KING_VALUE;

    white_material - black_material
}

/// Count the number of pieces on a bitboard
#[inline(always)]
fn count_pieces(bb: Bitboard) -> i32 {
    bb.count() as i32
}

/// Get the value of a piece
#[inline(always)]
pub fn piece_value(piece: Piece) -> i32 {
    PIECE_VALUES[piece as usize]
}

/// Check if a position has sufficient material for mate
pub fn has_mating_material(
    white_pawns: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_rooks: Bitboard,
    white_queens: Bitboard,
    black_pawns: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_rooks: Bitboard,
    black_queens: Bitboard,
) -> bool {
    // Check if either side has pawns
    if !white_pawns.is_empty() || !black_pawns.is_empty() {
        return true;
    }

    // Check if either side has queens
    if !white_queens.is_empty() || !black_queens.is_empty() {
        return true;
    }

    // Check if either side has rooks
    if !white_rooks.is_empty() || !black_rooks.is_empty() {
        return true;
    }

    // Check for sufficient minor pieces
    let white_minors = count_pieces(white_knights) + count_pieces(white_bishops);
    let black_minors = count_pieces(black_knights) + count_pieces(black_bishops);

    // King vs king is insufficient
    if white_minors == 0 && black_minors == 0 {
        return false;
    }

    // One minor piece is usually insufficient (except bishop vs bishop of opposite colors)
    // But we'll be conservative and say it's sufficient unless both sides have no pieces
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{Bitboard, Square};

    #[test]
    fn test_piece_values() {
        assert_eq!(piece_value(Piece::Pawn), PAWN_VALUE);
        assert_eq!(piece_value(Piece::Queen), QUEEN_VALUE);
        assert_eq!(piece_value(Piece::King), KING_VALUE);
    }

    #[test]
    fn test_material_evaluation() {
        // Starting position material balance should be 0
        let white_pawns = Bitboard(0x0000_0000_0000_FF00);
        let white_knights = Bitboard(0x0000_0000_0000_0042);
        let white_bishops = Bitboard(0x0000_0000_0000_0024);
        let white_rooks = Bitboard(0x0000_0000_0000_0081);
        let white_queens = Bitboard(0x0000_0000_0000_0008);
        let white_king = Bitboard(0x0000_0000_0000_0010);

        let black_pawns = Bitboard(0x00FF_0000_0000_0000);
        let black_knights = Bitboard(0x4200_0000_0000_0000);
        let black_bishops = Bitboard(0x2400_0000_0000_0000);
        let black_rooks = Bitboard(0x8100_0000_0000_0000);
        let black_queens = Bitboard(0x0800_0000_0000_0000);
        let black_king = Bitboard(0x1000_0000_0000_0000);

        let score = evaluate_material(
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
        );

        assert_eq!(score, 0);
    }

    #[test]
    fn test_mating_material() {
        // King vs king
        assert!(!has_mating_material(
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY
        ));

        // King and pawn vs king
        let mut white_pawns = Bitboard::EMPTY;
        white_pawns.set(Square::E4);
        assert!(has_mating_material(
            white_pawns,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY
        ));
    }
}
