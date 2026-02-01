//! Alpha-beta search algorithm
//!
//! This module implements the alpha-beta pruning algorithm for chess search.

use super::quiescence::quiescence_search;
use crate::bitboard::{Bitboard, Color};
use crate::eval::Evaluator;
use crate::movegen::{Move, MoveList};
use crate::search::transposition::{TTEntry, TranspositionTable};
use crate::uci::commands::TimeControl;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Search result containing the best move and score
#[derive(Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub nodes_searched: u64,
}

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

/// Alpha-beta search with transposition table
pub fn alpha_beta_search(
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    color: Color,
    tt: &mut TranspositionTable,
    evaluator: &Evaluator,
    position: &crate::bitboard::position::Position,
    stop_flag: &Arc<AtomicBool>,
) -> SearchResult {
    let mut result = SearchResult {
        best_move: None,
        score: 0,
        nodes_searched: 1, // Count this node
    };

    // Check transposition table
    let pos_hash = position.zobrist_hash().value();
    if let Some(tt_entry) = tt.probe(pos_hash) {
        if tt_entry.depth >= depth {
            match tt_entry.node_type {
                crate::search::transposition::NodeType::Exact => {
                    return SearchResult {
                        best_move: Some(tt_entry.best_move),
                        score: tt_entry.score,
                        nodes_searched: 1,
                    };
                }
                crate::search::transposition::NodeType::Lower => {
                    alpha = alpha.max(tt_entry.score);
                }
                crate::search::transposition::NodeType::Upper => {
                    beta = beta.min(tt_entry.score);
                }
            }
            if alpha >= beta {
                return SearchResult {
                    best_move: Some(tt_entry.best_move),
                    score: tt_entry.score,
                    nodes_searched: 1,
                };
            }
        }
    }

    // Base case: depth 0, go to quiescence
    if depth == 0 {
        result.score = quiescence_search(alpha, beta, color, evaluator, position, stop_flag);
        return result;
    }

    // Generate pseudo-legal moves
    use crate::bitboard::Piece;
    use crate::movegen::generator::*;
    use crate::movegen::legal::filter_legal_moves;

    let mut moves = MoveList::new();
    let color = color;
    let occupied = (0..6).fold(Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::White)
            | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::Black)
    });
    let enemies = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), color.opposite())
    });

    generate_pawn_moves(
        &mut moves,
        position.piece_bb(Piece::Pawn, color),
        occupied,
        enemies,
        color,
        position.en_passant,
    );
    generate_knight_moves(
        &mut moves,
        position.piece_bb(Piece::Knight, color),
        occupied,
        enemies,
    );
    generate_bishop_moves(
        &mut moves,
        position.piece_bb(Piece::Bishop, color),
        occupied,
        enemies,
    );
    generate_rook_moves(
        &mut moves,
        position.piece_bb(Piece::Rook, color),
        occupied,
        enemies,
    );
    generate_queen_moves(
        &mut moves,
        position.piece_bb(Piece::Queen, color),
        occupied,
        enemies,
    );
    if let Some(king_sq) = position.piece_bb(Piece::King, color).lsb() {
        generate_king_moves(&mut moves, king_sq, occupied, enemies);
    }

    // Filter legal moves
    let king_sq = position
        .piece_bb(Piece::King, color)
        .lsb()
        .unwrap_or(crate::bitboard::Square::E1);
    // Compute enemy attacks for legality check
    let enemy_attacks = compute_enemy_attacks(position, color.opposite());
    let legal_moves = filter_legal_moves(
        &moves,
        position,
        color,
    );

    let mut best_score = i32::MIN;
    let mut best_move = None;
    let mut node_type = crate::search::transposition::NodeType::Upper;

    for &mv in legal_moves.iter() {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let mut child_position = position.clone();
        let undo = child_position.make_move(mv);

        // Recursive search with negated score
        let child_result = alpha_beta_search(
            depth - 1,
            -beta,
            -alpha,
            color.opposite(),
            tt,
            evaluator,
            &child_position,
            stop_flag,
        );

        let score = -child_result.score;
        result.nodes_searched += child_result.nodes_searched;

        child_position.unmake_move(undo);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        alpha = alpha.max(score);
        if alpha >= beta {
            // Beta cutoff
            node_type = crate::search::transposition::NodeType::Lower;
            break;
        }
    }

    result.score = best_score;
    result.best_move = best_move;

    // Store in transposition table
    if let Some(mv) = best_move {
        tt.store(
            pos_hash,
            TTEntry {
                score: best_score,
                best_move: mv,
                depth,
                node_type,
            },
        );
    }

    result
}

/// Iterative deepening alpha-beta search
pub fn iterative_deepening(
    time_control: &TimeControl,
    color: Color,
    tt: &mut TranspositionTable,
    evaluator: &Evaluator,
    position: &crate::bitboard::position::Position,
    stop_flag: &Arc<AtomicBool>,
) -> SearchResult {
    let max_depth = time_control.depth.unwrap_or(4) as i32;
    let mut result = SearchResult {
        best_move: None,
        score: 0,
        nodes_searched: 0,
    };

    for depth in 1..=max_depth {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let window_result = alpha_beta_search(
            depth,
            i32::MIN / 2,
            i32::MAX / 2,
            color,
            tt,
            evaluator,
            position,
            stop_flag,
        );

        result = window_result;

        // Could add time management here
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpha_beta_structure() {
        // Basic test that the functions exist and can be called
        let mut tt = TranspositionTable::new();
        let evaluator = Evaluator::new();
        let stop_flag = Arc::new(AtomicBool::new(false));

        let dummy_position = crate::bitboard::position::Position::empty();
        let result = alpha_beta_search(
            1,
            i32::MIN / 2,
            i32::MAX / 2,
            Color::White,
            &mut tt,
            &evaluator,
            &dummy_position,
            &stop_flag,
        );

        // In a real test, we'd have a position and check the result
        assert!(result.nodes_searched >= 1);
    }
}
