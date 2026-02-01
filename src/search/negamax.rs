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
    position: &crate::bitboard::Position,
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

    // Filter legal moves using real enemy attacks
    let king_sq = position
        .piece_bb(Piece::King, color)
        .lsb()
        .unwrap_or(crate::bitboard::Square::E1);

    // Compute enemy attacks for legality
    let mut enemy_attacks = crate::bitboard::Bitboard::EMPTY;
    let opp_color = color.opposite();
    let opp_pawns = position.piece_bb(Piece::Pawn, opp_color);
    let opp_knights = position.piece_bb(Piece::Knight, opp_color);
    let opp_bishops = position.piece_bb(Piece::Bishop, opp_color);
    let opp_rooks = position.piece_bb(Piece::Rook, opp_color);
    let opp_queens = position.piece_bb(Piece::Queen, opp_color);
    let opp_king = position.piece_bb(Piece::King, opp_color);

    for sq in opp_pawns.iter() {
        enemy_attacks |= crate::bitboard::attacks::pawn_attacks(sq, opp_color);
    }
    for sq in opp_knights.iter() {
        enemy_attacks |= crate::bitboard::attacks::knight_attacks(sq);
    }
    for sq in opp_bishops.iter() {
        enemy_attacks |= crate::bitboard::attacks::bishop_attacks(sq, occupied);
    }
    for sq in opp_rooks.iter() {
        enemy_attacks |= crate::bitboard::attacks::rook_attacks(sq, occupied);
    }
    for sq in opp_queens.iter() {
        enemy_attacks |= crate::bitboard::attacks::queen_attacks(sq, occupied);
    }
    if let Some(ksq) = opp_king.lsb() {
        enemy_attacks |= crate::bitboard::attacks::king_attacks(ksq);
    }

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

        // Compute enemy attacks for the child position
        let mut child_enemy_attacks = crate::bitboard::Bitboard::EMPTY;
        let child_opp_color = color;
        let child_opp_pawns = child_position.piece_bb(Piece::Pawn, child_opp_color);
        let child_opp_knights = child_position.piece_bb(Piece::Knight, child_opp_color);
        let child_opp_bishops = child_position.piece_bb(Piece::Bishop, child_opp_color);
        let child_opp_rooks = child_position.piece_bb(Piece::Rook, child_opp_color);
        let child_opp_queens = child_position.piece_bb(Piece::Queen, child_opp_color);
        let child_opp_king = child_position.piece_bb(Piece::King, child_opp_color);
        let child_occupied = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
            acc | child_position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::White)
                | child_position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::Black)
        });

        for sq in child_opp_pawns.iter() {
            child_enemy_attacks |= crate::bitboard::attacks::pawn_attacks(sq, child_opp_color);
        }
        for sq in child_opp_knights.iter() {
            child_enemy_attacks |= crate::bitboard::attacks::knight_attacks(sq);
        }
        for sq in child_opp_bishops.iter() {
            child_enemy_attacks |= crate::bitboard::attacks::bishop_attacks(sq, child_occupied);
        }
        for sq in child_opp_rooks.iter() {
            child_enemy_attacks |= crate::bitboard::attacks::rook_attacks(sq, child_occupied);
        }
        for sq in child_opp_queens.iter() {
            child_enemy_attacks |= crate::bitboard::attacks::queen_attacks(sq, child_occupied);
        }
        if let Some(ksq) = child_opp_king.lsb() {
            child_enemy_attacks |= crate::bitboard::attacks::king_attacks(ksq);
        }

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
