//! Attack bitboard generation for all piece types
//!
//! This module provides functions to generate attack bitboards for each piece type.
//! It includes precomputed lookup tables for efficient attack generation.

use super::types::*;
use crate::asm::prelude::*;

/// Precomputed knight attack table
static KNIGHT_ATTACKS: [Bitboard; 64] = generate_knight_attacks();

/// Precomputed king attack table
static KING_ATTACKS: [Bitboard; 64] = generate_king_attacks();

/// Precomputed pawn attack tables (one for each color)
static PAWN_ATTACKS: [[Bitboard; 64]; 2] = generate_pawn_attacks();

/// Generate knight attacks for all squares
const fn generate_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0;

    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;

        // All possible knight moves (relative to current square)
        let deltas = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        let mut i = 0;
        while i < deltas.len() {
            let (dr, df) = deltas[i];
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_sq = (new_rank * 8 + new_file) as u8;
                attacks[sq].0 |= 1u64 << target_sq;
            }

            i += 1;
        }

        sq += 1;
    }

    attacks
}

/// Generate king attacks for all squares
const fn generate_king_attacks() -> [Bitboard; 64] {
    let mut attacks = [Bitboard::EMPTY; 64];
    let mut sq = 0;

    while sq < 64 {
        let file = sq % 8;
        let rank = sq / 8;

        // All possible king moves (relative to current square)
        let deltas = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let mut i = 0;
        while i < deltas.len() {
            let (dr, df) = deltas[i];
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let target_sq = (new_rank * 8 + new_file) as u8;
                attacks[sq].0 |= 1u64 << target_sq;
            }

            i += 1;
        }

        sq += 1;
    }

    attacks
}

/// Generate pawn attacks for all squares and both colors
const fn generate_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut attacks = [[Bitboard::EMPTY; 64]; 2];

    // White pawns
    let mut sq = 0;
    while sq < 64 {
        let file = sq % 8;

        // White pawn attacks: up-left and up-right
        if file > 0 {
            let target = sq + 7; // up-left for white
            if target < 64 {
                attacks[Color::White as usize][sq].0 |= 1u64 << target;
            }
        }
        if file < 7 {
            let target = sq + 9; // up-right for white
            if target < 64 {
                attacks[Color::White as usize][sq].0 |= 1u64 << target;
            }
        }

        sq += 1;
    }

    // Black pawns
    sq = 0;
    while sq < 64 {
        let file = sq % 8;

        // Black pawn attacks: down-left and down-right
        if file > 0 {
            let target = sq.wrapping_sub(9); // down-left for black
            if target < 64 {
                attacks[Color::Black as usize][sq].0 |= 1u64 << target;
            }
        }
        if file < 7 {
            let target = sq.wrapping_sub(7); // down-right for black
            if target < 64 {
                attacks[Color::Black as usize][sq].0 |= 1u64 << target;
            }
        }

        sq += 1;
    }

    attacks
}

/// Get knight attacks for a square
#[inline(always)]
pub fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square.0 as usize]
}

/// Get king attacks for a square
#[inline(always)]
pub fn king_attacks(square: Square) -> Bitboard {
    KING_ATTACKS[square.0 as usize]
}

/// Get pawn attacks for a square and color
#[inline(always)]
pub fn pawn_attacks(square: Square, color: Color) -> Bitboard {
    PAWN_ATTACKS[color as usize][square.0 as usize]
}

/// Generate bishop attacks using magic bitboards or NEON when available
pub fn bishop_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    #[cfg(target_arch = "aarch64")]
    {
        Bitboard(crate::asm::attacks_neon::bishop_attacks_neon(square.0 as u32, occupied.0))
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        unsafe { crate::bitboard::magic::bishop_attacks_magic(square, occupied) }
    }
}

/// Generate rook attacks using magic bitboards or NEON when available
pub fn rook_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    #[cfg(target_arch = "aarch64")]
    {
        Bitboard(crate::asm::attacks_neon::rook_attacks_neon(square.0 as u32, occupied.0))
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        unsafe { crate::bitboard::magic::rook_attacks_magic(square, occupied) }
    }
}

/// Generate queen attacks (combination of bishop and rook)
#[inline(always)]
pub fn queen_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    unsafe { crate::bitboard::magic::queen_attacks_magic(square, occupied) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knight_attacks() {
        let attacks = knight_attacks(Square::E4);
        // Knight on e4 should attack d2, f2, c3, g3, c5, g5, d6, f6
        assert_eq!(attacks.count(), 8);

        // Test corner case
        let corner_attacks = knight_attacks(Square::A1);
        assert_eq!(corner_attacks.count(), 2); // Only B3 and C2
    }

    #[test]
    fn test_king_attacks() {
        let attacks = king_attacks(Square::E4);
        // King on e4 should attack all 8 adjacent squares
        assert_eq!(attacks.count(), 8);

        // Test corner case
        let corner_attacks = king_attacks(Square::A1);
        assert_eq!(corner_attacks.count(), 3); // Only A2, B1, B2
    }

    #[test]
    fn test_pawn_attacks() {
        // White pawn on e4
        let white_attacks = pawn_attacks(Square::E4, Color::White);
        assert!(white_attacks.is_occupied(Square::D5));
        assert!(white_attacks.is_occupied(Square::F5));
        assert_eq!(white_attacks.count(), 2);

        // Black pawn on e4
        let black_attacks = pawn_attacks(Square::E4, Color::Black);
        assert!(black_attacks.is_occupied(Square::D3));
        assert!(black_attacks.is_occupied(Square::F3));
        assert_eq!(black_attacks.count(), 2);
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        crate::bitboard::magic::init_magics();
        let attacks = bishop_attacks(Square::E4, Bitboard::EMPTY);
        // Bishop on e4 should attack all diagonals
        assert!(attacks.is_occupied(Square::D3));
        assert!(attacks.is_occupied(Square::F5));
        assert!(attacks.is_occupied(Square::A8));
        assert!(attacks.is_occupied(Square::H7));
    }

    #[test]
    fn test_rook_attacks_empty_board() {
        crate::bitboard::magic::init_magics();
        let attacks = rook_attacks(Square::E4, Bitboard::EMPTY);
        // Rook on e4 should attack entire rank and file
        assert!(attacks.is_occupied(Square::E1));
        assert!(attacks.is_occupied(Square::E8));
        assert!(attacks.is_occupied(Square::A4));
        assert!(attacks.is_occupied(Square::H4));
    }
}
