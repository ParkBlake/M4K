//! Main evaluator - Combine all evaluation components
//!
//! This module provides the main Evaluator struct that combines all
//! evaluation components into a complete position evaluation.

use super::material::*;
use crate::bitboard::{Bitboard, Color};

/// Main position evaluator
pub struct Evaluator {
    // Evaluation parameters could be stored here
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Evaluator {}
    }

    /// Evaluate a position from the perspective of the side to move
    ///
    /// Returns a score in centipawns where positive scores favor the side to move.
    pub fn evaluate(&self, position: &crate::bitboard::position::Position) -> i32 {
        use crate::bitboard::{Color, Piece};
        use crate::eval::{
            king_safety::evaluate_king_safety, material::evaluate_material,
            pawn::evaluate_pawn_structure, pst::evaluate_pst,
        };

        // Extract bitboards for each piece and color
        let wp = position.piece_bb(Piece::Pawn, Color::White);
        let wn = position.piece_bb(Piece::Knight, Color::White);
        let wb = position.piece_bb(Piece::Bishop, Color::White);
        let wr = position.piece_bb(Piece::Rook, Color::White);
        let wq = position.piece_bb(Piece::Queen, Color::White);
        let wk = position.piece_bb(Piece::King, Color::White);

        let bp = position.piece_bb(Piece::Pawn, Color::Black);
        let bn = position.piece_bb(Piece::Knight, Color::Black);
        let bb = position.piece_bb(Piece::Bishop, Color::Black);
        let br = position.piece_bb(Piece::Rook, Color::Black);
        let bq = position.piece_bb(Piece::Queen, Color::Black);
        let bk = position.piece_bb(Piece::King, Color::Black);

        // Material
        let material = evaluate_material(wp, wn, wb, wr, wq, wk, bp, bn, bb, br, bq, bk);

        // Piece-square tables
        let pst = evaluate_pst(
            wp,
            wn,
            wb,
            wr,
            wq,
            wk.lsb().unwrap_or(crate::bitboard::Square::E1),
            bp,
            bn,
            bb,
            br,
            bq,
            bk.lsb().unwrap_or(crate::bitboard::Square::E8),
        );

        // Pawn structure
        let pawn_structure = evaluate_pawn_structure(wp, bp);

        // King safety
        let king_safety = evaluate_king_safety(
            wk.lsb().unwrap_or(crate::bitboard::Square::E1),
            bk.lsb().unwrap_or(crate::bitboard::Square::E8),
            wp,
            bp,
        );

        // Mobility
        let mobility = Self::evaluate_mobility(position);

        // Weighted sum (best-practice weights, can be tuned)
        // Material: 1.0, PST: 0.2, Pawn structure: 0.15, King safety: 0.15, Mobility: 0.1
        let eval = (material as f32)
            + 0.2 * (pst as f32)
            + 0.15 * (pawn_structure as f32)
            + 0.15 * (king_safety as f32)
            + 0.1 * (mobility as f32);

        // Return from the perspective of the side to move
        if position.side_to_move == Color::White {
            eval.round() as i32
        } else {
            (-eval).round() as i32
        }
    }

    /// Evaluate mobility for both sides (difference in number of pseudo-legal moves)
    pub fn evaluate_mobility(position: &crate::bitboard::position::Position) -> i32 {
        use crate::bitboard::{Bitboard, Color, Piece, Square};
        use crate::movegen::generator::*;
        use crate::movegen::MoveList;

        // Helper to count moves for a color
        fn count_moves(position: &crate::bitboard::position::Position, color: Color) -> i32 {
            let mut moves = MoveList::new();
            let occupied = (0..6).fold(Bitboard::EMPTY, |acc, p| {
                acc | position.piece_bb(Piece::from_u8(p).unwrap(), Color::White)
                    | position.piece_bb(Piece::from_u8(p).unwrap(), Color::Black)
            });
            let enemies = (0..6).fold(Bitboard::EMPTY, |acc, p| {
                acc | position.piece_bb(Piece::from_u8(p).unwrap(), color.opposite())
            });

            let pawns = position.piece_bb(Piece::Pawn, color);
            let knights = position.piece_bb(Piece::Knight, color);
            let bishops = position.piece_bb(Piece::Bishop, color);
            let rooks = position.piece_bb(Piece::Rook, color);
            let queens = position.piece_bb(Piece::Queen, color);
            let king = position.piece_bb(Piece::King, color);

            generate_pawn_moves(
                &mut moves,
                pawns,
                occupied,
                enemies,
                color,
                position.en_passant,
            );
            generate_knight_moves(&mut moves, knights, occupied, enemies);
            generate_bishop_moves(&mut moves, bishops, occupied, enemies);
            generate_rook_moves(&mut moves, rooks, occupied, enemies);
            generate_queen_moves(&mut moves, queens, occupied, enemies);
            if let Some(king_sq) = king.lsb() {
                generate_king_moves(&mut moves, king_sq, occupied, enemies);
            }
            moves.len() as i32
        }

        let white_moves = count_moves(position, Color::White);
        let black_moves = count_moves(position, Color::Black);

        white_moves - black_moves
    }

    /// Evaluate material balance only
    pub fn evaluate_material_only(
        &self,
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
        evaluate_material(
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
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::new();
        // Test that evaluation returns a reasonable score
        let dummy_position = crate::bitboard::position::Position::empty();
        let score = evaluator.evaluate(&dummy_position);
        assert!(score >= -20000 && score <= 20000); // Within reasonable bounds
    }
}
