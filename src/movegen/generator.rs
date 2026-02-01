//! Move generation - Generate all possible moves for a position
//!
//! This module contains the core move generation logic for all piece types.
//! It generates pseudo-legal moves that may need to be validated for legality.

use crate::bitboard::attacks::*;
use crate::bitboard::{Bitboard, Color, Piece, Square};
use arrayvec::ArrayVec;

/// Maximum number of moves in a position (reasonable upper bound)
pub const MAX_MOVES: usize = 256;

/// A chess move encoded in 16 bits
///
/// Bit layout:
/// - 0-5: From square (0-63)
/// - 6-11: To square (0-63)
/// - 12-13: Move type (0=normal, 1=promotion, 2=en passant, 3=castling)
/// - 14-15: Promotion piece (0=queen, 1=rook, 2=bishop, 3=knight) for promotion moves
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move(pub u16);

impl Move {
    /// Create a new normal move
    #[inline(always)]
    pub const fn new(from: Square, to: Square) -> Self {
        Move(((from.0 as u16) << 0) | ((to.0 as u16) << 6))
    }

    /// Create a promotion move
    #[inline(always)]
    pub const fn promotion(from: Square, to: Square, promo_piece: Piece) -> Self {
        let promo_bits = match promo_piece {
            Piece::Queen => 0,
            Piece::Rook => 1,
            Piece::Bishop => 2,
            Piece::Knight => 3,
            _ => 0, // Default to queen
        };
        Move(((from.0 as u16) << 0) | ((to.0 as u16) << 6) | (1 << 12) | (promo_bits << 14))
    }

    /// Create an en passant move
    #[inline(always)]
    pub const fn en_passant(from: Square, to: Square) -> Self {
        Move(((from.0 as u16) << 0) | ((to.0 as u16) << 6) | (2 << 12))
    }

    /// Create a castling move
    #[inline(always)]
    pub const fn castling(from: Square, to: Square) -> Self {
        Move(((from.0 as u16) << 0) | ((to.0 as u16) << 6) | (3 << 12))
    }

    /// Get the from square
    #[inline(always)]
    pub const fn from(self) -> Square {
        Square((self.0 & 0x3F) as u8)
    }

    /// Get the to square
    #[inline(always)]
    pub const fn to(self) -> Square {
        Square(((self.0 >> 6) & 0x3F) as u8)
    }

    /// Get the move type
    #[inline(always)]
    pub const fn move_type(self) -> MoveType {
        match (self.0 >> 12) & 0x3 {
            0 => MoveType::Normal,
            1 => MoveType::Promotion,
            2 => MoveType::EnPassant,
            3 => MoveType::Castling,
            _ => MoveType::Normal,
        }
    }

    /// Get the promotion piece (only valid for promotion moves)
    #[inline(always)]
    pub const fn promotion_piece(self) -> Piece {
        match (self.0 >> 14) & 0x3 {
            0 => Piece::Queen,
            1 => Piece::Rook,
            2 => Piece::Bishop,
            3 => Piece::Knight,
            _ => Piece::Queen,
        }
    }

    /// Check if this is a capture move
    #[inline(always)]
    pub fn is_capture(self, occupied: Bitboard) -> bool {
        occupied.is_occupied(self.to())
    }

    /// Check if this is a promotion move
    pub fn is_promotion(self) -> bool {
        self.move_type() == MoveType::Promotion
    }

    /// Check if this is an en passant move
    pub fn is_en_passant(self) -> bool {
        self.move_type() == MoveType::EnPassant
    }

    /// Check if this is a castling move
    pub fn is_castling(self) -> bool {
        self.move_type() == MoveType::Castling
    }
}

