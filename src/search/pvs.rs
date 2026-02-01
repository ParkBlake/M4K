//! Principal Variation Search (PVS) algorithm
//!
//! This module implements Principal Variation Search, an optimization
//! of alpha-beta search that searches the principal variation with full window
//! and other variations with null window.

use crate::bitboard::Color;
use crate::eval::Evaluator;
use crate::movegen::Move;

/// PVS search result
pub struct PvsResult {
    pub score: i32,
    pub pv: Vec<Move>,
}

/// Perform Principal Variation Search
use crate::bitboard::position::Position;
use crate::movegen::generator::*;
use crate::movegen::MoveList;

pub fn pvs_search(
    depth: i32,
    mut alpha: i32,
    beta: i32,
    color: Color,
    evaluator: &Evaluator,
    position: &Position,
) -> PvsResult {
    use crate::movegen::legal::filter_legal_moves;

    // Base case: evaluate position
    if depth == 0 {
        return PvsResult {
            score: evaluator.evaluate(position) * if color == Color::White { 1 } else { -1 },
            pv: Vec::new(),
        };
    }

    let mut best_score = i32::MIN;
    let mut best_pv = Vec::new();
    let mut first_move = true;

    // Generate pseudo-legal moves
    let mut moves = MoveList::new();
    let occupied = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(
            crate::bitboard::Piece::from_u8(p).unwrap(),
            crate::bitboard::Color::White,
        ) | position.piece_bb(
            crate::bitboard::Piece::from_u8(p).unwrap(),
            crate::bitboard::Color::Black,
        )
    });
    let enemies = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(
            crate::bitboard::Piece::from_u8(p).unwrap(),
            color.opposite(),
        )
    });

    generate_pawn_moves(
        &mut moves,
        position.piece_bb(crate::bitboard::Piece::Pawn, color),
        occupied,
        enemies,
        color,
        position.en_passant,
    );
    generate_knight_moves(
        &mut moves,
        position.piece_bb(crate::bitboard::Piece::Knight, color),
        occupied,
        enemies,
    );
    generate_bishop_moves(
        &mut moves,
        position.piece_bb(crate::bitboard::Piece::Bishop, color),
        occupied,
        enemies,
    );
    generate_rook_moves(
        &mut moves,
        position.piece_bb(crate::bitboard::Piece::Rook, color),
        occupied,
        enemies,
    );
    generate_queen_moves(
        &mut moves,
        position.piece_bb(crate::bitboard::Piece::Queen, color),
        occupied,
        enemies,
    );
    if let Some(king_sq) = position.piece_bb(crate::bitboard::Piece::King, color).lsb() {
        generate_king_moves(&mut moves, king_sq, occupied, enemies);
    }

    // Filter legal moves
    let king_sq = position
        .piece_bb(crate::bitboard::Piece::King, color)
        .lsb()
        .unwrap_or(crate::bitboard::Square::E1);
    let enemy_attacks = crate::bitboard::Bitboard::EMPTY;
    let legal_moves = filter_legal_moves(
        &moves,
        position,
        color,
    );

    if legal_moves.is_empty() {
        // No moves: checkmate or stalemate
        return PvsResult {
            score: 0,
            pv: Vec::new(),
        };
    }

    for &mv in legal_moves.iter() {
        let mut child_position = position.clone();
        let undo = child_position.make_move(mv);

        let score = if first_move {
            // Full window for first move
            -pvs_search(
                depth - 1,
                -beta,
                -alpha,
                color.opposite(),
                evaluator,
                &child_position,
            )
            .score
        } else {
            // Null window for subsequent moves
            let mut score = -pvs_search(
                depth - 1,
                -alpha - 1,
                -alpha,
                color.opposite(),
                evaluator,
                &child_position,
            )
            .score;
            if score > alpha && score < beta {
                // Re-search with full window
                score = -pvs_search(
                    depth - 1,
                    -beta,
                    -alpha,
                    color.opposite(),
                    evaluator,
                    &child_position,
                )
                .score;
            }
            score
        };

        child_position.unmake_move(undo);

        if score > best_score {
            best_score = score;
            best_pv = vec![mv];
        }

        if score > alpha {
            alpha = score;
        }

        if alpha >= beta {
            break;
        }

        first_move = false;
    }

    PvsResult {
        score: best_score,
        pv: best_pv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pvs_structure() {
        let evaluator = Evaluator::new();
        let dummy_position = crate::bitboard::position::Position::empty();
        let result = pvs_search(
            1,
            i32::MIN / 2,
            i32::MAX / 2,
            Color::White,
            &evaluator,
            &dummy_position,
        );
        // Basic test that it returns a result
        assert!(result.score >= i32::MIN / 2 && result.score <= i32::MAX / 2);
    }
}
