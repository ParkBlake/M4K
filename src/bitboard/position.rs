/*!
    Position module - Complete chess position representation

    This module provides the Position struct, which encapsulates the full state of a chess board,
    including piece placement, side to move, castling rights, en passant, and move counters.
*/

use crate::bitboard::{Bitboard, CastleRights, Color, Piece, Square};
use crate::utils::zobrist::ZobristHash;
use std::fmt;
use std::str::FromStr;

/// Position struct - encapsulates the full chess board state
#[derive(Clone, PartialEq, Eq)]
pub struct Position {
    /// Bitboards for each piece type and color: [piece][color]
    pub pieces: [[Bitboard; 2]; 6],
    /// Side to move
    pub side_to_move: Color,
    /// Castling rights
    pub castling_rights: CastleRights,
    /// En passant square (if any)
    pub en_passant: Option<Square>,
    /// Halfmove clock (for 50-move rule)
    pub halfmove_clock: u32,
    /// Fullmove number (starts at 1, incremented after Black's move)
    pub fullmove_number: u32,
}

impl Position {
    /// Create a new empty position (for setup or testing)
    pub fn empty() -> Self {
        Self {
            pieces: [[Bitboard::EMPTY; 2]; 6],
            side_to_move: Color::White,
            castling_rights: CastleRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Parse a FEN string and set the position accordingly.
    pub fn set_fen(&mut self, fen: &str) -> Result<(), String> {
        let parts: Vec<&str> = fen.trim().split_whitespace().collect();
        if parts.len() < 4 {
            return Err("FEN string must have at least 4 fields".to_string());
        }

        // Clear the board
        *self = Position::empty();

        // Piece placement
        let mut rank = 7;
        let mut file = 0;
        for c in parts[0].chars() {
            match c {
                '/' => {
                    if file != 8 {
                        return Err("Invalid FEN: not enough squares in rank".to_string());
                    }
                    rank = rank.checked_sub(1).ok_or("Too many ranks in FEN")?;
                    file = 0;
                }
                '1'..='8' => {
                    file += c.to_digit(10).unwrap() as u8;
                }
                'p' | 'n' | 'b' | 'r' | 'q' | 'k' | 'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => {
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let piece = match c.to_ascii_lowercase() {
                        'p' => Piece::Pawn,
                        'n' => Piece::Knight,
                        'b' => Piece::Bishop,
                        'r' => Piece::Rook,
                        'q' => Piece::Queen,
                        'k' => Piece::King,
                        _ => return Err(format!("Invalid piece char: {}", c)),
                    };
                    if file > 7 || rank > 7 {
                        return Err("Invalid FEN: file or rank out of bounds".to_string());
                    }
                    self.set_piece(piece, color, Square::new(file, rank));
                    file += 1;
                }
                _ => return Err(format!("Invalid FEN char: {}", c)),
            }
        }
        if rank != 0 || file != 8 {
            return Err("Invalid FEN: not all squares filled".to_string());
        }

        // Side to move
        self.side_to_move = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid FEN: side to move".to_string()),
        };

        // Castling rights
        self.castling_rights = CastleRights(0);
        if parts[2] != "-" {
            for c in parts[2].chars() {
                match c {
                    'K' => self.castling_rights.add(CastleRights::WHITE_KING),
                    'Q' => self.castling_rights.add(CastleRights::WHITE_QUEEN),
                    'k' => self.castling_rights.add(CastleRights::BLACK_KING),
                    'q' => self.castling_rights.add(CastleRights::BLACK_QUEEN),
                    _ => return Err(format!("Invalid castling char: {}", c)),
                }
            }
        }

        // En passant
        self.en_passant = if parts[3] == "-" {
            None
        } else {
            let bytes = parts[3].as_bytes();
            if bytes.len() != 2 {
                return Err("Invalid FEN: en passant square".to_string());
            }
            let file = bytes[0];
            let rank = bytes[1];
            let file_idx = match file {
                b'a'..=b'h' => file - b'a',
                _ => return Err("Invalid FEN: en passant file".to_string()),
            };
            let rank_idx = match rank {
                b'1'..=b'8' => rank - b'1',
                _ => return Err("Invalid FEN: en passant rank".to_string()),
            };
            Some(Square::new(file_idx, rank_idx))
        };

        // Halfmove clock
        self.halfmove_clock = if parts.len() > 4 {
            parts[4].parse().unwrap_or(0)
        } else {
            0
        };

        // Fullmove number
        self.fullmove_number = if parts.len() > 5 {
            parts[5].parse().unwrap_or(1)
        } else {
            1
        };

        Ok(())
    }

