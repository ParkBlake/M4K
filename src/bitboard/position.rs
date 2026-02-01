use crate::bitboard::{Bitboard, CastleRights, Color, Piece, Square};
use crate::utils::zobrist::ZobristHash;
use std::fmt;

/// Represents the full state of a chess position.
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