/// Move type enumeration
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MoveType {
    Normal,
    Promotion,
    EnPassant,
    Castling,
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.move_type() {
            MoveType::Normal => write!(f, "{:?}{:?}", self.from(), self.to()),
            MoveType::Promotion => write!(
                f,
                "{:?}{:?}={:?}",
                self.from(),
                self.to(),
                self.promotion_piece()
            ),
            MoveType::EnPassant => write!(f, "{:?}{:?} e.p.", self.from(), self.to()),
            MoveType::Castling => write!(f, "{:?}{:?} castling", self.from(), self.to()),
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// A list of moves with a fixed maximum capacity
#[derive(Clone)]
pub struct MoveList {
    moves: ArrayVec<Move, MAX_MOVES>,
}

impl MoveList {
    /// Create a new empty move list
    pub fn new() -> Self {
        MoveList {
            moves: ArrayVec::new(),
        }
    }

    /// Add a move to the list
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        self.moves.push(mv);
    }

    /// Get the number of moves in the list
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Check if the list is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Clear the move list
    #[inline(always)]
    pub fn clear(&mut self) {
        self.moves.clear();
    }

    /// Get a move by index
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<&Move> {
        self.moves.get(index)
    }

    /// Iterate over moves
    pub fn iter(&self) -> std::slice::Iter<Move> {
        self.moves.iter()
    }

    /// Iterate over moves mutably
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Move> {
        self.moves.iter_mut()
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.moves[index]
    }
}

impl std::ops::IndexMut<usize> for MoveList {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.moves[index]
    }
}

/// Generate all pseudo-legal moves for pawns
pub fn generate_pawn_moves(
    moves: &mut MoveList,
    pawns: Bitboard,
    occupied: Bitboard,
    enemies: Bitboard,
    color: Color,
    en_passant: Option<Square>,
) {
    let (direction, start_rank, promotion_rank) = match color {
        Color::White => (8, 1, 6),  // White moves up (increasing rank)
        Color::Black => (-8, 6, 1), // Black moves down (decreasing rank)
    };

    for pawn_sq in pawns.iter() {
        let pawn_rank = pawn_sq.rank();

        // Single push
        let target_sq = Square((pawn_sq.0 as i32 + direction) as u8);
        if !occupied.is_occupied(target_sq) {
            if pawn_rank == promotion_rank {
                // Promotion moves
                moves.push(Move::promotion(pawn_sq, target_sq, Piece::Queen));
                moves.push(Move::promotion(pawn_sq, target_sq, Piece::Rook));
                moves.push(Move::promotion(pawn_sq, target_sq, Piece::Bishop));
                moves.push(Move::promotion(pawn_sq, target_sq, Piece::Knight));
            } else {
                moves.push(Move::new(pawn_sq, target_sq));
            }

            // Double push from starting position
            if pawn_rank == start_rank {
                let double_target = Square((pawn_sq.0 as i32 + 2 * direction) as u8);
                if !occupied.is_occupied(double_target) {
                    moves.push(Move::new(pawn_sq, double_target));
                }
            }
        }

        // Captures
        let capture_targets = pawn_attacks(pawn_sq, color) & enemies;
        for capture_sq in capture_targets.iter() {
            if pawn_rank == promotion_rank {
                // Promotion captures
                moves.push(Move::promotion(pawn_sq, capture_sq, Piece::Queen));
                moves.push(Move::promotion(pawn_sq, capture_sq, Piece::Rook));
                moves.push(Move::promotion(pawn_sq, capture_sq, Piece::Bishop));
                moves.push(Move::promotion(pawn_sq, capture_sq, Piece::Knight));
            } else {
                moves.push(Move::new(pawn_sq, capture_sq));
            }
        }

        // En passant
        if let Some(ep_sq) = en_passant {
            if pawn_attacks(pawn_sq, color).is_occupied(ep_sq) {
                moves.push(Move::en_passant(pawn_sq, ep_sq));
            }
        }
    }
}

/// Generate all pseudo-legal moves for knights
pub fn generate_knight_moves(
    moves: &mut MoveList,
    knights: Bitboard,
    occupied: Bitboard,
    enemies: Bitboard,
) {
    for knight_sq in knights.iter() {
        let attacks = knight_attacks(knight_sq) & !occupied;

        for target_sq in attacks.iter() {
            moves.push(Move::new(knight_sq, target_sq));
        }

        // Captures
        let captures = knight_attacks(knight_sq) & enemies;
        for capture_sq in captures.iter() {
            moves.push(Move::new(knight_sq, capture_sq));
        }
    }
}

/// Generate all pseudo-legal moves for bishops
pub fn generate_bishop_moves(
    moves: &mut MoveList,
    bishops: Bitboard,
    occupied: Bitboard,
    enemies: Bitboard,
) {
    let friends = occupied & !enemies;  // occupied - enemies = friends
    for bishop_sq in bishops.iter() {
        let attacks = bishop_attacks(bishop_sq, occupied) & !friends;

        for target_sq in attacks.iter() {
            moves.push(Move::new(bishop_sq, target_sq));
        }
    }
}