    /// Generate a FEN string from the current position.
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Piece placement
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = Square::new(file, rank);
                let mut found = false;
                for piece in 0..6 {
                    for color in 0..2 {
                        if self.pieces[piece][color].is_occupied(sq) {
                            if empty > 0 {
                                fen.push_str(&empty.to_string());
                                empty = 0;
                            }
                            let symbol =
                                match (Piece::from_u8(piece as u8), Color::from_u8(color as u8)) {
                                    (Some(Piece::Pawn), Color::White) => 'P',
                                    (Some(Piece::Pawn), Color::Black) => 'p',
                                    (Some(Piece::Knight), Color::White) => 'N',
                                    (Some(Piece::Knight), Color::Black) => 'n',
                                    (Some(Piece::Bishop), Color::White) => 'B',
                                    (Some(Piece::Bishop), Color::Black) => 'b',
                                    (Some(Piece::Rook), Color::White) => 'R',
                                    (Some(Piece::Rook), Color::Black) => 'r',
                                    (Some(Piece::Queen), Color::White) => 'Q',
                                    (Some(Piece::Queen), Color::Black) => 'q',
                                    (Some(Piece::King), Color::White) => 'K',
                                    (Some(Piece::King), Color::Black) => 'k',
                                    _ => '?',
                                };
                            fen.push(symbol);
                            found = true;
                        }
                    }
                }
                if !found {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        fen.push(' ');
        let mut rights = String::new();
        if self.castling_rights.has(CastleRights::WHITE_KING) {
            rights.push('K');
        }
        if self.castling_rights.has(CastleRights::WHITE_QUEEN) {
            rights.push('Q');
        }
        if self.castling_rights.has(CastleRights::BLACK_KING) {
            rights.push('k');
        }
        if self.castling_rights.has(CastleRights::BLACK_QUEEN) {
            rights.push('q');
        }
        if rights.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&rights);
        }

        // En passant
        fen.push(' ');
        if let Some(ep) = self.en_passant {
            let file = (b'a' + ep.file()) as char;
            let rank = (b'1' + ep.rank()) as char;
            fen.push(file);
            fen.push(rank);
        } else {
            fen.push('-');
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    /// Compute the Zobrist hash for the current position.
    pub fn zobrist_hash(&self) -> ZobristHash {
        use crate::utils::zobrist::{
            ZOBRIST_BLACK_TO_MOVE, ZOBRIST_CASTLE, ZOBRIST_EN_PASSANT, ZOBRIST_PIECE_SQUARE,
        };

        let mut hash = 0u64;

        // Pieces
        for piece in 0..6 {
            for color in 0..2 {
                let mut bb = self.pieces[piece][color];
                while let Some(sq) = bb.pop_lsb() {
                    hash ^= ZOBRIST_PIECE_SQUARE[piece][color][sq.0 as usize];
                }
            }
        }

        // Side to move
        if self.side_to_move == Color::Black {
            hash ^= *ZOBRIST_BLACK_TO_MOVE;
        }

        // Castling rights
        hash ^= ZOBRIST_CASTLE[self.castling_rights.0 as usize];

        // En passant
        if let Some(ep_sq) = self.en_passant {
            hash ^= ZOBRIST_EN_PASSANT[ep_sq.file() as usize];
        }

        ZobristHash(hash)
    }

    /// Place a piece on the board.
    pub fn set_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.pieces[piece as usize][color as usize].set(sq);
    }

    /// Remove a piece from the board.
    pub fn remove_piece(&mut self, piece: Piece, color: Color, sq: Square) {
        self.pieces[piece as usize][color as usize].clear(sq);
    }

    /// Get the bitboard for a given piece and color.
    pub fn piece_bb(&self, piece: Piece, color: Color) -> Bitboard {
        self.pieces[piece as usize][color as usize]
    }

    /// Set up the standard chess starting position.
    pub fn set_startpos(&mut self) {
        use Color::*;
        use Piece::*;
        use Square::*;

        *self = Position::empty();

        // Pawns
        for file in 0..8 {
            self.set_piece(Pawn, White, Square::new(file, 1));
            self.set_piece(Pawn, Black, Square::new(file, 6));
        }
        // Knights
        self.set_piece(Knight, White, B1);
        self.set_piece(Knight, White, G1);
        self.set_piece(Knight, Black, B8);
        self.set_piece(Knight, Black, G8);
        // Bishops
        self.set_piece(Bishop, White, C1);
        self.set_piece(Bishop, White, F1);
        self.set_piece(Bishop, Black, C8);
        self.set_piece(Bishop, Black, F8);
        // Rooks
        self.set_piece(Rook, White, A1);
        self.set_piece(Rook, White, H1);
        self.set_piece(Rook, Black, A8);
        self.set_piece(Rook, Black, H8);
        // Queens
        self.set_piece(Queen, White, D1);
        self.set_piece(Queen, Black, D8);
        // Kings
        self.set_piece(King, White, E1);
        self.set_piece(King, Black, E8);

        self.side_to_move = White;
        self.castling_rights = CastleRights::ALL;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 1;
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board = [['.'; 8]; 8];
        for piece in 0..6 {
            for color in 0..2 {
                let mut bb = self.pieces[piece][color];
                let symbol = match (Piece::from_u8(piece as u8), Color::from_u8(color as u8)) {
                    (Some(Piece::Pawn), Color::White) => 'P',
                    (Some(Piece::Pawn), Color::Black) => 'p',
                    (Some(Piece::Knight), Color::White) => 'N',
                    (Some(Piece::Knight), Color::Black) => 'n',
                    (Some(Piece::Bishop), Color::White) => 'B',
                    (Some(Piece::Bishop), Color::Black) => 'b',
                    (Some(Piece::Rook), Color::White) => 'R',
                    (Some(Piece::Rook), Color::Black) => 'r',
                    (Some(Piece::Queen), Color::White) => 'Q',
                    (Some(Piece::Queen), Color::Black) => 'q',
                    (Some(Piece::King), Color::White) => 'K',
                    (Some(Piece::King), Color::Black) => 'k',
                    _ => '?',
                };
                while let Some(sq) = bb.pop_lsb() {
                    let file = sq.file() as usize;
                    let rank = sq.rank() as usize;
                    board[rank][file] = symbol;
                }
            }
        }
        for rank in (0..8).rev() {
            write!(f, "{} ", rank + 1)?;
            for file in 0..8 {
                write!(f, "{} ", board[rank][file])?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  a b c d e f g h")?;
        writeln!(f, "Side to move: {:?}", self.side_to_move)?;
        writeln!(f, "Castling rights: {:?}", self.castling_rights)?;
        writeln!(f, "En passant: {:?}", self.en_passant)?;
        writeln!(f, "Halfmove clock: {}", self.halfmove_clock)?;
        writeln!(f, "Fullmove number: {}", self.fullmove_number)?;
        Ok(())
    }
}

// Tests for Position
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::{Color, Piece, Square};

    #[test]
    fn test_startpos_zobrist_hash_consistency() {
        let mut pos = Position::empty();
        pos.set_startpos();
        let hash1 = pos.zobrist_hash();
        let hash2 = pos.zobrist_hash();
        assert_eq!(
            hash1, hash2,
            "Zobrist hash should be consistent for startpos"
        );
    }

    #[test]
    fn test_piece_placement_and_removal() {
        let mut pos = Position::empty();
        pos.set_piece(Piece::Knight, Color::White, Square::E4);
        assert_eq!(
            pos.piece_bb(Piece::Knight, Color::White)
                .is_occupied(Square::E4),
            true
        );
        pos.remove_piece(Piece::Knight, Color::White, Square::E4);
        assert_eq!(
            pos.piece_bb(Piece::Knight, Color::White)
                .is_occupied(Square::E4),
            false
        );
    }
}

// Helper trait implementations for Piece and Color
impl Piece {
    pub fn from_u8(val: u8) -> Option<Piece> {
        match val {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => None,
        }
    }
}

impl Color {
    pub fn from_u8(val: u8) -> Color {
        match val {
            0 => Color::White,
            1 => Color::Black,
            _ => Color::White,
        }
    }
}
