//! Quiescence search - Extend search in tactical positions
//!
//! This module implements quiescence search, which extends the main search
//! into positions with captures and checks to avoid the horizon effect.

use crate::bitboard::Color;
use crate::eval::Evaluator;
use crate::movegen::Move;

/// Quiescence search to evaluate quiet positions
///
/// This function searches captures and other tactical moves to ensure
/// the evaluation is stable and not affected by the horizon effect.
pub fn quiescence_search(
    mut alpha: i32,
    beta: i32,
    color: Color,
    evaluator: &Evaluator,
    position: &crate::bitboard::position::Position,
) -> i32 {
    // Stand pat: evaluate the current position
    let stand_pat = evaluator.evaluate(position);
    let stand_pat = if color == crate::bitboard::Color::White {
        stand_pat
    } else {
        -stand_pat
    };

    // Beta cutoff: if standing pat is better than beta, we can stop
    if stand_pat >= beta {
        return beta;
    }

    // Update alpha with stand pat
    alpha = alpha.max(stand_pat);

    // Generate all capture moves
    use crate::bitboard::Piece;
    use crate::movegen::generator::*;
    use crate::movegen::legal::filter_legal_moves;

    let mut captures = crate::movegen::MoveList::new();
    let occupied = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::White)
            | position.piece_bb(Piece::from_u8(p).unwrap(), crate::bitboard::Color::Black)
    });
    let enemies = (0..6).fold(crate::bitboard::Bitboard::EMPTY, |acc, p| {
        acc | position.piece_bb(Piece::from_u8(p).unwrap(), color.opposite())
    });

    // Only generate captures for each piece type
    generate_pawn_moves(
        &mut captures,
        position.piece_bb(Piece::Pawn, color),
        occupied,
        enemies,
        color,
        position.en_passant,
    );
    generate_knight_moves(
        &mut captures,
        position.piece_bb(Piece::Knight, color),
        occupied,
        enemies,
    );
    generate_bishop_moves(
        &mut captures,
        position.piece_bb(Piece::Bishop, color),
        occupied,
        enemies,
    );
    generate_rook_moves(
        &mut captures,
        position.piece_bb(Piece::Rook, color),
        occupied,
        enemies,
    );
    generate_queen_moves(
        &mut captures,
        position.piece_bb(Piece::Queen, color),
        occupied,
        enemies,
    );
    if let Some(king_sq) = position.piece_bb(Piece::King, color).lsb() {
        generate_king_moves(&mut captures, king_sq, occupied, enemies);
    }

    // Filter only capturing moves
    let captures: Vec<_> = captures
        .iter()
        .cloned()
        .filter(|mv| {
            // A move is a capture if the destination square is occupied by an enemy piece
            let to = mv.to();
            (0..6).any(|p| {
                position
                    .piece_bb(Piece::from_u8(p).unwrap(), color.opposite())
                    .is_occupied(to)
            }) || mv.is_en_passant()
        })
        .collect();

    for mv in captures {
        let mut child_position = position.clone();
        let undo = child_position.make_move(mv);

        // Recursive quiescence search
        let score = -quiescence_search(-beta, -alpha, color.opposite(), evaluator, &child_position);

        child_position.unmake_move(undo);

        // Beta cutoff
        if score >= beta {
            return beta;
        }

        // Update alpha
        alpha = alpha.max(score);
    }

    alpha
}

/// Check if a position is quiet (no captures or checks)
pub fn is_quiet_position(
    // Position parameters
    _captures_available: bool,
    _in_check: bool,
) -> bool {
    // A position is quiet if there are no captures and we're not in check
    !_captures_available && !_in_check
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Evaluator;

    #[test]
    fn test_quiescence_structure() {
        let evaluator = Evaluator::new();

        // Basic test that quiescence search can be called
        let dummy_position = crate::bitboard::position::Position::empty();
        let score = quiescence_search(
            i32::MIN / 2,
            i32::MAX / 2,
            Color::White,
            &evaluator,
            &dummy_position,
        );

        // In a real test, we'd check the score bounds
        assert!(score >= i32::MIN / 2 && score <= i32::MAX / 2);
    }
}