/// Generate all pseudo-legal moves for rooks
pub fn generate_rook_moves(
    moves: &mut MoveList,
    rooks: Bitboard,
    occupied: Bitboard,
    enemies: Bitboard,
) {
    let friends = occupied & !enemies;  // occupied - enemies = friends
    for rook_sq in rooks.iter() {
        let attacks = rook_attacks(rook_sq, occupied) & !friends;

        for target_sq in attacks.iter() {
            moves.push(Move::new(rook_sq, target_sq));
        }
    }
}

/// Generate all pseudo-legal moves for queens
pub fn generate_queen_moves(
    moves: &mut MoveList,
    queens: Bitboard,
    occupied: Bitboard,
    enemies: Bitboard,
) {
    let friends = occupied & !enemies;  // occupied - enemies = friends
    for queen_sq in queens.iter() {
        let attacks = queen_attacks(queen_sq, occupied) & !friends;

        for target_sq in attacks.iter() {
            moves.push(Move::new(queen_sq, target_sq));
        }
    }
}

/// Generate all pseudo-legal moves for kings
pub fn generate_king_moves(
    moves: &mut MoveList,
    king_sq: Square,
    occupied: Bitboard,
    enemies: Bitboard,
) {
    let attacks = king_attacks(king_sq) & !occupied;

    for target_sq in attacks.iter() {
        moves.push(Move::new(king_sq, target_sq));
    }

    // Captures
    let captures = king_attacks(king_sq) & enemies;
    for capture_sq in captures.iter() {
        moves.push(Move::new(king_sq, capture_sq));
    }
}

/// Generate castling moves for the king
pub fn generate_castling_moves(
    moves: &mut MoveList,
    king_sq: Square,
    castle_rights: crate::bitboard::CastleRights,
    occupied: Bitboard,
    color: Color,
) {
    match color {
        Color::White => {
            // Kingside castling
            if castle_rights.has(crate::bitboard::CastleRights::WHITE_KING) {
                let kingside_clear =
                    !occupied.is_occupied(Square::F1) && !occupied.is_occupied(Square::G1);
                if kingside_clear {
                    moves.push(Move::castling(king_sq, Square::G1));
                }
            }

            // Queenside castling
            if castle_rights.has(crate::bitboard::CastleRights::WHITE_QUEEN) {
                let queenside_clear = !occupied.is_occupied(Square::B1)
                    && !occupied.is_occupied(Square::C1)
                    && !occupied.is_occupied(Square::D1);
                if queenside_clear {
                    moves.push(Move::castling(king_sq, Square::C1));
                }
            }
        }
        Color::Black => {
            // Kingside castling
            if castle_rights.has(crate::bitboard::CastleRights::BLACK_KING) {
                let kingside_clear =
                    !occupied.is_occupied(Square::F8) && !occupied.is_occupied(Square::G8);
                if kingside_clear {
                    moves.push(Move::castling(king_sq, Square::G8));
                }
            }

            // Queenside castling
            if castle_rights.has(crate::bitboard::CastleRights::BLACK_QUEEN) {
                let queenside_clear = !occupied.is_occupied(Square::B8)
                    && !occupied.is_occupied(Square::C8)
                    && !occupied.is_occupied(Square::D8);
                if queenside_clear {
                    moves.push(Move::castling(king_sq, Square::C8));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_creation() {
        let mv = Move::new(Square::E2, Square::E4);
        assert_eq!(mv.from(), Square::E2);
        assert_eq!(mv.to(), Square::E4);
        assert_eq!(mv.move_type(), MoveType::Normal);
    }

    #[test]
    fn test_promotion_move() {
        let mv = Move::promotion(Square::E7, Square::E8, Piece::Queen);
        assert_eq!(mv.from(), Square::E7);
        assert_eq!(mv.to(), Square::E8);
        assert_eq!(mv.move_type(), MoveType::Promotion);
        assert_eq!(mv.promotion_piece(), Piece::Queen);
    }

    #[test]
    fn test_en_passant_move() {
        let mv = Move::en_passant(Square::E5, Square::F6);
        assert_eq!(mv.move_type(), MoveType::EnPassant);
    }

    #[test]
    fn test_castling_move() {
        let mv = Move::castling(Square::E1, Square::G1);
        assert_eq!(mv.move_type(), MoveType::Castling);
    }

    #[test]
    fn test_move_list() {
        let mut list = MoveList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());

        list.push(Move::new(Square::E2, Square::E4));
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        assert_eq!(list[0].from(), Square::E2);
    }
}
