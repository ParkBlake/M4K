//! Piece-square table evaluation
//!
//! This module provides piece-square tables for positional evaluation,
//! giving bonuses to pieces based on their position on the board.

use crate::bitboard::{Bitboard, Color, Piece, Square};

/// Piece-square table for pawns (from white's perspective)
pub const PAWN_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5, 5,
    10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10, -20,
    -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Piece-square table for knights
pub const KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15, 10,
    0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15, 15, 10,
    5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

/// Piece-square table for bishops
pub const BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5, 0,
    -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10, 10, 10,
    -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

/// Piece-square table for rooks
pub const ROOK_PST: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0,
    0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 0, 0,
    0, 5, 5, 0, 0, 0,
];

/// Piece-square table for queens
pub const QUEEN_PST: [i32; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0, 5, 0, 0,
    0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
];

/// Piece-square table for kings (middlegame)
pub const KING_PST: [i32; 64] = [
    -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40,
    -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40, -40, -30,
    -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20, 30, 10, 0, 0,
    10, 30, 20,
];

/// Array of piece-square tables indexed by piece type
pub const PIECE_PST: [[i32; 64]; 6] = [
    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST,
];

/// Get the piece-square table value for a piece on a square
#[inline(always)]
pub fn pst_value(piece: Piece, square: Square, color: Color) -> i32 {
    let table = &PIECE_PST[piece as usize];
    let index = if color == Color::White {
        square.0 as usize
    } else {
        // Flip the square for black
        (square.0 ^ 56) as usize
    };
    table[index]
}

/// Evaluate piece-square table bonuses for all pieces
pub fn evaluate_pst(
    white_pawns: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_rooks: Bitboard,
    white_queens: Bitboard,
    white_king: Square,
    black_pawns: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_rooks: Bitboard,
    black_queens: Bitboard,
    black_king: Square,
) -> i32 {
    let mut score = 0;

    // White pieces
    for sq in white_pawns.iter() {
        score += pst_value(Piece::Pawn, sq, Color::White);
    }
    for sq in white_knights.iter() {
        score += pst_value(Piece::Knight, sq, Color::White);
    }
    for sq in white_bishops.iter() {
        score += pst_value(Piece::Bishop, sq, Color::White);
    }
    for sq in white_rooks.iter() {
        score += pst_value(Piece::Rook, sq, Color::White);
    }
    for sq in white_queens.iter() {
        score += pst_value(Piece::Queen, sq, Color::White);
    }
    score += pst_value(Piece::King, white_king, Color::White);

    // Black pieces (negated because PSTs are from white's perspective)
    for sq in black_pawns.iter() {
        score -= pst_value(Piece::Pawn, sq, Color::Black);
    }
    for sq in black_knights.iter() {
        score -= pst_value(Piece::Knight, sq, Color::Black);
    }
    for sq in black_bishops.iter() {
        score -= pst_value(Piece::Bishop, sq, Color::Black);
    }
    for sq in black_rooks.iter() {
        score -= pst_value(Piece::Rook, sq, Color::Black);
    }
    for sq in black_queens.iter() {
        score -= pst_value(Piece::Queen, sq, Color::Black);
    }
    score -= pst_value(Piece::King, black_king, Color::Black);

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pst_value() {
        // Test center square bonus for knight
        let center_value = pst_value(Piece::Knight, Square::E4, Color::White);
        assert!(center_value > 0);

        // Test that black gets the same bonus on mirrored square
        let black_value = pst_value(Piece::Knight, Square::E5, Color::Black);
        assert_eq!(center_value, black_value);
    }

    #[test]
    fn test_pst_evaluation() {
        // Simple test with one piece each
        let score = evaluate_pst(
            Square::E4.to_bitboard(), // white pawn on good central square
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Square::E1,                        // white king
            Square::H2.to_bitboard(), // black pawn on bad square
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Bitboard::EMPTY,
            Square::E8, // black king
        );

        // Should be positive (white has better position)
        assert!(score > 0);
    }
}
