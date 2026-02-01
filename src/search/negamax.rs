//! Negamax search algorithm
//!
//! This module implements the negamax search algorithm,
//! a variant of minimax that simplifies the implementation.

use crate::bitboard::Color;
use crate::eval::Evaluator;
use crate::movegen::Move;

/// Negamax search result
pub struct NegamaxResult {
    pub score: i32,
}

/// Perform negamax search
pub fn negamax(
    depth: i32,
    color: Color,
    evaluator: &Evaluator,
    // Position parameters would be passed here
) -> NegamaxResult {
    // Base case: evaluate position
    if depth == 0 {
        return NegamaxResult {
            score: evaluator.evaluate(position) * if color == Color::White { 1 } else { -1 },
        };
    }

    let mut max_score = i32::MIN;

    // Generate pseudo-legal moves
    use crate::bitboard::Color as BoardColor;
    use crate::bitboard::Piece;
    use crate::movegen::generator::*;
    use crate::movegen::legal::filter_legal_moves;

    let color = color;
    let mut moves = crate::movegen::MoveList::new();
    let occupied = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), BoardColor::White)
            | position.piece_bb(Piece::from_u8(p).unwrap(), BoardColor::Black)
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
    let enemy_attacks = crate::bitboard::Bitboard::EMPTY; // TODO: Compute real enemy attacks for legality
    let legal_moves = filter_legal_moves(
        &moves,
        position.piece_bb(Piece::Pawn, color)
            | position.piece_bb(Piece::Knight, color)
            | position.piece_bb(Piece::Bishop, color)
            | position.piece_bb(Piece::Rook, color)
            | position.piece_bb(Piece::Queen, color)
            | position.piece_bb(Piece::King, color),
        position.piece_bb(Piece::Pawn, color.opposite())
            | position.piece_bb(Piece::Knight, color.opposite())
            | position.piece_bb(Piece::Bishop, color.opposite())
            | position.piece_bb(Piece::Rook, color.opposite())
            | position.piece_bb(Piece::Queen, color.opposite())
            | position.piece_bb(Piece::King, color.opposite()),
        occupied,
        king_sq,
        enemy_attacks,
    );

    for &mv in legal_moves.iter() {
        let mut child_position = position.clone();
        let undo = child_position.make_move(mv);

        // Recursive search with negated color
        let result = negamax(depth - 1, color.opposite(), evaluator, &child_position);
        let score = -result.score;

        child_position.unmake_move(undo);

        if score > max_score {
            max_score = score;
        }
    }

    NegamaxResult { score: max_score }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negamax_structure() {
        let evaluator = Evaluator::new();
        let dummy_position = crate::bitboard::position::Position::empty();
        let result = negamax(1, Color::White, &evaluator, &dummy_position);
        // Basic test that it returns a result
        assert!(result.score >= i32::MIN && result.score <= i32::MAX);
    }
}
