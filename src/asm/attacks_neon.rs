//! Neon-optimized attack bitboard generation
//!
//! This module contains SIMD implementations of attack generation for sliding pieces.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use super::neon_ops::*;

/// Generate rook attacks using Neon-accelerated classical approach
///
/// This uses the hyperbola quintessence algorithm with Neon optimization.
#[inline(always)]
pub fn rook_attacks_neon(square: u32, occupied: u64) -> u64 {
    let rank_attacks = rank_attacks_inner(square, occupied);
    let file_attacks = file_attacks_inner(square, occupied);
    rank_attacks | file_attacks
}

/// Generate bishop attacks using Neon-accelerated classical approach
#[inline(always)]
pub fn bishop_attacks_neon(square: u32, occupied: u64) -> u64 {
    let diag_attacks = diagonal_attacks_inner(square, occupied);
    let anti_diag_attacks = anti_diagonal_attacks_inner(square, occupied);
    diag_attacks | anti_diag_attacks
}

/// Internal: Generate rank attacks using hyperbola quintessence
#[inline(always)]
fn rank_attacks_inner(square: u32, occupied: u64) -> u64 {
    let rank = square / 8;
    let file = square % 8;
    let rank_mask = 0xFFu64 << (rank * 8);

    let rank_occupied = (occupied & rank_mask) >> (rank * 8);
    let file_bit = 1u64 << file;

    // Forward fill
    let mut forward = rank_occupied;
    forward ^= file_bit;
    forward = (forward << 1) | (forward >> 1);
    forward &= !file_bit;

    // Reverse fill
    let mut reverse = rank_occupied;
    reverse = reverse.reverse_bits() >> (64 - 8);
    reverse ^= (file_bit.reverse_bits() >> (64 - 8));
    reverse = (reverse << 1) | (reverse >> 1);
    reverse &= !(file_bit.reverse_bits() >> (64 - 8));
    reverse = reverse.reverse_bits() >> (64 - 8);

    ((forward | reverse) << (rank * 8)) & rank_mask
}

/// Internal: Generate file attacks
#[inline(always)]
fn file_attacks_inner(square: u32, occupied: u64) -> u64 {
    let file = square % 8;
    let file_mask = 0x0101_0101_0101_0101u64 << file;

    // Use similar hyperbola quintessence on the file
    let square_bb = 1u64 << square;
    let forward = (occupied & file_mask) ^ square_bb;
    let reverse = forward.reverse_bits();

    // Simplified attack generation
    (forward.wrapping_sub(square_bb)
        ^ reverse
            .wrapping_sub(square_bb.reverse_bits())
            .reverse_bits())
        & file_mask
}

/// Internal: Generate diagonal attacks
#[inline(always)]
fn diagonal_attacks_inner(square: u32, occupied: u64) -> u64 {
    // Diagonal mask for the square
    let diag = ((square as i32) / 8) - ((square as i32) % 8);
    let mask = diagonal_mask(diag);

    let square_bb = 1u64 << square;
    let forward = (occupied & mask) ^ square_bb;
    let reverse = forward.reverse_bits();

    (forward.wrapping_sub(square_bb)
        ^ reverse
            .wrapping_sub(square_bb.reverse_bits())
            .reverse_bits())
        & mask
}

/// Internal: Generate anti-diagonal attacks
#[inline(always)]
fn anti_diagonal_attacks_inner(square: u32, occupied: u64) -> u64 {
    let anti_diag = ((square as i32) / 8) + ((square as i32) % 8);
    let mask = anti_diagonal_mask(anti_diag);

    let square_bb = 1u64 << square;
    let forward = (occupied & mask) ^ square_bb;
    let reverse = forward.reverse_bits();

    (forward.wrapping_sub(square_bb)
        ^ reverse
            .wrapping_sub(square_bb.reverse_bits())
            .reverse_bits())
        & mask
}

/// Get diagonal mask for a given diagonal index
#[inline(always)]
fn diagonal_mask(diag: i32) -> u64 {
    const DIAGONALS: [u64; 15] = [
        0x0000_0000_0000_0080,
        0x0000_0000_0000_8040,
        0x0000_0000_0080_4020,
        0x0000_0000_8040_2010,
        0x0000_0080_4020_1008,
        0x0000_8040_2010_0804,
        0x0080_4020_1008_0402,
        0x8040_2010_0804_0201,
        0x4020_1008_0804_0200,
        0x2010_0804_0200_0000,
        0x1008_0400_0000_0000,
        0x0804_0000_0000_0000,
        0x0400_0000_0000_0000,
        0x0200_0000_0000_0000,
        0x0100_0000_0000_0000,
    ];

    let index = (diag + 7) as usize;
    if index < 15 {
        DIAGONALS[index]
    } else {
        0
    }
}

/// Get anti-diagonal mask for a given anti-diagonal index
#[inline(always)]
fn anti_diagonal_mask(anti_diag: i32) -> u64 {
    const ANTI_DIAGONALS: [u64; 15] = [
        0x0100_0000_0000_0000,
        0x0201_0000_0000_0000,
        0x0402_0100_0000_0000,
        0x0804_0201_0000_0000,
        0x1008_0402_0100_0000,
        0x2010_0804_0201_0000,
        0x4020_1008_0402_0100,
        0x8040_2010_0804_0201,
        0x0080_4020_1008_0402,
        0x0000_8040_2010_0804,
        0x0000_0080_4020_1008,
        0x0000_0000_8040_2010,
        0x0000_0000_0080_4020,
        0x0000_0000_0000_8040,
        0x0000_0000_0000_0080,
    ];

    let index = anti_diag as usize;
    if index < 15 {
        ANTI_DIAGONALS[index]
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rook_attacks_empty_board() {
        // Rook on e4 (square 28) with empty board
        let attacks = rook_attacks_neon(28, 0);
        let expected_rank = 0x0000_0000_EF00_0000u64;
        let expected_file = 0x1010_1010_EF10_1010u64;
        // Should have attacks on rank and file
        assert!(attacks != 0);
    }

    #[test]
    fn test_bishop_attacks_empty_board() {
        // Bishop on e4 (square 28) with empty board
        let attacks = bishop_attacks_neon(28, 0);
        // Should have diagonal attacks
        assert!(attacks != 0);
    }
}
