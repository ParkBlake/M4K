//! Zobrist hashing for chess positions
//!
//! Zobrist hashing is a method for generating unique keys for chess positions
//! that is used in transposition tables and other caching mechanisms.

use crate::bitboard::{Bitboard, CastleRights, Color, Piece, Square};
use once_cell::sync::Lazy;

/// Random 64-bit numbers for Zobrist hashing
///
/// We use a large array of random numbers to ensure minimal collisions.
/// The structure is: [piece][color][square]
static ZOBRIST_PIECE_SQUARE: Lazy<[[[u64; 64]; 2]; 6]> = Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut table = [[[0u64; 64]; 2]; 6];

    for piece in 0..6 {
        for color in 0..2 {
            for square in 0..64 {
                table[piece][color][square] = rng.gen();
            }
        }
    }
    table
});

/// Random number for black to move
static ZOBRIST_BLACK_TO_MOVE: Lazy<u64> = Lazy::new(|| rand::random());

/// Random numbers for castling rights
static ZOBRIST_CASTLE: Lazy<[u64; 16]> = Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut table = [0u64; 16];
    for i in 0..16 {
        table[i] = rng.gen();
    }
    table
});

/// Random numbers for en passant files
static ZOBRIST_EN_PASSANT: Lazy<[u64; 8]> = Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut table = [0u64; 8];
    for i in 0..8 {
        table[i] = rng.gen();
    }
    table
});

/// Zobrist hash for a chess position
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    /// Create a new hash from a position
    pub fn new() -> Self {
        ZobristHash(0)
    }

    /// Update hash when a piece moves from one square to another
    #[inline(always)]
    pub fn move_piece(&mut self, piece: Piece, color: Color, from: Square, to: Square) {
        self.0 ^= ZOBRIST_PIECE_SQUARE[piece as usize][color as usize][from.0 as usize];
        self.0 ^= ZOBRIST_PIECE_SQUARE[piece as usize][color as usize][to.0 as usize];
    }

    /// Update hash when a piece is placed on a square
    #[inline(always)]
    pub fn place_piece(&mut self, piece: Piece, color: Color, square: Square) {
        self.0 ^= ZOBRIST_PIECE_SQUARE[piece as usize][color as usize][square.0 as usize];
    }

    /// Update hash when a piece is removed from a square
    #[inline(always)]
    pub fn remove_piece(&mut self, piece: Piece, color: Color, square: Square) {
        self.0 ^= ZOBRIST_PIECE_SQUARE[piece as usize][color as usize][square.0 as usize];
    }

    /// Update hash when side to move changes
    #[inline(always)]
    pub fn flip_side(&mut self) {
        self.0 ^= *ZOBRIST_BLACK_TO_MOVE;
    }

    /// Update hash when castling rights change
    #[inline(always)]
    pub fn update_castle_rights(&mut self, old_rights: CastleRights, new_rights: CastleRights) {
        self.0 ^= ZOBRIST_CASTLE[old_rights.0 as usize];
        self.0 ^= ZOBRIST_CASTLE[new_rights.0 as usize];
    }

    /// Update hash when en passant square changes
    #[inline(always)]
    pub fn update_en_passant(&mut self, old_ep: Option<Square>, new_ep: Option<Square>) {
        if let Some(ep) = old_ep {
            self.0 ^= ZOBRIST_EN_PASSANT[ep.file() as usize];
        }
        if let Some(ep) = new_ep {
            self.0 ^= ZOBRIST_EN_PASSANT[ep.file() as usize];
        }
    }

    /// Get the hash value
    #[inline(always)]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl From<u64> for ZobristHash {
    fn from(value: u64) -> Self {
        ZobristHash(value)
    }
}

impl std::fmt::Debug for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ZobristHash({:016x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zobrist_hash_creation() {
        let hash = ZobristHash::new();
        assert_eq!(hash.value(), 0);
    }

    #[test]
    fn test_piece_placement() {
        let mut hash = ZobristHash::new();
        let original = hash.value();

        hash.place_piece(Piece::Pawn, Color::White, Square::E4);
        assert_ne!(hash.value(), original);

        // Placing the same piece again should toggle it back
        hash.place_piece(Piece::Pawn, Color::White, Square::E4);
        assert_eq!(hash.value(), original);
    }

    #[test]
    fn test_piece_movement() {
        let mut hash = ZobristHash::new();
        let original = hash.value();

        // Place a piece and move it
        hash.place_piece(Piece::Knight, Color::Black, Square::B1);
        let after_place = hash.value();
        assert_ne!(after_place, original);

        hash.move_piece(Piece::Knight, Color::Black, Square::B1, Square::C3);
        let after_move = hash.value();
        assert_ne!(after_move, after_place);

        // Moving back should restore the hash
        hash.move_piece(Piece::Knight, Color::Black, Square::C3, Square::B1);
        assert_eq!(hash.value(), after_place);
    }

    #[test]
    fn test_side_to_move() {
        let mut hash = ZobristHash::new();
        let original = hash.value();

        hash.flip_side();
        assert_ne!(hash.value(), original);

        // Flipping twice should restore original
        hash.flip_side();
        assert_eq!(hash.value(), original);
    }

    #[test]
    fn test_castle_rights() {
        let mut hash = ZobristHash::new();
        let original = hash.value();

        hash.update_castle_rights(CastleRights::ALL, CastleRights::NONE);
        assert_ne!(hash.value(), original);

        // Updating back should restore
        hash.update_castle_rights(CastleRights::NONE, CastleRights::ALL);
        assert_eq!(hash.value(), original);
    }

    #[test]
    fn test_en_passant() {
        let mut hash = ZobristHash::new();
        let original = hash.value();

        hash.update_en_passant(None, Some(Square::E3));
        assert_ne!(hash.value(), original);

        // Setting the same EP square again should toggle it off
        hash.update_en_passant(Some(Square::E3), Some(Square::E3));
        assert_eq!(hash.value(), original);
    }
}
