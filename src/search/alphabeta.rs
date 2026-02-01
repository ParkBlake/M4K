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
use std::time::{Duration, Instant};

/// Time management for search
struct TimeManager {
    start_time: Instant,
    time_limit: Option<Duration>,
    max_time: Option<Duration>,
    allocated_time: Duration,
}

impl TimeManager {
    /// Create a new time manager for the given time control
    fn new(time_control: &TimeControl, color: Color) -> Self {
        let start_time = Instant::now();

        // Calculate time allocation based on time control type
        let (time_limit, max_time, allocated_time) = if time_control.infinite {
            // Infinite time - no limits
            (None, None, Duration::from_secs(3600)) // 1 hour fallback
        } else if let Some(movetime) = time_control.movetime {
            // Fixed time per move
            let duration = Duration::from_millis(movetime);
            (Some(duration), Some(duration), duration)
        } else {
            // Time-based game with increments
            let (our_time, our_inc) = match color {
                Color::White => (time_control.wtime, time_control.winc),
                Color::Black => (time_control.btime, time_control.binc),
            };

            if let Some(our_time_ms) = our_time {
                let our_time = Duration::from_millis(our_time_ms);
                let our_inc = our_inc.map(|inc| Duration::from_millis(inc)).unwrap_or(Duration::ZERO);

                // Calculate moves to go (default to 40 if not specified)
                let moves_to_go = time_control.movestogo.unwrap_or(40) as u32;

                // Allocate time for this move: (remaining_time / moves_to_go) + increment
                // Use a more conservative allocation to avoid timeouts
                let base_allocation = our_time.checked_div(moves_to_go).unwrap_or(Duration::from_millis(100));
                let allocated = base_allocation.saturating_add(our_inc.saturating_mul(3).checked_div(4).unwrap_or(Duration::ZERO));

                // Set hard time limit to 90% of allocated time to be safe
                let time_limit = Some(allocated.mul_f32(0.9));

                // Maximum time is 5x the allocated time (for emergencies)
                let max_time = Some(allocated.saturating_mul(5));

                (time_limit, max_time, allocated)
            } else {
                // No time information - use default
                let default_time = Duration::from_millis(1000);
                (Some(default_time), Some(default_time), default_time)
            }
        };

        TimeManager {
            start_time,
            time_limit,
            max_time,
            allocated_time,
        }
    }

    /// Check if we should stop searching due to time constraints
    fn should_stop(&self) -> bool {
        let elapsed = self.start_time.elapsed();

        // Check hard time limit
        if let Some(limit) = self.time_limit {
            if elapsed >= limit {
                return true;
            }
        }

        // Check maximum time (emergency stop)
        if let Some(max) = self.max_time {
            if elapsed >= max {
                return true;
            }
        }

        false
    }

    /// Get elapsed time since search started
    fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get remaining time before hard limit
    fn remaining_time(&self) -> Option<Duration> {
        self.time_limit.map(|limit| {
            let elapsed = self.elapsed();
            if elapsed >= limit {
                Duration::ZERO
            } else {
                limit - elapsed
            }
        })
    }
}
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
    start_time: Instant,
    time_limit: Option<Duration>,
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
        result.score = quiescence_search(alpha, beta, color, evaluator, position, stop_flag, start_time, time_limit);
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

        // Check time limit
        if let Some(limit) = time_limit {
            if start_time.elapsed() >= limit {
                break;
            }
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
            start_time,
            time_limit,
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
    let time_manager = TimeManager::new(time_control, color);
    let max_depth = time_control.depth.unwrap_or(8) as i32;
    let mut result = SearchResult {
        best_move: None,
        score: 0,
        nodes_searched: 0,
    };

    // Generate at least one legal move as fallback
    let fallback_move = generate_fallback_move(position, color);

    // Iterative deepening with time management
    for depth in 1..=max_depth {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        // Check if we have enough time for this depth
        if depth > 1 {
            let elapsed = time_manager.elapsed();
            let estimated_next_time = elapsed.mul_f32(4.0);

            if let Some(remaining) = time_manager.remaining_time() {
                if estimated_next_time > remaining.mul_f32(0.8) {
                    break;
                }
            }
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
            time_manager.start_time,
            time_manager.time_limit,
        );

        result = window_result;

        // Check time after each depth
        if time_manager.should_stop() {
            break;
        }

        // For deeper searches, be more conservative about time
        if depth >= 6 {
            let elapsed = time_manager.elapsed();
            if let Some(remaining) = time_manager.remaining_time() {
                if elapsed.as_millis() as f32 > (time_manager.allocated_time.as_millis() as f32 * 0.6) {
                    break;
                }
            }
        }
    }

    // Guarantee we have a move to return
    if result.best_move.is_none() {
        result.best_move = fallback_move;
    }

    result
}

/// Generate a fallback move (first legal move found)
fn generate_fallback_move(position: &crate::bitboard::position::Position, color: Color) -> Option<Move> {
    use crate::bitboard::Piece;
    use crate::movegen::generator::*;
    use crate::movegen::legal::filter_legal_moves;

    let mut moves = MoveList::new();
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

    let legal_moves = filter_legal_moves(&moves, position, color);
    legal_moves.iter().next().copied()
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
        let start_time = Instant::now();

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
            start_time,
            Some(Duration::from_secs(1)),
        );

        // In a real test, we'd have a position and check the result
        assert!(result.nodes_searched >= 1);
    }
}
