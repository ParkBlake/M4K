//! Legal move validation - Ensure moves don't leave king in check
//!
//! This module validates pseudo-legal moves to ensure they are actually legal
//! by checking that the king is not left in check after the move.

use crate::bitboard::attacks::*;
use crate::bitboard::{Bitboard, Color, Piece, Square, CastleRights};
use super::generator::{Move, MoveList};

/// Compute attacks by enemy pieces
fn compute_enemy_attacks(position: &crate::bitboard::position::Position, enemy_color: Color) -> Bitboard {
    use crate::bitboard::{attacks, Piece};
    let occupied = (0..6).fold(Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::White)
            | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::Black)
    });

    let mut enemy_attacks = Bitboard::EMPTY;

    // Pawn attacks
    let enemy_pawns = position.piece_bb(Piece::Pawn, enemy_color);
    for sq in enemy_pawns.iter() {
        enemy_attacks |= attacks::pawn_attacks(sq, enemy_color);
    }

    // Knight attacks
    let enemy_knights = position.piece_bb(Piece::Knight, enemy_color);
    for sq in enemy_knights.iter() {
        enemy_attacks |= attacks::knight_attacks(sq);
    }

    // Bishop attacks
    let enemy_bishops = position.piece_bb(Piece::Bishop, enemy_color);
    for sq in enemy_bishops.iter() {
        enemy_attacks |= attacks::bishop_attacks(sq, occupied);
    }

    // Rook attacks
    let enemy_rooks = position.piece_bb(Piece::Rook, enemy_color);
    for sq in enemy_rooks.iter() {
        enemy_attacks |= attacks::rook_attacks(sq, occupied);
    }

    // Queen attacks
    let enemy_queens = position.piece_bb(Piece::Queen, enemy_color);
    for sq in enemy_queens.iter() {
        enemy_attacks |= attacks::queen_attacks(sq, occupied);
    }

    // King attacks
    let enemy_king = position.piece_bb(Piece::King, enemy_color);
    for sq in enemy_king.iter() {
        enemy_attacks |= attacks::king_attacks(sq);
    }

    enemy_attacks
}

/// Check if a move is legal in the current position
///
/// This function assumes the move is pseudo-legal and checks if it leaves
/// the king in check.
pub fn is_legal_move(
    mv: Move,
    position: &crate::bitboard::position::Position,
    color: Color,
) -> bool {
    // For most moves, we just need to check if our king is attacked after the move
    // For castling, we need additional checks

    match mv.move_type() {
        super::generator::MoveType::Castling => {
            is_legal_castling(mv, position, color)
        }
        _ => {
            // Simulate the move and check if king is safe
            let king_safe = simulate_move_and_check_king(mv, position, color);
            king_safe
        }
    }
}

/// Check if castling is legal
fn is_legal_castling(
    mv: Move,
    position: &crate::bitboard::position::Position,
    color: Color,
) -> bool {
    let from = mv.from();
    let to = mv.to();
    let king_square = position.piece_bb(Piece::King, color).lsb().unwrap();
    let enemy_attacks = compute_enemy_attacks(position, color.opposite());

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
    position: &crate::bitboard::position::Position,
    color: Color,
) -> bool {
    // Clone the position, make the move, then check if king is attacked
    let mut new_position = position.clone();
    let undo = new_position.make_move(mv);

    // Compute enemy attacks in the new position
    let enemy_attacks = compute_enemy_attacks(&new_position, color.opposite());

    // Get our king square in the new position
    let king_square = new_position.piece_bb(Piece::King, color).lsb().unwrap();

    // Check if king is attacked
    let king_safe = !enemy_attacks.is_occupied(king_square);

    // Unmake the move
    new_position.unmake_move(undo);

    king_safe
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
    position: &crate::bitboard::position::Position,
    color: Color,
) -> MoveList {
    let mut legal = MoveList::new();

    for &mv in pseudo_legal.iter() {
        if is_legal_move(mv, position, color) {
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
