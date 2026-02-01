//! Legal move validation - Ensure moves don't leave king in check
//!
//! This module validates pseudo-legal moves to ensure they are actually legal
//! by checking that the king is not left in check after the move.

use crate::bitboard::attacks::*;
use crate::bitboard::{Bitboard, Color, Piece, Square, CastleRights};
use super::generator::{Move, MoveList};

/// Check if a move is legal in the current position
///
/// This function assumes the move is pseudo-legal and checks if it leaves
/// the king in check.
pub fn is_legal_move(
    mv: Move,
    // Position parameters would be passed here
    our_pieces: Bitboard,
    enemy_pieces: Bitboard,
    occupied: Bitboard,
    king_square: Square,
    enemy_attacks: Bitboard, // Precomputed enemy attacks
) -> bool {
    // For most moves, we just need to check if our king is attacked after the move
    // For castling, we need additional checks

    match mv.move_type() {
        super::generator::MoveType::Castling => {
            is_legal_castling(mv, occupied, enemy_attacks, king_square)
        }
        _ => {
            // Simulate the move and check if king is safe
            let king_safe = simulate_move_and_check_king(
                mv,
                our_pieces,
                enemy_pieces,
                occupied,
                king_square,
                enemy_attacks,
            );
            king_safe
        }
    }
}

/// Check if castling is legal
fn is_legal_castling(
    mv: Move,
    occupied: Bitboard,
    enemy_attacks: Bitboard,
    king_square: Square,
) -> bool {
    let from = mv.from();
    let to = mv.to();

    // King must not be in check
    if enemy_attacks.is_occupied(king_square) {
        return false;
    }

    // Squares the king passes through must not be attacked
    match (from, to) {
        (Square::E1, Square::G1) => {
            // White kingside
            !enemy_attacks.is_occupied(Square::F1) && !enemy_attacks.is_occupied(Square::G1)
        }
        (Square::E1, Square::C1) => {
            // White queenside
            !enemy_attacks.is_occupied(Square::D1) && !enemy_attacks.is_occupied(Square::C1)
        }
        (Square::E8, Square::G8) => {
            // Black kingside
            !enemy_attacks.is_occupied(Square::F8) && !enemy_attacks.is_occupied(Square::G8)
        }
        (Square::E8, Square::C8) => {
            // Black queenside
            !enemy_attacks.is_occupied(Square::D8) && !enemy_attacks.is_occupied(Square::C8)
        }
        _ => false,
    }
}

/// Simulate a move and check if the king is safe afterwards
fn simulate_move_and_check_king(
    mv: Move,
    our_pieces: Bitboard,
    enemy_pieces: Bitboard,
    occupied: Bitboard,
    king_square: Square,
    enemy_attacks: Bitboard,
) -> bool {
    // This is a simplified version. In a full implementation, we would:
    // 1. Make the move on a temporary board
    // 2. Recalculate enemy attacks
    // 3. Check if our king is attacked

    // For now, just return true (placeholder)
    // In a real engine, this would be much more complex
    true
}

/// Check if the current position is in check
pub fn is_in_check(
    king_square: Square,
    enemy_attacks: Bitboard,
) -> bool {
    enemy_attacks.is_occupied(king_square)
}

/// Check if the current position is checkmate
pub fn is_checkmate(
    king_square: Square,
    enemy_attacks: Bitboard,
    legal_moves: &MoveList,
) -> bool {
    is_in_check(king_square, enemy_attacks) && legal_moves.is_empty()
}

/// Check if the current position is stalemate
pub fn is_stalemate(
    king_square: Square,
    enemy_attacks: Bitboard,
    legal_moves: &MoveList,
) -> bool {
    !is_in_check(king_square, enemy_attacks) && legal_moves.is_empty()
}

/// Filter a list of pseudo-legal moves to only include legal ones
pub fn filter_legal_moves(
    pseudo_legal: &MoveList,
    // Position parameters
    our_pieces: Bitboard,
    enemy_pieces: Bitboard,
    occupied: Bitboard,
    king_square: Square,
    enemy_attacks: Bitboard,
) -> MoveList {
    let mut legal = MoveList::new();

    for &mv in pseudo_legal.iter() {
        if is_legal_move(
            mv,
            our_pieces,
            enemy_pieces,
            occupied,
            king_square,
            enemy_attacks,
        ) {
            legal.push(mv);
        }
    }

    legal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::Square;

    #[test]
    fn test_check_detection() {
        // King on e1, enemy attacking e1
        let king_sq = Square::E1;
        let mut enemy_attacks = Bitboard::EMPTY;
        enemy_attacks.set(Square::E1);

        assert!(is_in_check(king_sq, enemy_attacks));

        // King not attacked
        let enemy_attacks = Bitboard::EMPTY;
        assert!(!is_in_check(king_sq, enemy_attacks));
    }

    #[test]
    fn test_checkmate_stalemate() {
        let king_sq = Square::E1;
        let enemy_attacks = Bitboard::EMPTY;
        let empty_moves = MoveList::new();

        // Stalemate: not in check, no legal moves
        assert!(is_stalemate(king_sq, enemy_attacks, &empty_moves));
        assert!(!is_checkmate(king_sq, enemy_attacks, &empty_moves));

        // Checkmate: in check, no legal moves
        let mut enemy_attacks = Bitboard::EMPTY;
        enemy_attacks.set(Square::E1);
        assert!(is_checkmate(king_sq, enemy_attacks, &empty_moves));
        assert!(!is_stalemate(king_sq, enemy_attacks, &empty_moves));
    }
}
